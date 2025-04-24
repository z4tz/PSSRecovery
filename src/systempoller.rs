mod multipinger;
pub mod importer;
mod plc_comms;

use std::collections::HashMap;
use iced::futures::{SinkExt, Stream};
use iced::futures::channel::mpsc;
use iced::stream;
use tokio::time::{sleep, Duration, Instant};
use multipinger::{Multipinger};
use importer::{import, get_filename};
use plc_comms::{read_and_reset};

#[derive(Clone, Debug)]
pub enum Event{
    Setup(mpsc::Sender<String>),
    Update(SystemInfo),
}

pub fn testpoller() -> impl Stream<Item = Event> {
    stream::channel(1000, |mut output| async move {

        let (sender, mut receiver) = mpsc::channel(1000);
        let _ = output.send(Event::Setup(sender)).await;
        let mut to_reset: Vec<String> = vec![];

        let mut system_infos: HashMap<String, SystemInfo> = import(&get_filename()).await;

        let addresses_to_ping: Vec<String> = system_infos.values()
            .map(|sys| sys.get_addresses()).flatten().collect();
        let pinger = Multipinger::new(addresses_to_ping);

        loop {
            let start = Instant::now();

            loop { // fetch all waiting resets
                match receiver.try_next() {
                    Ok(Some(message)) => {to_reset.push(message);},
                    _ => {break;}
                }
            }

            let ping_results = pinger.ping_all().await;
            // update each system info
            for (_, system_info) in system_infos.iter_mut() {
                system_info.update_eth(&ping_results);
                system_info.update_nodes(&ping_results);
            }
            
            let plc_interactions: Vec<(String, String, bool)> = system_infos.iter()
                .map(|(name, sys)| (name.to_string(), sys.get_eth_address(), to_reset.contains(&sys.name)))
                .collect();
            
            let plc_results = read_and_reset(plc_interactions).await;
            for (system_name, res) in plc_results {
                system_infos.get_mut(&system_name).unwrap().alarms_active = res;
            }
            
            // Send updated clone to GUI
            for (_, system_info) in system_infos.iter_mut() {
                let _ = output.send(Event::Update(system_info.clone())).await;
            }
            
            to_reset.clear();
            
            let elapsed = start.elapsed();
            println!("Scan took {elapsed:?}");
            if elapsed < Duration::from_millis(3000) {
                sleep(Duration::from_millis(2000)).await;    
            }
        }
    })
}


#[derive(Default, Clone, Debug)]
pub struct SystemInfo {
    pub name: String,
    plc_eths: Vec<Host>,
    plc_nodes: Vec<Host>,
    alarms_active: Option<bool>,
}
impl SystemInfo {
    // "backend methods
    pub fn new(system_name: String) -> Self {
        SystemInfo {
            name: system_name, ..Default::default()
        }
    }
    pub fn add_eth(&mut self, host: Host) {
        self.plc_eths.push(host);
    }

    pub fn add_node(&mut self, host: Host) {
        self.plc_nodes.push(host);
    }

    pub fn update_eth(&mut self, responses: &HashMap<String,bool>) {
        for host in self.plc_eths.iter_mut() {
            host.responding = responses[&host.ip_address];
        }
    }
    pub fn update_nodes(&mut self, responses: &HashMap<String,bool>) {
        for host in self.plc_nodes.iter_mut() {
            host.responding = responses[&host.ip_address];
        }
    }

    pub fn get_addresses(&self) -> Vec<String> {
        let mut addresses = vec![];
        for host in self.plc_eths.iter() {
            addresses.push(host.ip_address.to_string());
        }
        for host in self.plc_nodes.iter() {
            addresses.push(host.ip_address.to_string());
        }
        addresses
    }

    pub fn get_eth_address(&self) -> String {
        // return first responding eth
        for host in self.plc_eths.iter() {
            if host.responding {
                return host.ip_address.to_string();
            }
        }
        // if none are responsive return first
        self.plc_eths.first().unwrap().ip_address.to_string()
    }
    
    // "front end" methods
    pub fn eth_status(&self) -> String {
        format!("{}/{}", self.plc_eths.iter().filter(|host| host.responding).count(), self.plc_eths.len())
    }
    
    pub fn nodes_status(&self) -> String {
        format!("{}/{}", self.plc_nodes.iter().filter(|host| host.responding).count(), self.plc_nodes.len())
    }
    
    pub fn active_alarms(&self) -> Option<bool> {
        self.alarms_active
    }
    
    pub fn failed_hosts(&self) -> String {
        let mut failed_hosts = vec![];
        failed_hosts.extend(self.plc_eths.iter().filter(|host| !host.responding));
        failed_hosts.extend(self.plc_nodes.iter().filter(|host| !host.responding));
        failed_hosts.into_iter()
            .map(|host| host.hostname.to_string())
            .collect::<Vec<String>>()
            .join("\n")
    }
    
    pub fn eths_ok(&self) -> bool {
        self.plc_eths.iter().all(|host| host.responding)
    }
    pub fn nodes_ok(&self) -> bool {
        self.plc_nodes.iter().all(|host| host.responding)
    }
    
}

#[derive(Clone, Debug)]
pub struct Host {
    hostname: String,
    ip_address: String,
    responding: bool,
}
impl Host {
    pub fn new(hostname: String, ip_address: String) -> Self {
        Host {hostname, ip_address, responding: false}
    }
}