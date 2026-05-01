//! Code generator — produces C headers, Rust crates, Python modules, SVD

use crate::register_map::RegisterMap;
use crate::fsm_states::HalBuildState;

/// HAL code generator
#[derive(Debug)]
pub struct CodeGenerator {
    /// Target output directory
    output_dir: std::path::PathBuf,
}

impl CodeGenerator {
    /// Create a new CodeGenerator
    pub fn new(output_dir: std::path::PathBuf) -> Self {
        Self { output_dir }
    }

    /// Generate C header file from register map
    pub async fn generate_c(&self, regmap: &RegisterMap) -> Result<CodeGenResult, anyhow::Error> {
        tracing::info!(map = %regmap.name, "generating C header");
        let output_path = self.output_dir.join(format!("{}.h", regmap.name.to_lowercase()));
        // TODO: actual codegen
        Ok(CodeGenResult {
            success: true,
            output_path,
            state: HalBuildState::Generating,
        })
    }

    /// Generate Rust crate from register map
    pub async fn generate_rust(&self, regmap: &RegisterMap) -> Result<CodeGenResult, anyhow::Error> {
        tracing::info!(map = %regmap.name, "generating Rust crate");
        let output_path = self.output_dir.join(format!("{}_hal", regmap.name.to_lowercase()));
        // TODO: actual codegen
        Ok(CodeGenResult {
            success: true,
            output_path,
            state: HalBuildState::Generating,
        })
    }

    /// Generate Python module from register map
    pub async fn generate_python(&self, regmap: &RegisterMap) -> Result<CodeGenResult, anyhow::Error> {
        tracing::info!(map = %regmap.name, "generating Python module");
        let output_path = self.output_dir.join(format!("{}.py", regmap.name.to_lowercase()));
        // TODO: actual codegen
        Ok(CodeGenResult {
            success: true,
            output_path,
            state: HalBuildState::Generating,
        })
    }

    /// Generate SVD (System View Description) from register map
    pub async fn generate_svd(&self, regmap: &RegisterMap) -> Result<CodeGenResult, anyhow::Error> {
        tracing::info!(map = %regmap.name, "generating SVD");
        let output_path = self.output_dir.join(format!("{}.svd", regmap.name.to_lowercase()));
        // TODO: actual codegen
        Ok(CodeGenResult {
            success: true,
            output_path,
            state: HalBuildState::Generating,
        })
    }
}

/// Code generation result
#[derive(Debug)]
pub struct CodeGenResult {
    pub success: bool,
    pub output_path: std::path::PathBuf,
    pub state: HalBuildState,
}