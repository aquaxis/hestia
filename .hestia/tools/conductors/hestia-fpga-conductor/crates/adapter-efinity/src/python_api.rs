//! Efinity Python API integration

/// Efinity Python API command builder for Efinity IDE integration.
pub struct EfinityApi {
    python_path: String,
    efinity_home: String,
}

impl EfinityApi {
    /// Create a new Efinity API instance.
    pub fn new(python_path: &str, efinity_home: &str) -> Self {
        Self {
            python_path: python_path.to_string(),
            efinity_home: efinity_home.to_string(),
        }
    }

    /// Build the command for Efinity synthesis.
    pub fn synth_command(&self, project: &str) -> String {
        format!(
            "{python} {home}/bin/efx_run.py synth -p {project}",
            python = self.python_path,
            home = self.efinity_home,
            project = project,
        )
    }

    /// Build the command for Efinity place & route.
    pub fn place_route_command(&self, project: &str) -> String {
        format!(
            "{python} {home}/bin/efx_run.py pnr -p {project}",
            python = self.python_path,
            home = self.efinity_home,
            project = project,
        )
    }

    /// Build the command for Efinity bitstream generation.
    pub fn bitstream_command(&self, project: &str) -> String {
        format!(
            "{python} {home}/bin/efx_run.py bgen -p {project}",
            python = self.python_path,
            home = self.efinity_home,
            project = project,
        )
    }

    /// Build the command for Efinity programming.
    pub fn program_command(&self, project: &str, device: &str) -> String {
        format!(
            "{python} {home}/bin/efx_run.py prog -p {project} -d {device}",
            python = self.python_path,
            home = self.efinity_home,
            project = project,
            device = device,
        )
    }
}