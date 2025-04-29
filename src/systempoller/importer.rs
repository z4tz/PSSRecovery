use std::collections::HashMap;
use crate::systempoller::{Host, SystemInfo};

use tokio::fs;

pub async fn import(filename: &str) -> Result<HashMap<String, SystemInfo>, String> {
    let mut system_infos = HashMap::new();
    match fs::read_to_string(filename).await {
        Err(e) => {
            Err(e.to_string())
        }
        Ok(contents) => {
            for line in contents.lines() {
                let parts = line.split(",").collect::<Vec<&str>>();
                if parts.len() != 2 {  // if invalid line, skip it.
                    continue;
                }
                let hostname = parts[0].trim().to_string();
                let ip_address = parts[1].trim().to_string();
                let system_name = hostname.split("_").collect::<Vec<&str>>()[0].to_string();
                let system_info = system_infos.entry(system_name.clone()).or_insert(SystemInfo::new(system_name));
                if hostname.to_lowercase().contains("eth") {
                    system_info.add_eth(Host::new(hostname, ip_address));
                }
                else {
                    system_info.add_node(Host::new(hostname, ip_address));
                }
            }
            Ok(system_infos)
        }
    }
}