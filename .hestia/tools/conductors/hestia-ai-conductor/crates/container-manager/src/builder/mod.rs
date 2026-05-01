//! Container builder — reads ContainerToml, generates Containerfile, runs podman build

use std::path::Path;
use std::process::Command;

use minijinja::Environment;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use tracing::info;

#[derive(Debug, Error)]
pub enum BuilderError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("TOML parse error: {0}")]
    Toml(#[from] toml::de::Error),
    #[error("Template render error: {0}")]
    Template(#[from] minijinja::Error),
    #[error("Podman build failed: {0}")]
    BuildFailed(String),
}

/// Minimal representation of a container TOML descriptor.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContainerToml {
    pub container: ContainerSection,
    #[serde(default)]
    pub tools: Vec<ToolEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContainerSection {
    pub name: String,
    pub base: String,
    #[serde(default)]
    pub tag: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolEntry {
    pub name: String,
    pub version: String,
}

const CONTAINERFILE_TEMPLATE: &str = r#"FROM {{ base }}
LABEL org.opencontainers.image.title="{{ name }}"

RUN apt-get update && apt-get install -y \
    ca-certificates \
    curl \
    && rm -rf /var/lib/apt/lists/*
"#;

pub struct ContainerBuilder {
    env: Environment<'static>,
}

impl ContainerBuilder {
    pub fn new() -> Self {
        let mut env = Environment::new();
        env.add_template("Containerfile", CONTAINERFILE_TEMPLATE)
            .expect("embedded template is valid");
        Self { env }
    }

    /// Read a ContainerToml from the given filesystem path.
    pub fn build_from_toml(&self, path: &Path) -> Result<ContainerToml, BuilderError> {
        let contents = std::fs::read_to_string(path)?;
        let container_toml: ContainerToml = toml::from_str(&contents)?;
        Ok(container_toml)
    }

    /// Render a Containerfile string from the given ContainerToml descriptor.
    pub fn generate_containerfile(&self, spec: &ContainerToml) -> Result<String, BuilderError> {
        let tmpl = self.env.get_template("Containerfile")?;
        let ctx = minijinja::context! {
            base => &spec.container.base,
            name => &spec.container.name,
        };
        let rendered = tmpl.render(ctx)?;
        Ok(rendered)
    }

    /// Execute `podman build` with the given context directory and tag.
    pub fn run_podman_build(
        &self,
        context_dir: &Path,
        tag: &str,
    ) -> Result<(), BuilderError> {
        info!("running podman build for tag={tag}");
        let status = Command::new("podman")
            .args(["build", "-t", tag, context_dir.to_string_lossy().as_ref()])
            .status()?;

        if status.success() {
            Ok(())
        } else {
            Err(BuilderError::BuildFailed(format!(
                "podman build exited with code {:?}",
                status.code()
            )))
        }
    }
}

impl Default for ContainerBuilder {
    fn default() -> Self {
        Self::new()
    }
}