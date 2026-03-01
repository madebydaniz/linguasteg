use argon2::{Algorithm, Argon2, Params, Version};
use chacha20poly1305::aead::{Aead, KeyInit};
use chacha20poly1305::{Key, XChaCha20Poly1305, XNonce};

pub type CryptoEnvelopeResult<T> = Result<T, CryptoEnvelopeError>;

const ENVELOPE_MAGIC: [u8; 4] = *b"LSTG";
const ENVELOPE_VERSION_V1: u8 = 1;
const KDF_ARGON2ID: u8 = 1;
const AEAD_XCHACHA20POLY1305: u8 = 1;
const SALT_LEN: usize = 16;
const NONCE_LEN: usize = 24;
const KEY_LEN: usize = 32;
const HEADER_LEN: usize = 13;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KeyDerivationParams {
    pub memory_kib: u32,
    pub iterations: u32,
    pub parallelism: u32,
}

impl Default for KeyDerivationParams {
    fn default() -> Self {
        Self {
            memory_kib: 19_456,
            iterations: 2,
            parallelism: 1,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct CryptoEnvelopeConfig {
    pub kdf: KeyDerivationParams,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CryptoEnvelopeError {
    SecretRequired,
    RandomnessUnavailable(String),
    InvalidEnvelope(String),
    UnsupportedVersion(u8),
    UnsupportedAlgorithms { kdf: u8, aead: u8 },
    KeyDerivationFailed(String),
    EncryptFailed,
    DecryptFailed,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CryptoEnvelopeMetadata {
    pub version: u8,
    pub kdf: u8,
    pub aead: u8,
    pub salt_len: u8,
    pub nonce_len: u8,
    pub ciphertext_len: u32,
    pub total_len: usize,
}

impl CryptoEnvelopeMetadata {
    pub fn kdf_name(&self) -> &'static str {
        kdf_name(self.kdf)
    }

    pub fn aead_name(&self) -> &'static str {
        aead_name(self.aead)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CryptoEnvelopeInspection {
    NotEnvelope,
    Metadata(CryptoEnvelopeMetadata),
    Invalid(String),
}

impl core::fmt::Display for CryptoEnvelopeError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::SecretRequired => write!(f, "secret is required"),
            Self::RandomnessUnavailable(message) => {
                write!(f, "randomness source is unavailable: {message}")
            }
            Self::InvalidEnvelope(message) => write!(f, "invalid crypto envelope: {message}"),
            Self::UnsupportedVersion(version) => {
                write!(f, "unsupported crypto envelope version: {version}")
            }
            Self::UnsupportedAlgorithms { kdf, aead } => {
                write!(f, "unsupported crypto algorithms: kdf={kdf}, aead={aead}")
            }
            Self::KeyDerivationFailed(message) => {
                write!(f, "failed to derive encryption key: {message}")
            }
            Self::EncryptFailed => write!(f, "failed to encrypt payload"),
            Self::DecryptFailed => write!(f, "failed to decrypt payload"),
        }
    }
}

impl std::error::Error for CryptoEnvelopeError {}

#[derive(Debug, Clone, PartialEq, Eq)]
struct EnvelopeHeader {
    version: u8,
    kdf: u8,
    aead: u8,
    salt_len: u8,
    nonce_len: u8,
    ciphertext_len: u32,
}

pub fn seal_payload(payload: &[u8], secret: &[u8]) -> CryptoEnvelopeResult<Vec<u8>> {
    seal_payload_with_config(payload, secret, &CryptoEnvelopeConfig::default())
}

pub fn seal_payload_with_config(
    payload: &[u8],
    secret: &[u8],
    config: &CryptoEnvelopeConfig,
) -> CryptoEnvelopeResult<Vec<u8>> {
    require_secret(secret)?;

    let mut salt = [0u8; SALT_LEN];
    fill_random(&mut salt)?;

    let mut nonce = [0u8; NONCE_LEN];
    fill_random(&mut nonce)?;

    seal_payload_with_material(payload, secret, config, &salt, &nonce)
}

pub fn open_payload(envelope: &[u8], secret: &[u8]) -> CryptoEnvelopeResult<Vec<u8>> {
    open_payload_with_config(envelope, secret, &CryptoEnvelopeConfig::default())
}

pub fn open_payload_with_config(
    envelope: &[u8],
    secret: &[u8],
    config: &CryptoEnvelopeConfig,
) -> CryptoEnvelopeResult<Vec<u8>> {
    require_secret(secret)?;

    let header = parse_header(envelope)?;
    if header.version != ENVELOPE_VERSION_V1 {
        return Err(CryptoEnvelopeError::UnsupportedVersion(header.version));
    }
    if header.kdf != KDF_ARGON2ID || header.aead != AEAD_XCHACHA20POLY1305 {
        return Err(CryptoEnvelopeError::UnsupportedAlgorithms {
            kdf: header.kdf,
            aead: header.aead,
        });
    }
    if usize::from(header.salt_len) != SALT_LEN {
        return Err(CryptoEnvelopeError::InvalidEnvelope(format!(
            "expected salt length {SALT_LEN}, got {}",
            header.salt_len
        )));
    }
    if usize::from(header.nonce_len) != NONCE_LEN {
        return Err(CryptoEnvelopeError::InvalidEnvelope(format!(
            "expected nonce length {NONCE_LEN}, got {}",
            header.nonce_len
        )));
    }

    let body_len = usize::from(header.salt_len)
        + usize::from(header.nonce_len)
        + usize::try_from(header.ciphertext_len).map_err(|_| {
            CryptoEnvelopeError::InvalidEnvelope(
                "ciphertext length does not fit in usize".to_string(),
            )
        })?;
    let expected_len = HEADER_LEN + body_len;
    if envelope.len() != expected_len {
        return Err(CryptoEnvelopeError::InvalidEnvelope(format!(
            "expected total envelope length {expected_len}, got {}",
            envelope.len()
        )));
    }

    let salt_start = HEADER_LEN;
    let salt_end = salt_start + SALT_LEN;
    let nonce_end = salt_end + NONCE_LEN;

    let mut salt = [0u8; SALT_LEN];
    salt.copy_from_slice(&envelope[salt_start..salt_end]);
    let mut nonce = [0u8; NONCE_LEN];
    nonce.copy_from_slice(&envelope[salt_end..nonce_end]);

    let ciphertext = &envelope[nonce_end..];
    let key = derive_key(secret, &salt, &config.kdf)?;
    let cipher = XChaCha20Poly1305::new(Key::from_slice(&key));
    let plaintext = cipher
        .decrypt(XNonce::from_slice(&nonce), ciphertext)
        .map_err(|_| CryptoEnvelopeError::DecryptFailed)?;

    Ok(plaintext)
}

pub fn inspect_envelope(payload: &[u8]) -> CryptoEnvelopeInspection {
    if payload.len() < ENVELOPE_MAGIC.len() || payload[0..ENVELOPE_MAGIC.len()] != ENVELOPE_MAGIC {
        return CryptoEnvelopeInspection::NotEnvelope;
    }

    let header = match parse_header(payload) {
        Ok(header) => header,
        Err(error) => return CryptoEnvelopeInspection::Invalid(error.to_string()),
    };

    let body_len = usize::from(header.salt_len)
        + usize::from(header.nonce_len)
        + match usize::try_from(header.ciphertext_len) {
            Ok(length) => length,
            Err(_) => {
                return CryptoEnvelopeInspection::Invalid(
                    "ciphertext length does not fit in usize".to_string(),
                );
            }
        };
    let expected_len = HEADER_LEN + body_len;
    if payload.len() != expected_len {
        return CryptoEnvelopeInspection::Invalid(format!(
            "expected total envelope length {expected_len}, got {}",
            payload.len()
        ));
    }

    CryptoEnvelopeInspection::Metadata(CryptoEnvelopeMetadata {
        version: header.version,
        kdf: header.kdf,
        aead: header.aead,
        salt_len: header.salt_len,
        nonce_len: header.nonce_len,
        ciphertext_len: header.ciphertext_len,
        total_len: expected_len,
    })
}

fn kdf_name(id: u8) -> &'static str {
    match id {
        KDF_ARGON2ID => "argon2id",
        _ => "unknown",
    }
}

fn aead_name(id: u8) -> &'static str {
    match id {
        AEAD_XCHACHA20POLY1305 => "xchacha20poly1305",
        _ => "unknown",
    }
}

