// Integration tests for kill-process library
// These tests verify the core functionality without depending on external commands

use kill_process_lib::{parse_lsof_output, kill_process_with_signal, ProcessInfo, ProcessDetail, PortCheckResult};

#[test]
fn test_parse_lsof_output_basic() {
    let lsof_output = r#"COMMAND   PID    USER   FD   TYPE             DEVICE SIZE/OFF NODE NAME
node     1234 testuser   20u  IPv4 0x1234567890abcdef      0t0  TCP *:3000 (LISTEN)
"#;
    
    let result = parse_lsof_output(lsof_output, "3000");
    
    assert_eq!(result.len(), 1);
    assert_eq!(result[0].pid, "1234");
    assert_eq!(result[0].name, "node");
    assert_eq!(result[0].port, "3000");
}

#[test]
fn test_parse_lsof_output_deduplication() {
    let lsof_output = r#"COMMAND   PID    USER   FD   TYPE             DEVICE SIZE/OFF NODE NAME
node     1234 testuser   20u  IPv4 0x1234567890abcdef      0t0  TCP *:3000 (LISTEN)
node     1234 testuser   21u  IPv6 0x1234567890abcdef      0t0  TCP *:3000 (LISTEN)
"#;
    
    let result = parse_lsof_output(lsof_output, "3000");
    
    // Should deduplicate IPv4 and IPv6 entries for the same PID
    assert_eq!(result.len(), 1);
    assert_eq!(result[0].pid, "1234");
    assert_eq!(result[0].name, "node");
}

#[test]
fn test_parse_lsof_output_multiple_processes() {
    let lsof_output = r#"COMMAND   PID    USER   FD   TYPE             DEVICE SIZE/OFF NODE NAME
node     1234 testuser   20u  IPv4 0x1234567890abcdef      0t0  TCP *:3000 (LISTEN)
nginx    5678 www-data   10u  IPv4 0x9876543210fedcba      0t0  TCP *:3000 (LISTEN)
"#;
    
    let result = parse_lsof_output(lsof_output, "3000");
    
    assert_eq!(result.len(), 2);
    
    // First process
    assert_eq!(result[0].pid, "1234");
    assert_eq!(result[0].name, "node");
    
    // Second process  
    assert_eq!(result[1].pid, "5678");
    assert_eq!(result[1].name, "nginx");
}

#[test]
fn test_parse_lsof_output_empty() {
    let lsof_output = "COMMAND   PID    USER   FD   TYPE             DEVICE SIZE/OFF NODE NAME\n";
    
    let result = parse_lsof_output(lsof_output, "3000");
    
    assert_eq!(result.len(), 0);
}

#[test]
fn test_parse_lsof_output_malformed_line() {
    let lsof_output = r#"COMMAND   PID    USER   FD   TYPE             DEVICE SIZE/OFF NODE NAME
incomplete
node     1234 testuser   20u  IPv4 0x1234567890abcdef      0t0  TCP *:3000 (LISTEN)
"#;
    
    let result = parse_lsof_output(lsof_output, "3000");
    
    // Should skip malformed line and process the valid one
    assert_eq!(result.len(), 1);
    assert_eq!(result[0].pid, "1234");
    assert_eq!(result[0].name, "node");
}

#[test]
fn test_port_check_result_structure() {
    let result = PortCheckResult {
        is_occupied: true,
        processes: vec![
            ProcessInfo {
                pid: "1234".to_string(),
                name: "test_process".to_string(),
                port: "3000".to_string(),
            }
        ],
        error: None,
    };
    
    assert!(result.is_occupied);
    assert_eq!(result.processes.len(), 1);
    assert!(result.error.is_none());
    assert_eq!(result.processes[0].pid, "1234");
}

#[test]
fn test_process_detail_structure() {
    let detail = ProcessDetail {
        pid: "1234".to_string(),
        name: "test_process".to_string(),
        port: "3000, 8080".to_string(), // Multiple ports
        user: Some("testuser".to_string()),
        command: Some("/usr/bin/node app.js".to_string()),
        cpu_usage: Some("5.2%".to_string()),
        memory_usage: Some("128.5%".to_string()),
        start_time: Some("Jan 15 10:30:00".to_string()),
    };
    
    assert_eq!(detail.pid, "1234");
    assert_eq!(detail.name, "test_process");
    assert_eq!(detail.port, "3000, 8080");
    assert!(detail.user.is_some());
    assert!(detail.command.is_some());
    assert!(detail.cpu_usage.is_some());
    assert!(detail.memory_usage.is_some());
    assert!(detail.start_time.is_some());
}

