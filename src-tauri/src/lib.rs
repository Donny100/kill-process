use serde::{Deserialize, Serialize};
use std::process::Command;
use std::str::FromStr;

#[derive(Debug, Serialize, Deserialize)]
pub struct ProcessInfo {
    pub pid: String,
    pub name: String,
    pub port: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PortCheckResult {
    pub is_occupied: bool,
    pub processes: Vec<ProcessInfo>,
    pub error: Option<String>,
}

// Check if a port is occupied and return process information
#[tauri::command]
fn check_port(port: String) -> PortCheckResult {
    let port_num = match u16::from_str(&port) {
        Ok(p) => p,
        Err(_) => {
            return PortCheckResult {
                is_occupied: false,
                processes: vec![],
                error: Some("Invalid port number".to_string()),
            };
        }
    };

    // Use lsof to check port usage on macOS
    let output = Command::new("lsof")
        .args(&["-i", &format!(":{}", port_num), "-P", "-n"])
        .output();

    match output {
        Ok(output) => {
            if output.status.success() {
                let output_str = String::from_utf8_lossy(&output.stdout);
                let processes = parse_lsof_output(&output_str, &port);
                
                PortCheckResult {
                    is_occupied: !processes.is_empty(),
                    processes,
                    error: None,
                }
            } else {
                PortCheckResult {
                    is_occupied: false,
                    processes: vec![],
                    error: None,
                }
            }
        }
        Err(e) => PortCheckResult {
            is_occupied: false,
            processes: vec![],
            error: Some(format!("Failed to execute lsof: {}", e)),
        },
    }
}

// Kill a process by PID
#[tauri::command]
fn kill_process(pid: String) -> Result<String, String> {
    let output = Command::new("kill")
        .arg("-9")
        .arg(&pid)
        .output();

    match output {
        Ok(output) => {
            if output.status.success() {
                Ok(format!("Process {} killed successfully", pid))
            } else {
                let error_msg = String::from_utf8_lossy(&output.stderr);
                Err(format!("Failed to kill process {}: {}", pid, error_msg))
            }
        }
        Err(e) => Err(format!("Failed to execute kill command: {}", e)),
    }
}

// Parse lsof output to extract process information
fn parse_lsof_output(output: &str, port: &str) -> Vec<ProcessInfo> {
    let mut processes = Vec::new();
    
    for line in output.lines().skip(1) { // Skip header line
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 9 {
            let name = parts[0].to_string();
            let pid = parts[1].to_string();
            
            processes.push(ProcessInfo {
                pid,
                name,
                port: port.to_string(),
            });
        }
    }
    
    processes
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![check_port, kill_process])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
