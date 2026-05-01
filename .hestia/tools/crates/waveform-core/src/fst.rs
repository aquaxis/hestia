use crate::error::WaveformError;
use crate::types::{WaveformData, WaveformFormat, WaveformHeader};

pub fn parse_fst(_input: &[u8]) -> Result<WaveformData, WaveformError> {
    let header = WaveformHeader {
        version: None,
        date: None,
        timescale: None,
    };
    Ok(WaveformData {
        header,
        signals: Vec::new(),
        format: WaveformFormat::Fst,
    })
}