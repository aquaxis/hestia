//! rag-ingest -- PDF 7-stage / Web 8-stage ingestion pipelines

pub mod pipeline;

pub use pipeline::{IngestPipeline, IngestResult, PdfPipeline, WebPipeline};