use std::collections::HashMap;

use crate::types::{HdlLanguage, LspServerConfig};

#[derive(Debug, Clone)]
pub struct RoutingTable {
    extension_map: HashMap<String, HdlLanguage>,
}

impl RoutingTable {
    pub fn new() -> Self {
        let mut extension_map = HashMap::new();
        extension_map.insert(".v".to_string(), HdlLanguage::Verilog);
        extension_map.insert(".sv".to_string(), HdlLanguage::SystemVerilog);
        extension_map.insert(".svh".to_string(), HdlLanguage::SystemVerilog);
        extension_map.insert(".vhd".to_string(), HdlLanguage::Vhdl);
        extension_map.insert(".vhdl".to_string(), HdlLanguage::Vhdl);
        extension_map.insert(".va".to_string(), HdlLanguage::VerilogAms);
        extension_map.insert(".vams".to_string(), HdlLanguage::VerilogAms);
        Self { extension_map }
    }

    pub fn resolve(&self, path: &str) -> Option<&HdlLanguage> {
        std::path::Path::new(path)
            .extension()
            .and_then(|ext| {
                let dot_ext = format!(".{}", ext.to_string_lossy());
                self.extension_map.get(&dot_ext)
            })
            .or_else(|| {
                let path_lower = path.to_lowercase();
                for (ext, lang) in &self.extension_map {
                    if path_lower.ends_with(ext) {
                        return Some(lang);
                    }
                }
                None
            })
    }

    pub fn register(&mut self, extension: String, language: HdlLanguage) {
        let dot_ext = if extension.starts_with('.') {
            extension
        } else {
            format!(".{}", extension)
        };
        self.extension_map.insert(dot_ext, language);
    }
}

impl Default for RoutingTable {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug)]
pub struct LspBroker {
    pub routing_table: RoutingTable,
    servers: HashMap<HdlLanguage, LspServerConfig>,
}

impl LspBroker {
    pub fn new() -> Self {
        let mut servers = HashMap::new();
        servers.insert(HdlLanguage::Verilog, LspServerConfig::svls());
        servers.insert(HdlLanguage::SystemVerilog, LspServerConfig::svls());
        servers.insert(HdlLanguage::Vhdl, LspServerConfig::vhdl_ls());
        servers.insert(HdlLanguage::VerilogAms, LspServerConfig::verilog_ams_ls());
        Self {
            routing_table: RoutingTable::new(),
            servers,
        }
    }

    pub fn start_server(&mut self, config: LspServerConfig) {
        self.servers.insert(config.language.clone(), config);
    }

    pub fn shutdown_server(&mut self, language: &HdlLanguage) -> Option<LspServerConfig> {
        self.servers.remove(language)
    }

    pub fn get_config(&self, language: &HdlLanguage) -> Option<&LspServerConfig> {
        self.servers.get(language)
    }

    pub fn resolve_and_get_config(&self, path: &str) -> Option<(&HdlLanguage, &LspServerConfig)> {
        self.routing_table
            .resolve(path)
            .and_then(|lang| self.servers.get(lang).map(|cfg| (lang, cfg)))
    }
}

impl Default for LspBroker {
    fn default() -> Self {
        Self::new()
    }
}