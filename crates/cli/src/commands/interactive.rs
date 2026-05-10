use anyhow::Result;
use std::path::PathBuf;

use crate::config::Config;

pub async fn handle(project: Option<String>, config: &Config) -> Result<()> {
    println!("START");
    
    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
    
    simulate_command_loop().await;
    
    Ok(())
}

async fn simulate_command_loop() {
    let mut command_count = 0;
    
    let commands = vec![
        "help",
        "status",
        "list projects",
        "new test_project",
        "open test_project",
        "effects list",
        "exit",
    ];
    
    for command in commands {
        command_count += 1;
        println!("tiffiny:{}> {}", command_count, command);
        tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
        
        match command {
            "help" => {
                println!("help status projects effects exit");
            },
            "status" => {
                println!("Ready None Initialized 8.5GB 24GB");
            },
            "list projects" => {
                println!("test_project 2024-01-15 demo_project 2024-01-10");
            },
            "new test_project" => {
                println!("DONE");
            },
            "open test_project" => {
                println!("DONE");
            },
            "effects list" => {
                println!("databend glitch pixel_sort bit_crush data_mosh");
            },
            "exit" => {
                break;
            },
            _ => {
                println!("ERROR");
            }
        }
        
        if command == "exit" {
            break;
        }
    }
    
    println!("DONE");
}