fn seal_payload_with_material(
    payload: &[u8],
    secret: &[u8],
    config: &CryptoEnvelopeConfig,
    salt: &[u8; SALT_LEN],
    nonce: &[u8; NONCE_LEN],
) -> CryptoEnvelopeResult<Vec<u8>> {
    let key = derive_key(secret, salt, &config.kdf)?;
    let cipher = XChaCha20Poly1305::new(Key::from_slice(&key));
    let ciphertext = cipher
        .encrypt(XNonce::from_slice(nonce), payload)
        .map_err(|_| CryptoEnvelopeError::EncryptFailed)?;
    let ciphertext_len: u32 = ciphertext.len().try_into().map_err(|_| {
        CryptoEnvelopeError::InvalidEnvelope("ciphertext length exceeds u32::MAX".to_string())
    })?;

    let mut envelope = Vec::with_capacity(HEADER_LEN + SALT_LEN + NONCE_LEN + ciphertext.len());
    envelope.extend_from_slice(&ENVELOPE_MAGIC);
    envelope.push(ENVELOPE_VERSION_V1);
    envelope.push(KDF_ARGON2ID);
    envelope.push(AEAD_XCHACHA20POLY1305);
    envelope.push(SALT_LEN as u8);
    envelope.push(NONCE_LEN as u8);
    envelope.extend_from_slice(&ciphertext_len.to_be_bytes());
    envelope.extend_from_slice(salt);
    envelope.extend_from_slice(nonce);
    envelope.extend_from_slice(&ciphertext);

    Ok(envelope)
}

