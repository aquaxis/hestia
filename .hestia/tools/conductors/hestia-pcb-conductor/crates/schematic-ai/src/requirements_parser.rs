//! Natural language requirements parser for PCB design

use serde::{Deserialize, Serialize};

/// Parsed requirements from a natural language description.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParsedRequirements {
    /// Functional blocks identified.
    pub functional_blocks: Vec<FunctionalBlock>,
    /// Electrical constraints.
    pub electrical_constraints: Vec<ElectricalConstraint>,
    /// Interface requirements.
    pub interfaces: Vec<InterfaceRequirement>,
    /// Package preferences.
    pub package_preferences: Vec<String>,
}

/// A functional block in the design.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionalBlock {
    pub name: String,
    pub block_type: BlockType,
    pub description: String,
}

/// Type of functional block.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BlockType {
    Power,
    Processing,
    Memory,
    Communication,
    Analog,
    Sensor,
    Actuator,
}

/// An electrical constraint.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ElectricalConstraint {
    pub parameter: String,
    pub value: String,
    pub unit: String,
}

/// An interface requirement (e.g., I2C, SPI, UART).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InterfaceRequirement {
    pub protocol: String,
    pub role: String,
    pub speed_mbps: Option<f64>,
}

/// Parse a natural language description into structured requirements.
pub fn parse_requirements(description: &str) -> ParsedRequirements {
    tracing::info!(desc_len = description.len(), "parsing requirements from description");

    let lower = description.to_ascii_lowercase();
    let mut functional_blocks = Vec::new();
    let mut electrical_constraints = Vec::new();
    let mut interfaces = Vec::new();
    let mut package_preferences = Vec::new();

    // --- Component type extraction ---
    // Resistors
    if lower.contains("resistor") || lower.contains("resistors") {
        functional_blocks.push(FunctionalBlock {
            name: "ResistorNetwork".to_string(),
            block_type: BlockType::Analog,
            description: extract_context(&lower, "resistor"),
        });
    }

    // Capacitors
    if lower.contains("capacitor") || lower.contains("capacitors") || lower.contains("cap") {
        functional_blocks.push(FunctionalBlock {
            name: "CapacitorBank".to_string(),
            block_type: BlockType::Analog,
            description: extract_context(&lower, "capacitor"),
        });
    }

    // ICs / microcontrollers / processors
    let has_ic = lower.contains("ic ")
        || lower.contains("ics ")
        || lower.contains("integrated circuit")
        || lower.contains("integrated circuits")
        || lower.contains("microcontroller")
        || lower.contains("mcu")
        || lower.contains("processor")
        || lower.contains("cpu")
        || lower.contains("soc");
    if has_ic {
        let block_type = if lower.contains("microcontroller") || lower.contains("mcu") || lower.contains("processor") || lower.contains("cpu") || lower.contains("soc") {
            BlockType::Processing
        } else {
            BlockType::Processing
        };
        functional_blocks.push(FunctionalBlock {
            name: "IC".to_string(),
            block_type,
            description: extract_context(&lower, "ic"),
        });
    }

    // Connectors
    if lower.contains("connector") || lower.contains("connectors") || lower.contains("header") || lower.contains("socket") {
        functional_blocks.push(FunctionalBlock {
            name: "Connector".to_string(),
            block_type: BlockType::Communication,
            description: extract_context(&lower, "connector"),
        });
    }

    // Power supply / regulator
    if lower.contains("power") || lower.contains("regulator") || lower.contains("ldo") || lower.contains("dc-dc") || lower.contains("buck") || lower.contains("boost") {
        functional_blocks.push(FunctionalBlock {
            name: "PowerSupply".to_string(),
            block_type: BlockType::Power,
            description: extract_context(&lower, "power"),
        });
    }

    // Memory
    if lower.contains("memory") || lower.contains("ram") || lower.contains("flash") || lower.contains("eeprom") || lower.contains("sdram") || lower.contains("ddr") {
        functional_blocks.push(FunctionalBlock {
            name: "Memory".to_string(),
            block_type: BlockType::Memory,
            description: extract_context(&lower, "memory"),
        });
    }

    // Sensors
    if lower.contains("sensor") || lower.contains("accelerometer") || lower.contains("gyroscope") || lower.contains("thermistor") || lower.contains("photodiode") {
        functional_blocks.push(FunctionalBlock {
            name: "Sensor".to_string(),
            block_type: BlockType::Sensor,
            description: extract_context(&lower, "sensor"),
        });
    }

    // Actuators
    if lower.contains("motor") || lower.contains("actuator") || lower.contains("relay") || lower.contains("servo") || lower.contains("led") {
        functional_blocks.push(FunctionalBlock {
            name: "Actuator".to_string(),
            block_type: BlockType::Actuator,
            description: extract_context(&lower, "actuator"),
        });
    }

    // --- Voltage spec extraction ---
    // Patterns: "3.3V", "3v3", "5V", "5v", "12V", "1.8V", etc.
    let volt_patterns = [
        (r"(\d+\.?\d*)\s*[vV]", "V"),
        (r"(\d+\.?\d*)\s*volt", "V"),
    ];
    for (pat, unit) in &volt_patterns {
        if let Ok(re) = regex::Regex::new(pat) {
            for cap in re.captures_iter(description) {
                if let Some(m) = cap.get(1) {
                    let value = m.as_str();
                    electrical_constraints.push(ElectricalConstraint {
                        parameter: "voltage".to_string(),
                        value: format!("{}{}", value, unit),
                        unit: unit.to_string(),
                    });
                }
            }
        }
    }

    // --- Pin count extraction ---
    // Patterns: "8-pin", "8 pin", "32-pin", "100pin", etc.
    if let Ok(re) = regex::Regex::new(r"(\d+)\s*-?\s*pin") {
        for cap in re.captures_iter(description) {
            if let Some(m) = cap.get(1) {
                electrical_constraints.push(ElectricalConstraint {
                    parameter: "pin_count".to_string(),
                    value: m.as_str().to_string(),
                    unit: "pins".to_string(),
                });
            }
        }
    }

    // --- Interface type extraction ---
    let interface_defs = [
        ("UART", "uart", "controller", None),
        ("USART", "usart", "controller", None),
        ("SPI", "spi", "master", Some(10.0)),
        ("I2C", "i2c", "master", Some(1.0)),
        ("CAN", "can", "controller", Some(1.0)),
        ("USB", "usb", "device", Some(480.0)),
        ("Ethernet", "ethernet", "mac", Some(1000.0)),
        ("JTAG", "jtag", "master", None),
        ("SWD", "swd", "debug", None),
        ("GPIO", "gpio", "bidirectional", None),
        ("PWM", "pwm", "output", None),
        ("ADC", "adc", "input", None),
        ("DAC", "dac", "output", None),
    ];

    for (protocol, keyword, default_role, default_speed) in &interface_defs {
        if lower.contains(keyword) {
            // Try to determine role from context
            let role = if lower.contains(&format!("{} master", keyword)) || lower.contains(&format!("{} host", keyword)) {
                "master".to_string()
            } else if lower.contains(&format!("{} slave", keyword)) || lower.contains(&format!("{} device", keyword)) {
                "slave".to_string()
            } else {
                default_role.to_string()
            };

            // Try to extract speed if mentioned (e.g., "SPI at 50MHz")
            let speed_mbps = extract_interface_speed(&lower, keyword).or(*default_speed);

            interfaces.push(InterfaceRequirement {
                protocol: protocol.to_string(),
                role,
                speed_mbps,
            });
        }
    }

    // --- Package preference extraction ---
    let package_types = [
        "QFP", "QFN", "BGA", "SOP", "SOIC", "TSOP", "SOT", "DIP", "TQFP",
        "LQFP", "VQFN", "WLCSP", "CSP", "DFN", "MSOP", "VSSOP",
    ];
    for pkg in &package_types {
        if lower.contains(&pkg.to_ascii_lowercase()) {
            package_preferences.push(pkg.to_string());
        }
    }

    // Deduplicate voltage constraints
    electrical_constraints.sort_by(|a, b| a.value.cmp(&b.value));
    electrical_constraints.dedup_by(|a, b| a.value == b.value);

    tracing::info!(
        blocks = functional_blocks.len(),
        constraints = electrical_constraints.len(),
        interfaces = interfaces.len(),
        packages = package_preferences.len(),
        "requirements parsing complete"
    );

    ParsedRequirements {
        functional_blocks,
        electrical_constraints,
        interfaces,
        package_preferences,
    }
}

