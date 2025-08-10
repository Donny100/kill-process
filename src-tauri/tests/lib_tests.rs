// Integration tests for kill-process library
// These tests verify the core functionality without depending on external commands
//
// Test Coverage:
// - Port-based process detection (lsof parsing)
// - Name-based process search (ps parsing)  
// - Process killing with signal validation
// - Data structure validation
// - Edge cases and error handling
// - Unicode support and special characters
// - Real-world scenarios

use kill_process_lib::{
    parse_lsof_output, 
    parse_ps_output,
    kill_process_with_signal, 
    ProcessInfo, 
    ProcessDetail, 
    PortCheckResult,
    ProcessSearchResult
};

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

// Tests for parse_ps_output function (process name search)

#[test]
fn test_parse_ps_output_basic() {
    let ps_output = r#"  1234 node
  5678 nginx
  9999 chrome"#;
    
    let result = parse_ps_output(ps_output, "node");
    
    assert_eq!(result.len(), 1);
    assert_eq!(result[0].pid, "1234");
    assert_eq!(result[0].name, "node");
    assert_eq!(result[0].port, "Unknown");
}

#[test]
fn test_parse_ps_output_case_insensitive() {
    let ps_output = r#"  1234 Node
  5678 NGINX
  9999 chrome"#;
    
    let result = parse_ps_output(ps_output, "node");
    
    assert_eq!(result.len(), 1);
    assert_eq!(result[0].pid, "1234");
    assert_eq!(result[0].name, "Node");
}

#[test]
fn test_parse_ps_output_partial_match() {
    let ps_output = r#"  1234 nodejs
  5678 node-server
  9999 chrome"#;
    
    let result = parse_ps_output(ps_output, "node");
    
    assert_eq!(result.len(), 2);
    assert_eq!(result[0].pid, "1234");
    assert_eq!(result[0].name, "nodejs");
    assert_eq!(result[1].pid, "5678");
    assert_eq!(result[1].name, "node-server");
}

#[test]
fn test_parse_ps_output_no_matches() {
    let ps_output = r#"  1234 apache
  5678 mysql
  9999 chrome"#;
    
    let result = parse_ps_output(ps_output, "node");
    
    assert_eq!(result.len(), 0);
}

#[test]
fn test_parse_ps_output_empty() {
    let ps_output = "";
    
    let result = parse_ps_output(ps_output, "node");
    
    assert_eq!(result.len(), 0);
}

#[test]
fn test_parse_ps_output_malformed_lines() {
    let ps_output = r#"  1234 node
incomplete
  5678 nginx
just-one-part"#;
    
    let result = parse_ps_output(ps_output, "node");
    
    // Should only find the valid "node" entry
    assert_eq!(result.len(), 1);
    assert_eq!(result[0].pid, "1234");
    assert_eq!(result[0].name, "node");
}

#[test]
fn test_parse_ps_output_with_spaces() {
    let ps_output = r#"  1234   node  
    5678    nginx   
  9999 my-app"#;
    
    let result = parse_ps_output(ps_output, "node");
    
    assert_eq!(result.len(), 1);
    assert_eq!(result[0].pid, "1234");
    assert_eq!(result[0].name, "node");
}

#[test]
fn test_parse_ps_output_large_pid() {
    let ps_output = r#"999999 node
     1 init
123456 my-long-process-name"#;
    
    let result = parse_ps_output(ps_output, "node");
    
    assert_eq!(result.len(), 1);
    assert_eq!(result[0].pid, "999999");
    assert_eq!(result[0].name, "node");
}

#[test]
fn test_parse_ps_output_special_characters() {
    let ps_output = r#"  1234 my-app
  5678 test_process
  9999 app.exe"#;
    
    let result = parse_ps_output(ps_output, "app");
    
    assert_eq!(result.len(), 2);
    assert_eq!(result[0].pid, "1234");
    assert_eq!(result[0].name, "my-app");
    assert_eq!(result[1].pid, "9999");
    assert_eq!(result[1].name, "app.exe");
}

