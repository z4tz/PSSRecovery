use std::collections::HashMap;
use std::env;
use crate::systempoller::{Host, SystemInfo};

use tokio::fs;

pub async fn import(filename: &str) -> HashMap<String, SystemInfo> {
    let mut system_infos = HashMap::new();
    let contents = fs::read_to_string(filename).await.expect(&format!("Error reading file \"{filename}\""));
    for line in contents.lines() {
        let parts = line.split(",").collect::<Vec<&str>>();
        if parts.len() != 2 {  // if invalid line, skip it.
            continue;
        }
        let hostname = parts[0].to_string();
        let ip_address = parts[1].to_string();
        let system_name = hostname.split("_").collect::<Vec<&str>>()[0].to_string();
        let system_info = system_infos.entry(system_name.clone()).or_insert(SystemInfo::new(system_name));
        if hostname.to_lowercase().contains("eth") {
            system_info.add_eth(Host::new(hostname, ip_address));
        }
        else {
            system_info.add_node(Host::new(hostname, ip_address));
        }
    }
    system_infos
}

pub fn default_filename() -> String {
    match env::consts::OS {
        "windows" => {"ip_adresses.txt".to_string()}
        "linux" => {"ip_adresses.txt".to_string()}
        "macos" => {"ip_adresses.txt".to_string()}
        &_ => {"ip_adresses.txt".to_string()}
    }
}