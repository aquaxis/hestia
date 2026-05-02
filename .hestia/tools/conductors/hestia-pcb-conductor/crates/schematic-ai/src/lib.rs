//! schematic-ai -- AI schematic engine: CoT prompt, requirements parser

pub mod cot_prompt;
pub mod requirements_parser;

use serde::{Deserialize, Serialize};

/// Schematic generation request.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchematicRequest {
    /// Natural language description of the desired circuit.
    pub description: String,
    /// Component constraints (e.g., voltage range, package type).
    #[serde(default)]
    pub constraints: Vec<String>,
    /// Target application domain.
    #[serde(default)]
    pub domain: Option<String>,
}

/// Schematic generation result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchematicResult {
    /// Generated schematic in KiCad format.
    pub schematic_sexpr: String,
    /// Component list used.
    pub components: Vec<ComponentRef>,
    /// Confidence score (0.0 - 1.0).
    pub confidence: f64,
}

/// Reference to a component in the generated schematic.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentRef {
    pub reference: String,
    pub value: String,
    pub footprint: String,
    pub datasheet_url: Option<String>,
}

/// AI schematic engine that uses chain-of-thought prompting.
#[allow(dead_code)]
pub struct SchematicAiEngine {
    model_config: ModelConfig,
}

/// Configuration for the AI model used in schematic generation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelConfig {
    pub model_name: String,
    pub temperature: f64,
    pub max_tokens: u32,
}

impl Default for ModelConfig {
    fn default() -> Self {
        Self {
            model_name: "claude-3-opus".to_string(),
            temperature: 0.2,
            max_tokens: 4096,
        }
    }
}

impl SchematicAiEngine {
    /// Create a new AI schematic engine.
    pub fn new(config: ModelConfig) -> Self {
        Self { model_config: config }
    }

    /// Generate a schematic from a natural language request.
    ///
    /// Parses the description into structured requirements, then produces a
    /// KiCad S-expression schematic containing the identified components.
    pub fn generate(&self, request: &SchematicRequest) -> Result<SchematicResult, Box<dyn std::error::Error>> {
        tracing::info!(description = %request.description, "AI: generating schematic");

        let reqs = requirements_parser::parse_requirements(&request.description);

        let mut components = Vec::new();
        let mut sexpr_components = Vec::new();
        let mut net_items = Vec::new();
        let mut ref_counter: u32 = 0;
        let mut y_offset: i64 = 0;

        // Generate components from functional blocks
        for block in &reqs.functional_blocks {
            ref_counter += 1;
            let (ref_prefix, value, footprint) = match block.block_type {
                requirements_parser::BlockType::Power => {
                    ("U", "LM7805", "Package_TO_SOT_THT:TO-220-3_Vertical")
                }
                requirements_parser::BlockType::Processing => {
                    ("U", "STM32F103C8T6", "Package_QFP:LQFP-48_7x7mm_P0.5mm")
                }
                requirements_parser::BlockType::Memory => {
                    ("U", "W25Q32", "Package_SO:SOIC-8_3.9x4.9mm_P1.27mm")
                }
                requirements_parser::BlockType::Communication => {
                    ("J", "Conn_01x04", "Connector_PinHeader_2.54mm:PinHeader_1x04_P2.54mm_Vertical")
                }
                requirements_parser::BlockType::Analog => {
                    if block.name.contains("Resistor") {
                        ("R", "10k", "Resistor_SMD:R_0402_1005Metric")
                    } else {
                        ("C", "100nF", "Capacitor_SMD:C_0402_1005Metric")
                    }
                }
                requirements_parser::BlockType::Sensor => {
                    ("U", "BME280", "Package_LGA:BME280")
                }
                requirements_parser::BlockType::Actuator => {
                    ("D", "LED", "LED_SMD:LED_0805_2012Metric")
                }
            };

            let reference = format!("{}{}", ref_prefix, ref_counter);

            // Build S-expression for this component
            let comp_sexpr = format!(
                "  (component \"{}\" \n    (ref {})\n    (value \"{}\")\n    (footprint \"{}\")\n    (at 50 {})\n  )",
                value, reference, value, footprint, y_offset
            );
            sexpr_components.push(comp_sexpr);

            components.push(ComponentRef {
                reference: reference.clone(),
                value: value.to_string(),
                footprint: footprint.to_string(),
                datasheet_url: None,
            });

            y_offset += 2000; // 2mm spacing in KiCad mils (0.01mm units -> 2000 = 20mm)
        }

        // Add power nets from voltage constraints
        let mut net_id: u32 = 0;
        for constraint in &reqs.electrical_constraints {
            if constraint.parameter == "voltage" {
                net_id += 1;
                let net_name = format!("VCC_{}", constraint.value.replace('.', "p"));
                net_items.push(format!(
                    "  (net {} \"{}\")",
                    net_id, net_name
                ));
            }
        }

        // Add interface nets
        for iface in &reqs.interfaces {
            net_id += 1;
            let net_name = format!("NET_{}", iface.protocol);
            net_items.push(format!(
                "  (net {} \"{}\")",
                net_id, net_name
            ));
        }

        // Ensure at least GND net exists
        if !reqs.functional_blocks.is_empty() {
            net_id += 1;
            net_items.push(format!("  (net {} \"GND\")", net_id));
        }

        // Assemble the full KiCad S-expression schematic
        let components_section = if sexpr_components.is_empty() {
            String::new()
        } else {
            format!(" (components\n{}\n )", sexpr_components.join("\n"))
        };

        let nets_section = if net_items.is_empty() {
            String::new()
        } else {
            format!(" (nets\n{}\n )", net_items.join("\n"))
        };

        let schematic_sexpr = format!(
            "(kicad_sch\n (version 20230121)\n (generator \"hestia-schematic-ai\")\n{}\n{}\n)",
            components_section, nets_section
        );

        // Confidence heuristic: more extracted items -> higher confidence
        let total_items = reqs.functional_blocks.len()
            + reqs.electrical_constraints.len()
            + reqs.interfaces.len();
        let confidence = if total_items == 0 {
            0.1
        } else {
            (0.3 + 0.1 * total_items as f64).min(0.95)
        };

        tracing::info!(
            components = components.len(),
            nets = net_items.len(),
            confidence = confidence,
            "schematic generation complete"
        );

        Ok(SchematicResult {
            schematic_sexpr,
            components,
            confidence,
        })
    }
}