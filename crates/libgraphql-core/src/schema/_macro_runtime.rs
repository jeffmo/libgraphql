use crate::schema::Schema;

pub fn build_from_macro_serialized(serialized_schema: &[u8]) -> Schema {
    bincode::serde::decode_from_slice::<Schema, _>(
        serialized_schema,
        bincode::config::standard()
    ).expect("Failed to deserialize precompiled Schema").0
}
