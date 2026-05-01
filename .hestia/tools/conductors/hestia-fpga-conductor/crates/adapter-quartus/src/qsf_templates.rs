//! Quartus QSF/QIP template generation

/// Generate a Quartus project QSF file.
pub fn create_qsf(project_name: &str, device: &str, top_module: &str) -> String {
    format!(
        r#"# Quartus project settings for {project_name}
set_global_assignment -name FAMILY "Cyclone V"
set_global_assignment -name DEVICE {device}
set_global_assignment -name TOP_LEVEL_ENTITY {top_module}
"#,
        project_name = project_name,
        device = device,
        top_module = top_module,
    )
}

/// Generate a Quartus compilation script.
pub fn compile_script() -> String {
    r#"# Quartus compilation flow
load_package flow
execute_flow -compile
"#.to_string()
}

/// Generate a QIP file reference.
pub fn qip_reference(qip_path: &str) -> String {
    format!("set_global_assignment -name QIP_FILE {}", qip_path)
}