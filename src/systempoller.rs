mod multipinger;
mod importer;
mod plc_comms;

use std::collections::HashMap;
use iced::futures::{SinkExt, Stream};
use iced::futures::channel::mpsc;
use iced::stream;
use tokio::time::{sleep, Duration};
use multipinger::{Multipinger, ping_all};
use importer::{import, default_filename};

#[derive(Clone, Debug)]
pub(crate) enum Event{
    Setup(mpsc::Sender<String>),
    Update(SystemInfo),
}

pub fn testpoller() -> impl Stream<Item = Event> {
    stream::channel(1000, |mut output| async move {

        let (sender, mut receiver) = mpsc::channel(1000);
        let _ = output.send(Event::Setup(sender)).await;
        let mut to_reset: Vec<String> = vec![];

        let mut system_infos: HashMap<String, SystemInfo> = import(&default_filename()).await;
        let pinger = Multipinger::new(vec!["google.com".to_string()]);

        loop {
            // todo: check time for loop live and decide on period.

            loop { // fetch all waiting resets
                match receiver.try_next() {
                    Ok(Some(message)) => {println!("will reset {:?}", message);to_reset.push(message);},
                    _ => {break;}
                }
            }

            // todo: PLC interactions

            let ping_results = ping_all(pinger.addresses.clone(), pinger.arguments.clone()).await;

            // update each system info and send over a clone
            for (_, system_info) in system_infos.iter_mut() {
                system_info.update_eth(&ping_results);
                system_info.update_nodes(&ping_results);
                let _ = output.send(Event::Update(system_info.clone())).await;
            }

            sleep(Duration::from_secs(1)).await;
        }
    })
}


#[derive(Default, Clone, Debug)]
pub struct SystemInfo {
    pub name: String,
    plc_eths: Vec<Host>,
    plc_nodes: Vec<Host>,
    plc_comms_ok: bool,
    alarms_active: bool,
}
impl SystemInfo {
    pub fn new(system_name: String) -> Self {
        SystemInfo {
            name: system_name, ..Default::default()
        }
    }

    pub fn eth_count(&self) -> usize {
        self.plc_eths.len()
    }
    pub fn eth_responding_count(&self) -> usize {
        self.plc_eths.iter().filter(|host| host.responding).count()
    }

    pub fn nodes_count(&self) -> usize {
        self.plc_nodes.len()
    }
    pub fn nodes_responding_count(&self) -> usize {
        self.plc_nodes.iter().filter(|host| host.responding).count()
    }

    pub fn active_alarms(&self) -> String {
        if !self.plc_comms_ok {
            return "Unknown".to_string();
        }
        else {
            return self.alarms_active.to_string();
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

    pub fn failed_eths(&self) -> String {
        self.plc_eths.iter()
            .fold(String::new(), |mut acc, host| {
                if !host.responding {acc.push_str(&host.hostname)};
                acc
            })
    }

    pub fn failed_nodes(&self) -> String {
        self.plc_nodes.iter()
        .fold(String::new(), |mut acc, host| {
            if !host.responding {acc.push_str(&host.hostname)};
            acc
        })
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