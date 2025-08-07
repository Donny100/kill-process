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
pub struct ProcessDetail {
    pub pid: String,
    pub name: String,
    pub port: String,
    pub user: Option<String>,
    pub command: Option<String>,
    pub cpu_usage: Option<String>,
    pub memory_usage: Option<String>,
    pub start_time: Option<String>,
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

// Kill a process by PID using SIGKILL signal (force kill)
#[tauri::command]
fn kill_process(pid: String) -> Result<String, String> {
    kill_process_with_signal(pid, true)
}

// Kill a process by PID using SIGTERM signal (graceful kill)
#[tauri::command]
fn graceful_kill_process(pid: String) -> Result<String, String> {
    kill_process_with_signal(pid, false)
}

// Internal function to kill process with specified signal
fn kill_process_with_signal(pid: String, force: bool) -> Result<String, String> {
    let signal_type = if force { "SIGKILL (-9)" } else { "SIGTERM (-15)" };
    println!("[INFO] Attempting to {} process with PID: {} using {}", 
             if force { "force kill" } else { "gracefully terminate" }, pid, signal_type);
    
    // Validate PID format
    if let Err(e) = pid.parse::<u32>() {
        println!("[ERROR] Invalid PID format '{}': {}", pid, e);
        return Err(format!("Invalid PID format: {}", pid));
    }
    
    let signal_arg = if force { "-9" } else { "-15" };
    println!("[DEBUG] Executing kill {} command for PID: {}", signal_arg, pid);
    
    let output = Command::new("kill")
        .arg(signal_arg)
        .arg(&pid)
        .output();

    match output {
        Ok(output) => {
            if output.status.success() {
                let action = if force { "force killed" } else { "gracefully terminated" };
                println!("[INFO] Successfully {} process with PID: {}", action, pid);
                Ok(format!("Process {} {} successfully", pid, action))
            } else {
                let error_msg = String::from_utf8_lossy(&output.stderr);
                println!("[ERROR] Failed to {} process {}: status={}, stderr='{}'", 
                         if force { "force kill" } else { "gracefully terminate" }, 
                         pid, output.status, error_msg);
                Err(format!("Failed to {} process {}: {}", 
                           if force { "force kill" } else { "gracefully terminate" }, 
                           pid, error_msg))
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

// Get detailed process information using ps command
#[tauri::command]
fn get_process_detail(pid: String) -> Result<ProcessDetail, String> {
    println!("[INFO] Getting detailed information for process PID: {}", pid);
    
    // Validate PID format
    if let Err(e) = pid.parse::<u32>() {
        println!("[ERROR] Invalid PID format '{}': {}", pid, e);
        return Err(format!("Invalid PID format: {}", pid));
    }
    
    // Use ps command to get detailed process information
    // We'll use separate ps calls for better field parsing
    println!("[DEBUG] Getting basic process info for PID: {}", pid);
    
    // Get basic info: pid, command name, user, full command
    let basic_args = vec!["-p", &pid, "-o", "pid=,comm=,user=,args="];
    let basic_output = Command::new("ps").args(&basic_args).output();
    
    // Get resource usage: pid, pcpu, pmem
    let resource_args = vec!["-p", &pid, "-o", "pid=,pcpu=,pmem="];
    let resource_output = Command::new("ps").args(&resource_args).output();
    
    // Get start time: pid, lstart
    let time_args = vec!["-p", &pid, "-o", "pid=,lstart="];
    let time_output = Command::new("ps").args(&time_args).output();

    match (basic_output, resource_output, time_output) {
        (Ok(basic), Ok(resource), Ok(time)) => {
            if basic.status.success() && resource.status.success() && time.status.success() {
                let basic_str = String::from_utf8_lossy(&basic.stdout);
                let resource_str = String::from_utf8_lossy(&resource.stdout);
                let time_str = String::from_utf8_lossy(&time.stdout);
                
                println!("[DEBUG] Basic info: {}", basic_str.trim());
                println!("[DEBUG] Resource info: {}", resource_str.trim());
                println!("[DEBUG] Time info: {}", time_str.trim());
                
                // Parse basic info
                let basic_parts: Vec<&str> = basic_str.trim().split_whitespace().collect();
                if basic_parts.len() >= 4 {
                    let pid_parsed = basic_parts[0];
                    let name = basic_parts[1];
                    let user = basic_parts[2];
                    let command = basic_parts[3..].join(" ");
                    
                    // Parse resource info
                    let resource_parts: Vec<&str> = resource_str.trim().split_whitespace().collect();
                    let (cpu_usage, memory_usage) = if resource_parts.len() >= 3 {
                        (
                            Some(format!("{}%", resource_parts[1])),
                            Some(format!("{}%", resource_parts[2]))
                        )
                    } else {
                        (None, None)
                    };
                    
                    // Parse start time (skip PID, take the rest)
                    let start_time = if let Some(first_space) = time_str.trim().find(' ') {
                        Some(time_str.trim()[first_space + 1..].to_string())
                    } else {
                        None
                    };
                    
                    // Try to get port information from lsof
                    let port_info = get_process_port(&pid);
                    
                    let detail = ProcessDetail {
                        pid: pid_parsed.to_string(),
                        name: name.to_string(),
                        port: port_info.unwrap_or_else(|| "Unknown".to_string()),
                        user: Some(user.to_string()),
                        command: Some(command),
                        cpu_usage,
                        memory_usage,
                        start_time,
                    };
                    
                    println!("[INFO] Successfully retrieved detailed information for PID: {}", pid);
                    Ok(detail)
                } else {
                    println!("[ERROR] Unable to parse basic process info for PID: {}", pid);
                    Err("Unable to parse basic process info".to_string())
                }
            } else {
                println!("[ERROR] One or more ps commands failed for PID: {}", pid);
                Err(format!("Failed to get process information for PID: {}", pid))
            }
        }
        _ => {
            println!("[ERROR] Failed to execute ps commands for PID: {}", pid);
            Err("Failed to execute ps commands".to_string())
        }
    }
}

// Helper function to get port information for a specific process
fn get_process_port(pid: &str) -> Option<String> {
    let lsof_args = vec!["-p", pid, "-P", "-n", "-iTCP"];
    
    let output = Command::new("lsof")
        .args(&lsof_args)
        .output();
    
    if let Ok(output) = output {
        if output.status.success() {
            let output_str = String::from_utf8_lossy(&output.stdout);
            
            // Parse lsof output to find port information
            for line in output_str.lines().skip(1) {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 9 {
                    let name_field = parts[8];
                    // Look for patterns like *:port or localhost:port
                    if name_field.contains(':') {
                        if let Some(port_part) = name_field.split(':').last() {
                            // Check if it's a number (port) and not a service name
                            if port_part.parse::<u16>().is_ok() {
                                return Some(port_part.to_string());
                            }
                        }
                    }
                }
            }
        }
    }
    
    None
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![check_port, kill_process, graceful_kill_process, get_process_detail])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