fn parse_header(envelope: &[u8]) -> CryptoEnvelopeResult<EnvelopeHeader> {
    if envelope.len() < HEADER_LEN {
        return Err(CryptoEnvelopeError::InvalidEnvelope(format!(
            "envelope is too short: {} bytes",
            envelope.len()
        )));
    }

    if envelope[0..4] != ENVELOPE_MAGIC {
        return Err(CryptoEnvelopeError::InvalidEnvelope(
            "magic bytes mismatch".to_string(),
        ));
    }

    let ciphertext_len =
        u32::from_be_bytes([envelope[9], envelope[10], envelope[11], envelope[12]]);
    Ok(EnvelopeHeader {
        version: envelope[4],
        kdf: envelope[5],
        aead: envelope[6],
        salt_len: envelope[7],
        nonce_len: envelope[8],
        ciphertext_len,
    })
}

fn derive_key(
    secret: &[u8],
    salt: &[u8; SALT_LEN],
    params: &KeyDerivationParams,
) -> CryptoEnvelopeResult<[u8; KEY_LEN]> {
    let argon_params = Params::new(
        params.memory_kib,
        params.iterations,
        params.parallelism,
        Some(KEY_LEN),
    )
    .map_err(|error| CryptoEnvelopeError::KeyDerivationFailed(error.to_string()))?;
    let mut key = [0u8; KEY_LEN];
    let argon = Argon2::new(Algorithm::Argon2id, Version::V0x13, argon_params);
    argon
        .hash_password_into(secret, salt, &mut key)
        .map_err(|error| CryptoEnvelopeError::KeyDerivationFailed(error.to_string()))?;
    Ok(key)
}

fn fill_random(buf: &mut [u8]) -> CryptoEnvelopeResult<()> {
    getrandom::getrandom(buf)
        .map_err(|error| CryptoEnvelopeError::RandomnessUnavailable(error.to_string()))
}

