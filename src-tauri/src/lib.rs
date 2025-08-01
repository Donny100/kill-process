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
    println!("[INFO] Starting port check for port: {}", port);
    
    let port_num = match u16::from_str(&port) {
        Ok(p) => {
            println!("[DEBUG] Port number parsed successfully: {}", p);
            p
        },
        Err(e) => {
            println!("[ERROR] Invalid port number '{}': {}", port, e);
            return PortCheckResult {
                is_occupied: false,
                processes: vec![],
                error: Some("Invalid port number".to_string()),
            };
        }
    };

    // Use lsof to check port usage - works on macOS and Linux
    // -sTCP:LISTEN only shows processes in LISTEN state to avoid duplicates
    let port_arg = format!(":{}", port_num);
    let lsof_args = vec!["-i", &port_arg, "-P", "-n", "-sTCP:LISTEN"];
    println!("[DEBUG] Executing command: lsof {}", lsof_args.join(" "));
    
    let output = Command::new("lsof")
        .args(&lsof_args)
        .output();

    match output {
        Ok(output) => {
            if output.status.success() {
                let output_str = String::from_utf8_lossy(&output.stdout);
                println!("[DEBUG] lsof command successful, output length: {} characters", output_str.len());
                println!("[DEBUG] lsof raw output:\n{}", output_str);
                
                let processes = parse_lsof_output(&output_str, &port);
                println!("[INFO] Found {} LISTEN processes using port {}", processes.len(), port);
                
                for process in &processes {
                    println!("[DEBUG] Process found - PID: {}, Name: {}, Port: {}", 
                             process.pid, process.name, process.port);
                }
                
                PortCheckResult {
                    is_occupied: !processes.is_empty(),
                    processes,
                    error: None,
                }
            } else {
                let error_str = String::from_utf8_lossy(&output.stderr);
                println!("[DEBUG] lsof command failed with status: {}, stderr: {}", 
                         output.status, error_str);
                println!("[INFO] Port {} appears to be available (no processes found)", port);
                
                PortCheckResult {
                    is_occupied: false,
                    processes: vec![],
                    error: None,
                }
            }
        }
        Err(e) => {
            println!("[ERROR] Failed to execute lsof command: {}", e);
            PortCheckResult {
                is_occupied: false,
                processes: vec![],
                error: Some(format!("Failed to execute lsof: {}", e)),
            }
        },
    }
}

// Kill a process by PID using SIGKILL signal
#[tauri::command]
fn kill_process(pid: String) -> Result<String, String> {
    println!("[INFO] Attempting to kill process with PID: {}", pid);
    
    // Validate PID format
    if let Err(e) = pid.parse::<u32>() {
        println!("[ERROR] Invalid PID format '{}': {}", pid, e);
        return Err(format!("Invalid PID format: {}", pid));
    }
    
    println!("[DEBUG] Executing kill -9 command for PID: {}", pid);
    let output = Command::new("kill")
        .arg("-9")
        .arg(&pid)
        .output();

    match output {
        Ok(output) => {
            if output.status.success() {
                println!("[INFO] Successfully killed process with PID: {}", pid);
                Ok(format!("Process {} terminated successfully", pid))
            } else {
                let error_msg = String::from_utf8_lossy(&output.stderr);
                println!("[ERROR] Failed to kill process {}: status={}, stderr='{}'", 
                         pid, output.status, error_msg);
                Err(format!("Failed to terminate process {}: {}", pid, error_msg))
            }
        }
        Err(e) => {
            println!("[ERROR] Failed to execute kill command for PID {}: {}", pid, e);
            Err(format!("Failed to execute kill command: {}", e))
        },
    }
}

// Parse lsof output to extract process information
// lsof output format: COMMAND PID USER FD TYPE DEVICE SIZE/OFF NODE NAME
// Since we use -sTCP:LISTEN, all results are already LISTEN processes
fn parse_lsof_output(output: &str, port: &str) -> Vec<ProcessInfo> {
    println!("[DEBUG] Parsing lsof output, total lines: {}", output.lines().count());
    let mut processes = Vec::new();
    
    // Skip the header line and process each line
    for (line_num, line) in output.lines().skip(1).enumerate() {
        let parts: Vec<&str> = line.split_whitespace().collect();
        println!("[DEBUG] Line {}: {} parts - {}", line_num + 1, parts.len(), line);
        
        if parts.len() >= 2 {
            let name = parts[0].to_string();
            let pid = parts[1].to_string();
            
            println!("[DEBUG] Extracted LISTEN process - Name: '{}', PID: '{}'", name, pid);
            
            processes.push(ProcessInfo {
                pid,
                name,
                port: port.to_string(),
            });
        } else {
            println!("[WARN] Skipping malformed line {}: not enough parts ({})", 
                     line_num + 1, parts.len());
        }
    }
    
    println!("[INFO] Successfully parsed {} LISTEN processes from lsof output", processes.len());
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
