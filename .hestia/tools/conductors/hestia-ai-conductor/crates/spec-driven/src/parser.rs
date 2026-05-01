use serde::{Deserialize, Serialize};
use uuid::Uuid;
use crate::SpecError;

/// A parsed design specification.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DesignSpec {
    pub id: String,
    pub requirements: Vec<Requirement>,
    pub constraints: Vec<Constraint>,
    pub interfaces: Vec<Interface>,
}

/// A single requirement extracted from the spec.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Requirement {
    pub id: String,
    pub text: String,
}

/// A constraint extracted from the spec.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Constraint {
    pub id: String,
    pub text: String,
}

/// An interface definition extracted from the spec.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Interface {
    pub id: String,
    pub text: String,
}

/// Parses raw specification text into a structured `DesignSpec`.
pub struct SpecParser;

impl SpecParser {
    /// Create a new spec parser.
    pub fn new() -> Self {
        Self
    }

    /// Parse the input specification text and return a `DesignSpec`.
    ///
    /// Line prefix conventions:
    /// - `REQ:` marks a requirement
    /// - `CON:` marks a constraint
    /// - `IF:`  marks an interface
    pub fn parse(&self, input: &str) -> Result<DesignSpec, SpecError> {
        let mut requirements = Vec::new();
        let mut constraints = Vec::new();
        let mut interfaces = Vec::new();

        for (line_num, line) in input.lines().enumerate() {
            let trimmed = line.trim();
            if trimmed.is_empty() {
                continue;
            }

            if let Some(text) = trimmed.strip_prefix("REQ:") {
                let text = text.trim().to_string();
                if text.is_empty() {
                    return Err(SpecError::InvalidRequirement(format!(
                        "empty REQ on line {}",
                        line_num + 1
                    )));
                }
                let id = format!("REQ-{}", requirements.len() + 1);
                requirements.push(Requirement { id, text });
            } else if let Some(text) = trimmed.strip_prefix("CON:") {
                let text = text.trim().to_string();
                if text.is_empty() {
                    return Err(SpecError::InvalidRequirement(format!(
                        "empty CON on line {}",
                        line_num + 1
                    )));
                }
                let id = format!("CON-{}", constraints.len() + 1);
                constraints.push(Constraint { id, text });
            } else if let Some(text) = trimmed.strip_prefix("IF:") {
                let text = text.trim().to_string();
                if text.is_empty() {
                    return Err(SpecError::InvalidRequirement(format!(
                        "empty IF on line {}",
                        line_num + 1
                    )));
                }
                let id = format!("IF-{}", interfaces.len() + 1);
                interfaces.push(Interface { id, text });
            }
            // Lines without a recognized prefix are silently skipped.
        }

        Ok(DesignSpec {
            id: Uuid::new_v4().to_string(),
            requirements,
            constraints,
            interfaces,
        })
    }
}

impl Default for SpecParser {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_requirements() {
        let input = "REQ: The system shall authenticate users\nCON: Must respond within 200ms\nIF: REST API endpoint /auth";
        let spec = SpecParser::new().parse(input).unwrap();
        assert_eq!(spec.requirements.len(), 1);
        assert_eq!(spec.constraints.len(), 1);
        assert_eq!(spec.interfaces.len(), 1);
        assert_eq!(spec.requirements[0].id, "REQ-1");
        assert_eq!(spec.constraints[0].id, "CON-1");
        assert_eq!(spec.interfaces[0].id, "IF-1");
    }

    #[test]
    fn test_empty_input() {
        let spec = SpecParser::new().parse("").unwrap();
        assert!(spec.requirements.is_empty());
        assert!(spec.constraints.is_empty());
        assert!(spec.interfaces.is_empty());
    }

    #[test]
    fn test_unknown_prefix_ignored() {
        let spec = SpecParser::new().parse("NOTE: this is just a note").unwrap();
        assert!(spec.requirements.is_empty());
    }
}