fn require_secret(secret: &[u8]) -> CryptoEnvelopeResult<()> {
    if secret.is_empty() {
        return Err(CryptoEnvelopeError::SecretRequired);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{
        CryptoEnvelopeConfig, CryptoEnvelopeError, CryptoEnvelopeInspection, NONCE_LEN, SALT_LEN,
        inspect_envelope, open_payload, parse_header, seal_payload, seal_payload_with_material,
    };

    #[test]
    fn seal_and_open_roundtrip_preserves_payload() {
        let payload = b"salam donya";
        let secret = b"correct horse battery staple";

        let envelope = seal_payload(payload, secret).expect("seal should succeed");
        let restored = open_payload(&envelope, secret).expect("open should succeed");

        assert_eq!(restored, payload);
    }

    #[test]
    fn open_fails_when_secret_is_wrong() {
        let payload = b"private payload";
        let envelope = seal_payload(payload, b"right-secret").expect("seal should succeed");

        let result = open_payload(&envelope, b"wrong-secret");
        assert!(matches!(result, Err(CryptoEnvelopeError::DecryptFailed)));
    }

    #[test]
    fn open_fails_for_tampered_ciphertext() {
        let payload = b"sensitive";
        let secret = b"top-secret";
        let mut envelope = seal_payload(payload, secret).expect("seal should succeed");
        let last = envelope.len() - 1;
        envelope[last] ^= 0x01;

        let result = open_payload(&envelope, secret);
        assert!(matches!(result, Err(CryptoEnvelopeError::DecryptFailed)));
    }

    #[test]
    fn open_rejects_unsupported_version() {
        let payload = b"payload";
        let secret = b"secret";
        let mut envelope = seal_payload(payload, secret).expect("seal should succeed");
        envelope[4] = 9;

        let result = open_payload(&envelope, secret);
        assert!(matches!(
            result,
            Err(CryptoEnvelopeError::UnsupportedVersion(9))
        ));
    }

    #[test]
    fn open_rejects_empty_secret() {
        let result = seal_payload(b"payload", b"");
        assert!(matches!(result, Err(CryptoEnvelopeError::SecretRequired)));
    }

    #[test]
    fn envelope_header_is_versioned_and_length_prefixed() {
        let payload = b"abc";
        let secret = b"secret";
        let salt = [7u8; SALT_LEN];
        let nonce = [11u8; NONCE_LEN];
        let config = CryptoEnvelopeConfig::default();
        let envelope = seal_payload_with_material(payload, secret, &config, &salt, &nonce)
            .expect("seal with material should succeed");

        let header = parse_header(&envelope).expect("header should parse");
        assert_eq!(header.version, 1);
        assert_eq!(header.kdf, 1);
        assert_eq!(header.aead, 1);
        assert_eq!(usize::from(header.salt_len), SALT_LEN);
        assert_eq!(usize::from(header.nonce_len), NONCE_LEN);
    }

    #[test]
    fn open_rejects_invalid_magic() {
        let payload = b"payload";
        let secret = b"secret";
        let mut envelope = seal_payload(payload, secret).expect("seal should succeed");
        envelope[0] = b'X';

        let result = open_payload(&envelope, secret);
        assert!(matches!(
            result,
            Err(CryptoEnvelopeError::InvalidEnvelope(_))
        ));
    }

    #[test]
    fn inspect_envelope_reports_metadata_for_valid_envelope() {
        let envelope = seal_payload(b"payload", b"secret").expect("seal should succeed");
        let inspection = inspect_envelope(&envelope);
        match inspection {
            CryptoEnvelopeInspection::Metadata(metadata) => {
                assert_eq!(metadata.version, 1);
                assert_eq!(metadata.kdf_name(), "argon2id");
                assert_eq!(metadata.aead_name(), "xchacha20poly1305");
                assert_eq!(metadata.total_len, envelope.len());
            }
            _ => panic!("expected valid metadata inspection"),
        }
    }

    #[test]
    fn inspect_envelope_reports_not_envelope_for_plain_payload() {
        let inspection = inspect_envelope(b"plain-text");
        assert!(matches!(inspection, CryptoEnvelopeInspection::NotEnvelope));
    }
}
