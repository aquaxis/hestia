use crate::error::ConstraintFormat;
use crate::error::ConstraintError;
use crate::types::ConstraintModel;

pub fn parse_ucf(_input: &str) -> Result<ConstraintModel, ConstraintError> {
    Ok(ConstraintModel {
        constraints: Vec::new(),
        source_format: ConstraintFormat::Ucf,
    })
}