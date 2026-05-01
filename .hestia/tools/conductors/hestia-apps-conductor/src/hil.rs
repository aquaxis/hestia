//! HIL (Hardware-in-the-Loop) and SIL (Software-in-the-Loop) test runner

use crate::fsm_states::AppsBuildState;

/// HIL/SIL test runner
#[derive(Debug)]
pub struct HilSilTest {
    /// Test mode: "sil" or "hil"
    pub mode: String,
    /// Debug probe identifier (e.g., "jlink", "stlink", "cmsis-dap")
    pub probe: Option<String>,
    /// Target device identifier
    pub target_device: Option<String>,
}

/// Test execution result
#[derive(Debug)]
pub struct TestResult {
    pub passed: bool,
    pub total_tests: u32,
    pub passed_tests: u32,
    pub failed_tests: u32,
    pub duration_secs: f64,
    pub log_path: std::path::PathBuf,
    pub state: AppsBuildState,
}

impl HilSilTest {
    /// Create a new HIL/SIL test configuration
    pub fn new(mode: &str) -> Self {
        Self {
            mode: mode.to_string(),
            probe: None,
            target_device: None,
        }
    }

    /// Set the debug probe
    pub fn with_probe(mut self, probe: &str) -> Self {
        self.probe = Some(probe.to_string());
        self
    }

    /// Set the target device
    pub fn with_target(mut self, device: &str) -> Self {
        self.target_device = Some(device.to_string());
        self
    }

    /// Run Software-in-the-Loop tests (QEMU or host simulation)
    pub async fn run_sil(
        &self,
        project_dir: &std::path::Path,
    ) -> Result<TestResult, anyhow::Error> {
        tracing::info!(mode = %self.mode, "running SIL tests");
        // TODO: invoke QEMU / simulator, collect results
        Ok(TestResult {
            passed: true,
            total_tests: 0,
            passed_tests: 0,
            failed_tests: 0,
            duration_secs: 0.0,
            log_path: project_dir.join("build").join("test").join("sil.log"),
            state: AppsBuildState::Testing,
        })
    }

    /// Run Hardware-in-the-Loop tests (on actual target device)
    pub async fn run_hil(
        &self,
        project_dir: &std::path::Path,
    ) -> Result<TestResult, anyhow::Error> {
        tracing::info!(
            mode = %self.mode,
            probe = ?self.probe,
            target = ?self.target_device,
            "running HIL tests"
        );
        // TODO: flash firmware, run on-device tests, collect results via probe
        Ok(TestResult {
            passed: true,
            total_tests: 0,
            passed_tests: 0,
            failed_tests: 0,
            duration_secs: 0.0,
            log_path: project_dir.join("build").join("test").join("hil.log"),
            state: AppsBuildState::Testing,
        })
    }
}