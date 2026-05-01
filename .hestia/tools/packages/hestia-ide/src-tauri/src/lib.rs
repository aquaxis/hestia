#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use tauri::Manager;

#[tauri::command]
async fn start_conductor(conductor: String) -> Result<String, String> {
    Ok(format!("Started {}-conductor", conductor))
}

#[tauri::command]
async fn stop_conductor(conductor: String) -> Result<String, String> {
    Ok(format!("Stopped {}-conductor", conductor))
}

#[tauri::command]
async fn get_conductor_status(conductor: String) -> Result<String, String> {
    Ok(format!("{}-conductor: Offline", conductor))
}

#[tauri::command]
async fn run_cli_command(command: String, args: Vec<String>) -> Result<String, String> {
    Ok(format!("hestia {} {:?} completed", command, args))
}

pub fn run() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            start_conductor,
            stop_conductor,
            get_conductor_status,
            run_cli_command,
        ])
        .run(tauri::generate_context!())
        .expect("error while running hestia-ide");
}