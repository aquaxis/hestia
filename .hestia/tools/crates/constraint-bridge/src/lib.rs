pub mod error;
pub mod generators;
pub mod parsers;
pub mod types;

use error::ConstraintError;
use error::ConstraintFormat;
use types::ConstraintModel;

pub fn convert(
    input: &str,
    from: ConstraintFormat,
    to: ConstraintFormat,
) -> Result<String, ConstraintError> {
    let model = parse(input, from)?;
    generate(&model, to)
}

pub fn parse(
    input: &str,
    format: ConstraintFormat,
) -> Result<ConstraintModel, ConstraintError> {
    match format {
        ConstraintFormat::Xdc => parsers::xdc::parse_xdc(input),
        ConstraintFormat::Pcf => parsers::pcf::parse_pcf(input),
        ConstraintFormat::Sdc => parsers::sdc::parse_sdc(input),
        ConstraintFormat::EfinityXml => parsers::efinity::parse_efinity(input),
        ConstraintFormat::Qsf => parsers::qsf::parse_qsf(input),
        ConstraintFormat::Ucf => parsers::ucf::parse_ucf(input),
    }
}

pub fn generate(
    model: &ConstraintModel,
    format: ConstraintFormat,
) -> Result<String, ConstraintError> {
    match format {
        ConstraintFormat::Xdc => generators::xdc::generate_xdc(model),
        ConstraintFormat::Pcf => generators::pcf::generate_pcf(model),
        ConstraintFormat::Sdc => generators::sdc::generate_sdc(model),
        ConstraintFormat::EfinityXml => generators::efinity::generate_efinity(model),
        ConstraintFormat::Qsf => generators::qsf::generate_qsf(model),
        ConstraintFormat::Ucf => generators::ucf::generate_ucf(model),
    }
}