/// Extract a short context snippet around a keyword for the description field.
fn extract_context(lower_text: &str, keyword: &str) -> String {
    if let Some(pos) = lower_text.find(keyword) {
        let start = pos.saturating_sub(40);
        let end = (pos + keyword.len() + 60).min(lower_text.len());
        let snippet = &lower_text[start..end];
        // Trim to word boundaries
        let trimmed = snippet.trim();
        trimmed.to_string()
    } else {
        keyword.to_string()
    }
}

/// Extract interface speed from text (e.g., "SPI at 50MHz", "I2C 400kHz").
fn extract_interface_speed(lower_text: &str, keyword: &str) -> Option<f64> {
    // Match patterns like "spi at 50mhz", "i2c 400khz"
    let pattern = format!(r"(?:{})\s*(?:at\s*)?(\d+(?:\.\d+)?)\s*(mhz|khz|gbps|mbps)", regex::escape(keyword));
    if let Ok(re) = regex::Regex::new(&pattern) {
        for cap in re.captures_iter(lower_text) {
            if let (Some(val), Some(unit)) = (cap.get(1), cap.get(2)) {
                let num: f64 = val.as_str().parse().ok()?;
                return match unit.as_str() {
                    "mhz" => Some(num),
                    "khz" => Some(num / 1000.0),
                    "gbps" => Some(num * 1000.0),
                    "mbps" => Some(num),
                    _ => None,
                };
            }
        }
    }
    None
}