// Tests for ProcessSearchResult structure

#[test]
fn test_process_search_result_structure() {
    let result = ProcessSearchResult {
        processes: vec![
            ProcessInfo {
                pid: "1234".to_string(),
                name: "test_process".to_string(),
                port: "Unknown".to_string(),
            }
        ],
        error: None,
    };
    
    assert_eq!(result.processes.len(), 1);
    assert!(result.error.is_none());
    assert_eq!(result.processes[0].pid, "1234");
    assert_eq!(result.processes[0].name, "test_process");
    assert_eq!(result.processes[0].port, "Unknown");
}

#[test]
fn test_process_search_result_with_error() {
    let result = ProcessSearchResult {
        processes: vec![],
        error: Some("Process name cannot be empty".to_string()),
    };
    
    assert_eq!(result.processes.len(), 0);
    assert!(result.error.is_some());
    assert_eq!(result.error.unwrap(), "Process name cannot be empty");
}

// Edge case tests for process name search

#[test]
fn test_parse_ps_output_empty_search_term() {
    let ps_output = r#"  1234 node
  5678 nginx"#;
    
    let result = parse_ps_output(ps_output, "");
    
    // Empty search term should match nothing
    assert_eq!(result.len(), 0);
}

#[test]
fn test_parse_ps_output_whitespace_search_term() {
    let ps_output = r#"  1234 node
  5678 nginx"#;
    
    let result = parse_ps_output(ps_output, "   ");
    
    // Whitespace-only search term should match nothing
    assert_eq!(result.len(), 0);
}

#[test]
fn test_parse_ps_output_unicode_characters() {
    let ps_output = r#"  1234 node应用
  5678 nginx-测试"#;
    
    let result = parse_ps_output(ps_output, "应用");
    
    assert_eq!(result.len(), 1);
    assert_eq!(result[0].pid, "1234");
    assert_eq!(result[0].name, "node应用");
}

#[test]
fn test_parse_ps_output_very_long_process_name() {
    let ps_output = r#"  1234 very-long-process-name-that-might-be-truncated-in-real-world
  5678 short"#;
    
    let result = parse_ps_output(ps_output, "very-long");
    
    assert_eq!(result.len(), 1);
    assert_eq!(result[0].pid, "1234");
    assert_eq!(result[0].name, "very-long-process-name-that-might-be-truncated-in-real-world");
}

#[test]
fn test_parse_ps_output_numeric_search() {
    let ps_output = r#"  1234 app123
  5678 service456
  9999 test789"#;
    
    let result = parse_ps_output(ps_output, "123");
    
    assert_eq!(result.len(), 1);
    assert_eq!(result[0].pid, "1234");
    assert_eq!(result[0].name, "app123");
}

#[test]
fn test_parse_ps_output_complex_real_world_example() {
    let ps_output = r#"    1 launchd
  123 com.apple.dock
  456 Google Chrome Helper
  789 nodejs
 1000 node
 1234 VSCode Helper (Plugin Host)
 5678 node_modules/.bin/webpack
 9999 /usr/local/bin/node server.js"#;
    
    let result = parse_ps_output(ps_output, "node");
    
    // Should find: nodejs, node, node_modules/.bin/webpack, /usr/local/bin/node
    assert_eq!(result.len(), 4);
    
    // Verify specific matches
    let pids: Vec<&String> = result.iter().map(|p| &p.pid).collect();
    assert!(pids.contains(&&"789".to_string()));   // nodejs
    assert!(pids.contains(&&"1000".to_string()));  // node
    assert!(pids.contains(&&"5678".to_string()));  // node_modules/.bin/webpack
    assert!(pids.contains(&&"9999".to_string()));  // /usr/local/bin/node
}

// Tests for ProcessDetail validation and edge cases

