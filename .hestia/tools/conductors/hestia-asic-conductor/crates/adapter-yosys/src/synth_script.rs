//! Yosys synthesis script generation

use crate::SynthStrategy;

/// Generate a Yosys synthesis TCL script.
pub fn generate_synth_script(
    top_module: &str,
    rtl_files: &[String],
    strategy: SynthStrategy,
) -> String {
    let read_files: Vec<String> = rtl_files
        .iter()
        .map(|f| format!("read_verilog {}", f))
        .collect();

    let opt_pass = match strategy {
        SynthStrategy::Area => "opt_clean; opt_expr; opt_merge",
        SynthStrategy::Speed => "opt; opt_clean; opt_expr",
        SynthStrategy::Balanced => "opt",
    };

    format!(
        r#"# Yosys synthesis script for {top_module}
{read_cmds}
hierarchy -check -top {top_module}
{opt_pass}
synth -top {top_module}
write_json {top_module}.json
"#,
        top_module = top_module,
        read_cmds = read_files.join("\n"),
        opt_pass = opt_pass,
    )
}