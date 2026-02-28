use crate::schema::Schema;

#[derive(Debug, thiserror::Error)]
pub enum ViewError {
    #[error("evt_hash mismatch")]
    EvtHashMismatch { expected: [u8; 32], got: [u8; 32] },

    #[error("field count mismatch: expected {expected}, got {got}")]
    FieldCountMismatch { expected: usize, got: usize },

    #[error("field name mismatch at index {index}: expected `{expected}`, got `{got}`")]
    FieldNameMismatch {
        index: usize,
        expected: &'static str,
        got: String,
    },

    #[error("field `{field}` has incompatible type")]
    TypeMismatch { field: &'static str },

    #[error("field `{field}` is missing but not nullable")]
    MissingNotAllowed { field: &'static str },

    #[error("view error: {msg}")]
    Custom { msg: String },
}

impl ViewError {
    pub fn custom(msg: impl Into<String>) -> Self {
        ViewError::Custom { msg: msg.into() }
    }

    pub fn with_schema(self, _schema: &Schema) -> Self {
        // Placeholder for future enrichment.
        self
    }
}
