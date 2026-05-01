//! Vivado TCL template rendering

/// Generate a Vivado project creation TCL script.
pub fn create_project_script(project_name: &str, device: &str, top_module: &str) -> String {
    format!(
        r#"# Vivado project creation script
create_project {project_name} ./vivado -part {device}
set_property top {top_module} [current_fileset]
"#,
        project_name = project_name,
        device = device,
        top_module = top_module,
    )
}

/// Generate a Vivado synthesis TCL script.
pub fn synth_script(_project_name: &str) -> String {
    format!(
        r#"# Vivado synthesis script
reset_run synth_1
launch_runs synth_1 -jobs 4
wait_on_run synth_1
"#,
    )
}

/// Generate a Vivado implementation TCL script.
pub fn impl_script() -> String {
    r#"# Vivado implementation script
launch_runs impl_1 -jobs 4
wait_on_run impl_1
"#.to_string()
}

/// Generate a Vivado bitstream generation TCL script.
pub fn bitstream_script() -> String {
    r#"# Vivado bitstream generation script
launch_runs impl_1 -to_step write_bitstream -jobs 4
wait_on_run impl_1
"#.to_string()
}