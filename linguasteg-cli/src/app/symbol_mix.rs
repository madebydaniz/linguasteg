use linguasteg_core::SymbolicFramePlan;

const SYMBOL_MIX_MAGIC: u64 = 0x6c73_7465_675f_6d78;
const PRESERVED_PREFIX_BITS: usize = 120;

pub(crate) fn apply_secret_symbolic_mix(frames: &mut [SymbolicFramePlan], secret: &[u8]) {
    if secret.is_empty() {
        return;
    }

    let seed = fnv1a64(secret) ^ SYMBOL_MIX_MAGIC;
    for (frame_index, frame) in frames.iter_mut().enumerate() {
        let template_mix = fnv1a64(frame.template_id.as_str().as_bytes()).rotate_left(13);
        let mut slot_start_bit = frame.source.start_bit;
        for (value_index, value) in frame.values.iter_mut().enumerate() {
            let bit_width = u64::from(value.bit_width);
            let max_value = 1u64 << bit_width;
            if max_value <= 1 {
                slot_start_bit += usize::from(value.bit_width);
                continue;
            }
            if slot_start_bit < PRESERVED_PREFIX_BITS {
                slot_start_bit += usize::from(value.bit_width);
                continue;
            }

            let slot_mix = fnv1a64(value.slot.as_str().as_bytes()).rotate_left(29);
            let index_mix = (frame_index as u64)
                .wrapping_mul(0x9E37_79B9_7F4A_7C15)
                .wrapping_add((value_index as u64).wrapping_mul(0xD1B5_4A32_D192_ED03));
            let mask = max_value - 1;
            let mut word = seed ^ template_mix ^ slot_mix ^ index_mix;
            word ^= word.rotate_left(17);
            word ^= word.rotate_right(11);

            let offset = (word % mask) + 1;
            let mixed = (u64::from(value.value) ^ offset) & mask;
            value.value = mixed as u32;
            slot_start_bit += usize::from(value.bit_width);
        }
    }
}

fn fnv1a64(bytes: &[u8]) -> u64 {
    let mut hash = 0xcbf2_9ce4_8422_2325u64;
    for &byte in bytes {
        hash ^= u64::from(byte);
        hash = hash.wrapping_mul(0x0000_0001_0000_01b3);
    }
    hash
}