#[test]
fn test_kill_process_with_signal_validation() {
    // Test invalid PID formats
    let invalid_pids = vec!["abc", "", "12.34", "not_a_number"];
    
    for invalid_pid in invalid_pids {
        let result = kill_process_with_signal(invalid_pid.to_string(), true);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Invalid PID format"));
    }
}

#[test]
fn test_valid_pid_format() {
    let valid_pids = vec!["123", "1", "65535"];
    
    for valid_pid in valid_pids {
        // Just test that parsing doesn't fail - we won't actually kill processes in tests
        assert!(valid_pid.parse::<u32>().is_ok());
    }
}

#[test]
fn test_complex_deduplication_scenario() {
    let lsof_output = r#"COMMAND   PID    USER   FD   TYPE             DEVICE SIZE/OFF NODE NAME
ControlCe  830 donny.wu   12u  IPv4 0xe68acf8c27348fbf      0t0  TCP *:5000 (LISTEN)
ControlCe  830 donny.wu   13u  IPv6 0xa99dbc8dfc3f39ec      0t0  TCP *:5000 (LISTEN)
node      1234 testuser   20u  IPv4 0x1234567890abcdef      0t0  TCP *:5000 (LISTEN)
node      1234 testuser   21u  IPv6 0x1234567890abcdef      0t0  TCP *:5000 (LISTEN)
"#;
    
    let result = parse_lsof_output(lsof_output, "5000");
    
    // Should have 2 unique processes (ControlCe and node), not 4
    assert_eq!(result.len(), 2);
    
    // Verify the processes
    let pids: Vec<&String> = result.iter().map(|p| &p.pid).collect();
    assert!(pids.contains(&&"830".to_string()));
    assert!(pids.contains(&&"1234".to_string()));
    
    // Verify no duplicates
    let mut unique_pids = std::collections::HashSet::new();
    for process in &result {
        assert!(unique_pids.insert(&process.pid), "Duplicate PID found: {}", process.pid);
    }
}

// Additional edge case tests

#[test]
fn test_parse_lsof_output_with_special_characters() {
    let lsof_output = r#"COMMAND   PID    USER   FD   TYPE             DEVICE SIZE/OFF NODE NAME
my-app   1234 user.name  20u  IPv4 0x1234567890abcdef      0t0  TCP *:3000 (LISTEN)
"#;
    
    let result = parse_lsof_output(lsof_output, "3000");
    
    assert_eq!(result.len(), 1);
    assert_eq!(result[0].pid, "1234");
    assert_eq!(result[0].name, "my-app");
}

#[test]
fn test_parse_lsof_output_large_pid() {
    let lsof_output = r#"COMMAND   PID    USER   FD   TYPE             DEVICE SIZE/OFF NODE NAME
node     999999 testuser   20u  IPv4 0x1234567890abcdef      0t0  TCP *:3000 (LISTEN)
"#;
    
    let result = parse_lsof_output(lsof_output, "3000");
    
    assert_eq!(result.len(), 1);
    assert_eq!(result[0].pid, "999999");
    assert_eq!(result[0].name, "node");
}

#[test]
fn test_parse_lsof_output_different_ports() {
    let lsof_output = r#"COMMAND   PID    USER   FD   TYPE             DEVICE SIZE/OFF NODE NAME
node     1234 testuser   20u  IPv4 0x1234567890abcdef      0t0  TCP *:3000 (LISTEN)
nginx    5678 www-data   10u  IPv4 0x9876543210fedcba      0t0  TCP *:8080 (LISTEN)
"#;
    
    // Test filtering by specific port
    let result_3000 = parse_lsof_output(lsof_output, "3000");
    let result_8080 = parse_lsof_output(lsof_output, "8080");
    
    // Both should return all processes, but with different port labels
    assert_eq!(result_3000.len(), 2);
    assert_eq!(result_8080.len(), 2);
    
    // Verify port assignment
    assert_eq!(result_3000[0].port, "3000");
    assert_eq!(result_8080[0].port, "8080");
}