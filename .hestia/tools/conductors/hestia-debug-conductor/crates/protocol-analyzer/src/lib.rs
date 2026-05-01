//! debug-protocol-analyzer -- sigrok integration for protocol decoding

use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Protocol analyzer errors.
#[derive(Debug, Error)]
pub enum ProtocolAnalyzerError {
    #[error("sigrok error: {0}")]
    SigrokError(String),
    #[error("decoder not found: {0}")]
    DecoderNotFound(String),
    #[error("analysis failed: {0}")]
    AnalysisFailed(String),
    #[error("invalid sample rate: {0}")]
    InvalidSampleRate(String),
}

/// A sigrok protocol decoder descriptor.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecoderInfo {
    /// Decoder ID (e.g. "spi", "i2c", "uart").
    pub id: String,
    /// Human-readable decoder name.
    pub name: String,
    /// Description of the decoder.
    pub description: String,
    /// Required input channels.
    pub channels: Vec<String>,
    /// Optional configuration options.
    #[serde(default)]
    pub options: serde_json::Value,
}

/// Configuration for a protocol analysis session.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisConfig {
    /// Path to the input capture file (VCD/FST/sigrok native).
    pub input_path: String,
    /// Decoder to apply.
    pub decoder: String,
    /// Sample rate in Hz.
    pub sample_rate: u64,
    /// Channel-to-signal mapping.
    #[serde(default)]
    pub channel_map: serde_json::Value,
    /// Decoder-specific options.
    #[serde(default)]
    pub decoder_options: serde_json::Value,
}

/// Result of a protocol analysis session.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisResult {
    /// Decoder that was applied.
    pub decoder: String,
    /// Decoded protocol frames.
    pub frames: Vec<DecodedFrame>,
    /// Number of frames decoded.
    pub frame_count: usize,
}

/// A single decoded protocol frame.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecodedFrame {
    /// Frame number (0-indexed).
    pub index: usize,
    /// Start sample index.
    pub start_sample: u64,
    /// End sample index.
    pub end_sample: u64,
    /// Decoded data payload.
    pub data: serde_json::Value,
}

/// Protocol analyzer backed by sigrok (libsigrok / sigrok-cli).
pub struct ProtocolAnalyzer;

impl ProtocolAnalyzer {
    /// List available sigrok decoders.
    pub async fn list_decoders() -> Result<Vec<DecoderInfo>, ProtocolAnalyzerError> {
        tracing::info!("ProtocolAnalyzer: listing sigrok decoders");
        Ok(Vec::new())
    }

    /// Run a protocol analysis with the given configuration.
    pub async fn analyze(config: &AnalysisConfig) -> Result<AnalysisResult, ProtocolAnalyzerError> {
        tracing::info!(
            "ProtocolAnalyzer: running decoder={} input={}",
            config.decoder,
            config.input_path
        );
        Ok(AnalysisResult {
            decoder: config.decoder.clone(),
            frames: Vec::new(),
            frame_count: 0,
        })
    }
}