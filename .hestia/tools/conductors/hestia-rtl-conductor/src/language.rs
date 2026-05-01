//! HDL language types and detection

/// Re-export HdlLanguage from adapter_core
pub use adapter_core::HdlLanguage as HdlLanguage;

/// HDL language detector
#[derive(Debug, Default)]
pub struct LanguageDetector {
    /// Known file extensions mapped to languages
    extensions: Vec<(String, HdlLanguage)>,
}

impl LanguageDetector {
    /// Create a new language detector with default mappings
    pub fn new() -> Self {
        let extensions = vec![
            (".sv".to_string(), HdlLanguage::SystemVerilog),
            (".v".to_string(), HdlLanguage::Verilog),
            (".vhdl".to_string(), HdlLanguage::Vhdl),
            (".vhd".to_string(), HdlLanguage::Vhdl),
            (".scala".to_string(), HdlLanguage::Chisel),
            (".spinal".to_string(), HdlLanguage::SpinalHdl),
            (".py".to_string(), HdlLanguage::Amaranth),
            (".myhdl".to_string(), HdlLanguage::MyHdl),
        ];
        Self { extensions }
    }

    /// Detect HDL language from file extension
    pub fn detect_from_extension(&self, path: &std::path::Path) -> Option<HdlLanguage> {
        let ext = path.extension()?.to_str()?.to_string();
        let ext_with_dot = format!(".{ext}");
        self.extensions
            .iter()
            .find(|(e, _)| *e == ext_with_dot)
            .map(|(_, lang)| *lang)
    }

    /// Detect HDL language from language name string
    pub fn detect_from_name(&self, name: &str) -> Option<HdlLanguage> {
        match name.to_lowercase().as_str() {
            "systemverilog" | "sv" => Some(HdlLanguage::SystemVerilog),
            "verilog" | "v" => Some(HdlLanguage::Verilog),
            "vhdl" => Some(HdlLanguage::Vhdl),
            "chisel" => Some(HdlLanguage::Chisel),
            "spinalhdl" | "spinal" => Some(HdlLanguage::SpinalHdl),
            "amaranth" => Some(HdlLanguage::Amaranth),
            "myhdl" => Some(HdlLanguage::MyHdl),
            _ => None,
        }
    }
}