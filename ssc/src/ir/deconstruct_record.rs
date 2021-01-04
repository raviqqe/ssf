use super::expression::Expression;
use std::sync::Arc;

#[derive(Clone, Debug, PartialEq)]
pub struct DeconstructRecord {
    record: Arc<Expression>,
    field_index: usize,
    name: String,
}

impl DeconstructRecord {
    pub fn new(record: impl Into<Expression>, field_index: usize, name: impl Into<String>) -> Self {
        Self {
            record: record.into().into(),
            field_index,
            name: name.into(),
        }
    }

    pub fn record(&self) -> &Expression {
        &self.record
    }

    pub fn field_index(&self) -> usize {
        self.field_index
    }

    pub fn name(&self) -> &str {
        &self.name
    }
}