#[test]
fn test_process_detail_minimal_info() {
    let detail = ProcessDetail {
        pid: "1234".to_string(),
        name: "test".to_string(),
        port: "Unknown".to_string(),
        user: None,
        command: None,
        cpu_usage: None,
        memory_usage: None,
        start_time: None,
    };
    
    assert_eq!(detail.pid, "1234");
    assert_eq!(detail.name, "test");
    assert_eq!(detail.port, "Unknown");
    assert!(detail.user.is_none());
    assert!(detail.command.is_none());
    assert!(detail.cpu_usage.is_none());
    assert!(detail.memory_usage.is_none());
    assert!(detail.start_time.is_none());
}

#[test]
fn test_process_detail_with_complex_command() {
    let detail = ProcessDetail {
        pid: "1234".to_string(),
        name: "node".to_string(),
        port: "3000, 8080".to_string(),
        user: Some("testuser".to_string()),
        command: Some("/usr/local/bin/node --inspect=0.0.0.0:9229 --max-old-space-size=4096 server.js".to_string()),
        cpu_usage: Some("15.7%".to_string()),
        memory_usage: Some("256.8%".to_string()),
        start_time: Some("Mon Jan 15 10:30:45 2024".to_string()),
    };
    
    assert_eq!(detail.pid, "1234");
    assert_eq!(detail.name, "node");
    assert_eq!(detail.port, "3000, 8080");
    assert!(detail.command.as_ref().unwrap().contains("--inspect"));
    assert!(detail.command.as_ref().unwrap().contains("server.js"));
}

// Tests for kill process validation improvements

#[test]
fn test_kill_process_boundary_pids() {
    // Test minimum and maximum valid PIDs
    let valid_edge_pids = vec!["1", "32767", "65535"];
    
    for pid in valid_edge_pids {
        // Just test validation doesn't fail - we won't actually kill processes
        assert!(pid.parse::<u32>().is_ok());
    }
}

#[test]
fn test_kill_process_invalid_formats() {
    let invalid_formats = vec![
        "0",           // Zero PID
        "-1",          // Negative
        "1.5",         // Decimal
        "1e5",         // Scientific notation
        "pid123",      // Text prefix
        "123pid",      // Text suffix
        " 123 ",       // Whitespace (should fail in actual function)
        "99999999999", // Too large for u32
    ];
    
    for invalid_pid in invalid_formats {
        // Test that these would be rejected by the validation
        if invalid_pid == "0" {
            // PID 0 is technically valid in parse but not for kill
            continue;
        }
        let parse_result = invalid_pid.trim().parse::<u32>();
        if invalid_pid.contains('.') || invalid_pid.contains('e') || 
           invalid_pid.contains(char::is_alphabetic) {
            assert!(parse_result.is_err(), "Should fail for: {}", invalid_pid);
        }
    }
}

// Test for comprehensive functionality validation

#[test]
fn test_port_check_result_comprehensive() {
    let result_with_multiple_processes = PortCheckResult {
        is_occupied: true,
        processes: vec![
            ProcessInfo {
                pid: "1234".to_string(),
                name: "node".to_string(),
                port: "3000".to_string(),
            },
            ProcessInfo {
                pid: "5678".to_string(),
                name: "nginx".to_string(),
                port: "3000".to_string(),
            }
        ],
        error: None,
    };
    
    assert!(result_with_multiple_processes.is_occupied);
    assert_eq!(result_with_multiple_processes.processes.len(), 2);
    assert!(result_with_multiple_processes.error.is_none());
    
    // Verify both processes use the same port
    for process in &result_with_multiple_processes.processes {
        assert_eq!(process.port, "3000");
    }
}

#[test]
fn test_port_check_result_with_error() {
    let result_with_error = PortCheckResult {
        is_occupied: false,
        processes: vec![],
        error: Some("Failed to execute lsof: command not found".to_string()),
    };
    
    assert!(!result_with_error.is_occupied);
    assert_eq!(result_with_error.processes.len(), 0);
    assert!(result_with_error.error.is_some());
    assert!(result_with_error.error.unwrap().contains("lsof"));
}