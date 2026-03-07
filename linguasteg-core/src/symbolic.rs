use crate::{CoreError, CoreResult, SlotId, TemplateId};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SymbolicFieldSpec {
    pub slot: SlotId,
    pub bit_width: u8,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SymbolicFrameSchema {
    pub template_id: TemplateId,
    pub fields: Vec<SymbolicFieldSpec>,
}

impl SymbolicFrameSchema {
    pub fn total_bits(&self) -> usize {
        self.fields
            .iter()
            .map(|field| usize::from(field.bit_width))
            .sum()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SymbolicSlotValue {
    pub slot: SlotId,
    pub bit_width: u8,
    pub value: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BitRange {
    pub start_bit: usize,
    pub consumed_bits: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SymbolicFramePlan {
    pub template_id: TemplateId,
    pub source: BitRange,
    pub values: Vec<SymbolicSlotValue>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SymbolicPayloadPlan {
    pub original_len_bytes: usize,
    pub encoded_len_bytes: usize,
    pub length_prefix_bytes: usize,
    pub padding_bits: u8,
    pub frames: Vec<SymbolicFramePlan>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FixedWidthPlanningOptions {
    pub prepend_u16_be_length: bool,
}

impl Default for FixedWidthPlanningOptions {
    fn default() -> Self {
        Self {
            prepend_u16_be_length: true,
        }
    }
}

pub trait SymbolicPayloadPlanner: Send + Sync {
    fn plan_payload(
        &self,
        payload: &[u8],
        schemas: &[SymbolicFrameSchema],
    ) -> CoreResult<SymbolicPayloadPlan>;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FixedWidthBitPlanner {
    pub options: FixedWidthPlanningOptions,
}

impl Default for FixedWidthBitPlanner {
    fn default() -> Self {
        Self {
            options: FixedWidthPlanningOptions::default(),
        }
    }
}

impl SymbolicPayloadPlanner for FixedWidthBitPlanner {
    fn plan_payload(
        &self,
        payload: &[u8],
        schemas: &[SymbolicFrameSchema],
    ) -> CoreResult<SymbolicPayloadPlan> {
        plan_payload_to_symbolic_frames(payload, schemas, &self.options)
    }
}

pub fn validate_symbolic_frame_schema(schema: &SymbolicFrameSchema) -> CoreResult<()> {
    if schema.fields.is_empty() {
        return Err(CoreError::InvalidSymbolicSchema(format!(
            "schema '{}' has no fields",
            schema.template_id
        )));
    }

    let mut seen_slots: Vec<&SlotId> = Vec::new();
    for field in &schema.fields {
        if field.bit_width == 0 || field.bit_width > 31 {
            return Err(CoreError::InvalidSymbolicSchema(format!(
                "slot '{}' in schema '{}' has invalid bit width {}",
                field.slot, schema.template_id, field.bit_width
            )));
        }

        if seen_slots.contains(&&field.slot) {
            return Err(CoreError::InvalidSymbolicSchema(format!(
                "duplicate symbolic slot '{}' in schema '{}'",
                field.slot, schema.template_id
            )));
        }

        seen_slots.push(&field.slot);
    }

    let total_bits = schema.total_bits();
    if total_bits > usize::from(u8::MAX) {
        return Err(CoreError::InvalidSymbolicSchema(format!(
            "schema '{}' exceeds max frame width of {} bits: {}",
            schema.template_id,
            u8::MAX,
            total_bits
        )));
    }

    Ok(())
}

pub fn plan_payload_to_symbolic_frames(
    payload: &[u8],
    schemas: &[SymbolicFrameSchema],
    options: &FixedWidthPlanningOptions,
) -> CoreResult<SymbolicPayloadPlan> {
    if schemas.is_empty() {
        return Err(CoreError::InvalidSymbolicPlan(
            "at least one symbolic frame schema is required".to_string(),
        ));
    }

    for schema in schemas {
        validate_symbolic_frame_schema(schema)?;
    }

    let encoded_bytes = build_encoded_bytes(payload, options)?;
    let total_bits = encoded_bytes.len() * 8;

    let mut frames = Vec::new();
    let mut bit_cursor = 0usize;
    let mut frame_index = 0usize;

    while bit_cursor < total_bits {
        let schema = &schemas[frame_index % schemas.len()];
        let frame_start_bit = bit_cursor;
        let frame_total_bits = schema.total_bits();

        let values = schema
            .fields
            .iter()
            .scan(bit_cursor, |cursor, field| {
                let value = read_bits_padded(&encoded_bytes, *cursor, usize::from(field.bit_width));
                *cursor += usize::from(field.bit_width);
                Some(SymbolicSlotValue {
                    slot: field.slot.clone(),
                    bit_width: field.bit_width,
                    value,
                })
            })
            .collect();

        bit_cursor += frame_total_bits;
        let consumed_bits = (total_bits.saturating_sub(frame_start_bit)).min(frame_total_bits);

        frames.push(SymbolicFramePlan {
            template_id: schema.template_id.clone(),
            source: BitRange {
                start_bit: frame_start_bit,
                consumed_bits,
            },
            values,
        });

        frame_index += 1;
    }

    let raw_padding_bits = bit_cursor.saturating_sub(total_bits);
    let padding_bits = u8::try_from(raw_padding_bits).map_err(|_| {
        CoreError::InvalidSymbolicPlan(format!(
            "padding bits value {raw_padding_bits} exceeds u8::MAX"
        ))
    })?;

    Ok(SymbolicPayloadPlan {
        original_len_bytes: payload.len(),
        encoded_len_bytes: encoded_bytes.len(),
        length_prefix_bytes: if options.prepend_u16_be_length { 2 } else { 0 },
        padding_bits,
        frames,
    })
}

pub fn decode_payload_from_symbolic_frames(
    frames: &[SymbolicFramePlan],
    schemas: &[SymbolicFrameSchema],
    options: &FixedWidthPlanningOptions,
) -> CoreResult<Vec<u8>> {
    if schemas.is_empty() {
        return Err(CoreError::InvalidSymbolicPlan(
            "at least one symbolic frame schema is required".to_string(),
        ));
    }

    for schema in schemas {
        validate_symbolic_frame_schema(schema)?;
    }

    if frames.is_empty() {
        return if options.prepend_u16_be_length {
            Err(CoreError::InvalidSymbolicPlan(
                "cannot decode empty symbolic frame list with length prefix enabled".to_string(),
            ))
        } else {
            Ok(Vec::new())
        };
    }

    let mut encoded_bytes = Vec::new();
    let mut bit_cursor = 0usize;

    for (frame_index, frame) in frames.iter().enumerate() {
        let expected_schema = &schemas[frame_index % schemas.len()];
        validate_frame_against_schema(frame, expected_schema)?;

        for field_value in &frame.values {
            write_bits_msb(
                &mut encoded_bytes,
                &mut bit_cursor,
                field_value.value,
                field_value.bit_width,
            );
        }
    }

    if options.prepend_u16_be_length {
        decode_with_u16_length_prefix(&encoded_bytes, bit_cursor)
    } else {
        if bit_cursor % 8 != 0 {
            return Err(CoreError::InvalidSymbolicPlan(
                "decoded bitstream is not byte-aligned without length prefix".to_string(),
            ));
        }

        Ok(encoded_bytes)
    }
}

fn build_encoded_bytes(payload: &[u8], options: &FixedWidthPlanningOptions) -> CoreResult<Vec<u8>> {
    if options.prepend_u16_be_length {
        let payload_len: u16 = payload.len().try_into().map_err(|_| {
            CoreError::InvalidSymbolicPlan("payload is too large for u16 length prefix".to_string())
        })?;

        let mut bytes = Vec::with_capacity(payload.len() + 2);
        bytes.extend_from_slice(&payload_len.to_be_bytes());
        bytes.extend_from_slice(payload);
        Ok(bytes)
    } else {
        Ok(payload.to_vec())
    }
}

fn read_bits_padded(bytes: &[u8], start_bit: usize, bit_width: usize) -> u32 {
    let mut value = 0u32;
    for offset in 0..bit_width {
        let bit_index = start_bit + offset;
        let bit = if bit_index < bytes.len() * 8 {
            let byte = bytes[bit_index / 8];
            let shift = 7 - (bit_index % 8);
            u32::from((byte >> shift) & 1)
        } else {
            0
        };
        value = (value << 1) | bit;
    }
    value
}

fn validate_frame_against_schema(
    frame: &SymbolicFramePlan,
    schema: &SymbolicFrameSchema,
) -> CoreResult<()> {
    if frame.template_id != schema.template_id {
        return Err(CoreError::InvalidSymbolicPlan(format!(
            "frame template '{}' does not match expected schema '{}'",
            frame.template_id, schema.template_id
        )));
    }

    if frame.values.len() != schema.fields.len() {
        return Err(CoreError::InvalidSymbolicPlan(format!(
            "frame '{}' expected {} symbolic fields but got {}",
            frame.template_id,
            schema.fields.len(),
            frame.values.len()
        )));
    }

    for (field_value, expected_field) in frame.values.iter().zip(schema.fields.iter()) {
        if field_value.slot != expected_field.slot {
            return Err(CoreError::InvalidSymbolicPlan(format!(
                "frame '{}' slot mismatch: expected '{}' but got '{}'",
                frame.template_id, expected_field.slot, field_value.slot
            )));
        }

        if field_value.bit_width != expected_field.bit_width {
            return Err(CoreError::InvalidSymbolicPlan(format!(
                "frame '{}' bit width mismatch for slot '{}': expected {} but got {}",
                frame.template_id,
                expected_field.slot,
                expected_field.bit_width,
                field_value.bit_width
            )));
        }

        if !fits_bit_width(field_value.value, field_value.bit_width) {
            return Err(CoreError::InvalidSymbolicPlan(format!(
                "value {} does not fit bit width {} for slot '{}'",
                field_value.value, field_value.bit_width, field_value.slot
            )));
        }
    }

    Ok(())
}

fn fits_bit_width(value: u32, bit_width: u8) -> bool {
    let max_exclusive = 1u64 << bit_width;
    u64::from(value) < max_exclusive
}

fn write_bits_msb(output: &mut Vec<u8>, bit_cursor: &mut usize, value: u32, bit_width: u8) {
    for shift in (0..usize::from(bit_width)).rev() {
        let bit = ((value >> shift) & 1) as u8;
        let byte_index = *bit_cursor / 8;
        if byte_index == output.len() {
            output.push(0);
        }

        let bit_offset = *bit_cursor % 8;
        let byte_shift = 7 - bit_offset;
        output[byte_index] |= bit << byte_shift;
        *bit_cursor += 1;
    }
}

fn decode_with_u16_length_prefix(encoded_bytes: &[u8], bit_cursor: usize) -> CoreResult<Vec<u8>> {
    if bit_cursor < 16 {
        return Err(CoreError::InvalidSymbolicPlan(
            "decoded stream is too short for u16 length prefix".to_string(),
        ));
    }

    if encoded_bytes.len() < 2 {
        return Err(CoreError::InvalidSymbolicPlan(
            "decoded stream is too short for u16 length prefix bytes".to_string(),
        ));
    }

    let payload_len = usize::from(u16::from_be_bytes([encoded_bytes[0], encoded_bytes[1]]));
    let required_bits = (payload_len + 2) * 8;
    if bit_cursor < required_bits {
        return Err(CoreError::InvalidSymbolicPlan(format!(
            "decoded stream is too short for declared payload length: need {required_bits} bits, got {bit_cursor}"
        )));
    }

    if encoded_bytes.len() < payload_len + 2 {
        return Err(CoreError::InvalidSymbolicPlan(
            "decoded stream does not contain enough bytes for declared payload length".to_string(),
        ));
    }

    Ok(encoded_bytes[2..2 + payload_len].to_vec())
}

#[cfg(test)]
mod tests {
    use super::{
        FixedWidthBitPlanner, FixedWidthPlanningOptions, SymbolicFieldSpec, SymbolicFrameSchema,
        SymbolicPayloadPlanner, decode_payload_from_symbolic_frames,
        plan_payload_to_symbolic_frames, validate_symbolic_frame_schema,
    };
    use crate::{SlotId, TemplateId};

    #[test]
    fn schema_validation_rejects_duplicate_slots() {
        let schema = SymbolicFrameSchema {
            template_id: TemplateId::new("fa-demo").expect("valid template id"),
            fields: vec![
                SymbolicFieldSpec {
                    slot: SlotId::new("subject").expect("valid slot"),
                    bit_width: 5,
                },
                SymbolicFieldSpec {
                    slot: SlotId::new("subject").expect("valid slot"),
                    bit_width: 3,
                },
            ],
        };

        let error = validate_symbolic_frame_schema(&schema).expect_err("schema should fail");
        assert!(error.to_string().contains("duplicate symbolic slot"));
    }

    #[test]
    fn schema_validation_rejects_total_bit_width_above_u8_max() {
        let schema = SymbolicFrameSchema {
            template_id: TemplateId::new("fa-wide").expect("valid template id"),
            fields: vec![
                field("s1", 31),
                field("s2", 31),
                field("s3", 31),
                field("s4", 31),
                field("s5", 31),
                field("s6", 31),
                field("s7", 31),
                field("s8", 31),
                field("s9", 31),
            ],
        };

        let error = validate_symbolic_frame_schema(&schema).expect_err("schema should fail");
        assert!(error.to_string().contains("exceeds max frame width"));
    }

    #[test]
    fn planner_extracts_bits_in_msb_order_without_length_prefix() {
        let schema = SymbolicFrameSchema {
            template_id: TemplateId::new("fa-demo").expect("valid template id"),
            fields: vec![field("subject", 3), field("object", 3), field("verb", 2)],
        };

        let plan = plan_payload_to_symbolic_frames(
            &[0b1011_0011],
            &[schema],
            &FixedWidthPlanningOptions {
                prepend_u16_be_length: false,
            },
        )
        .expect("planning should succeed");

        assert_eq!(plan.frames.len(), 1);
        let values: Vec<u32> = plan.frames[0]
            .values
            .iter()
            .map(|item| item.value)
            .collect();
        assert_eq!(values, vec![5, 4, 3]);
        assert_eq!(plan.padding_bits, 0);
    }

    #[test]
    fn planner_pads_last_frame_when_bits_do_not_fill_schema() {
        let schema = SymbolicFrameSchema {
            template_id: TemplateId::new("fa-demo").expect("valid template id"),
            fields: vec![field("subject", 5), field("verb", 5)],
        };

        let plan = plan_payload_to_symbolic_frames(
            &[0b1111_0000],
            &[schema],
            &FixedWidthPlanningOptions {
                prepend_u16_be_length: false,
            },
        )
        .expect("planning should succeed");

        assert_eq!(plan.frames.len(), 1);
        let values: Vec<u32> = plan.frames[0]
            .values
            .iter()
            .map(|item| item.value)
            .collect();
        assert_eq!(values, vec![30, 0]);
        assert_eq!(plan.padding_bits, 2);
        assert_eq!(plan.frames[0].source.consumed_bits, 8);
    }

    #[test]
    fn planner_supports_rotating_schemas_and_length_prefix() {
        let planner = FixedWidthBitPlanner::default();
        let schemas = vec![
            SymbolicFrameSchema {
                template_id: TemplateId::new("fa-a").expect("valid template id"),
                fields: vec![field("a", 8)],
            },
            SymbolicFrameSchema {
                template_id: TemplateId::new("fa-b").expect("valid template id"),
                fields: vec![field("b", 8)],
            },
        ];

        let plan = planner
            .plan_payload(&[0x12, 0x34], &schemas)
            .expect("planning should succeed");

        assert_eq!(plan.length_prefix_bytes, 2);
        assert_eq!(plan.encoded_len_bytes, 4);
        assert_eq!(plan.frames.len(), 4);
        assert_eq!(plan.frames[0].template_id.as_str(), "fa-a");
        assert_eq!(plan.frames[1].template_id.as_str(), "fa-b");
        assert_eq!(plan.frames[2].template_id.as_str(), "fa-a");
        assert_eq!(plan.frames[3].template_id.as_str(), "fa-b");

        let values: Vec<u32> = plan
            .frames
            .iter()
            .map(|frame| frame.values[0].value)
            .collect();
        assert_eq!(values, vec![0x00, 0x02, 0x12, 0x34]);
    }

    #[test]
    fn decoder_roundtrips_payload_with_length_prefix() {
        let planner = FixedWidthBitPlanner::default();
        let schemas = vec![
            SymbolicFrameSchema {
                template_id: TemplateId::new("fa-a").expect("valid template id"),
                fields: vec![field("a", 8)],
            },
            SymbolicFrameSchema {
                template_id: TemplateId::new("fa-b").expect("valid template id"),
                fields: vec![field("b", 8)],
            },
        ];
        let payload = vec![0xAB, 0xCD, 0xEF];

        let plan = planner
            .plan_payload(&payload, &schemas)
            .expect("planning should succeed");
        let decoded = decode_payload_from_symbolic_frames(
            &plan.frames,
            &schemas,
            &FixedWidthPlanningOptions {
                prepend_u16_be_length: true,
            },
        )
        .expect("decoding should succeed");

        assert_eq!(decoded, payload);
    }

    #[test]
    fn decoder_rejects_frame_slot_mismatch() {
        let schemas = vec![SymbolicFrameSchema {
            template_id: TemplateId::new("fa-a").expect("valid template id"),
            fields: vec![field("expected", 5)],
        }];
        let invalid_frame = crate::SymbolicFramePlan {
            template_id: TemplateId::new("fa-a").expect("valid template id"),
            source: crate::BitRange {
                start_bit: 0,
                consumed_bits: 5,
            },
            values: vec![crate::SymbolicSlotValue {
                slot: SlotId::new("actual").expect("valid slot"),
                bit_width: 5,
                value: 3,
            }],
        };

        let error = decode_payload_from_symbolic_frames(
            &[invalid_frame],
            &schemas,
            &FixedWidthPlanningOptions {
                prepend_u16_be_length: false,
            },
        )
        .expect_err("decoding should fail");

        assert!(error.to_string().contains("slot mismatch"));
    }

    #[test]
    fn decoder_rejects_declared_length_longer_than_stream() {
        let schemas = vec![SymbolicFrameSchema {
            template_id: TemplateId::new("fa-a").expect("valid template id"),
            fields: vec![field("a", 8)],
        }];

        let frames = vec![crate::SymbolicFramePlan {
            template_id: TemplateId::new("fa-a").expect("valid template id"),
            source: crate::BitRange {
                start_bit: 0,
                consumed_bits: 8,
            },
            values: vec![crate::SymbolicSlotValue {
                slot: SlotId::new("a").expect("valid slot"),
                bit_width: 8,
                value: 1,
            }],
        }];

        let error = decode_payload_from_symbolic_frames(
            &frames,
            &schemas,
            &FixedWidthPlanningOptions {
                prepend_u16_be_length: true,
            },
        )
        .expect_err("decoding should fail");

        assert!(error.to_string().contains("too short"));
    }

    fn field(slot: &str, bit_width: u8) -> SymbolicFieldSpec {
        SymbolicFieldSpec {
            slot: SlotId::new(slot).expect("valid slot"),
            bit_width,
        }
    }
}
