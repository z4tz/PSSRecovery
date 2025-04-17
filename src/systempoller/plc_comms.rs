use rseip::client::ab_eip::*;
use rseip::precludes::*;


enum State{
    Disconnected,
    Connected(ab_eip::AbEipClient),
}

struct PLC {
    state: State,
    ip: String,
    reset_man_tag: EPath,
    reset_auto_tag: EPath,
    active_alarms_tag: EPath,
}
impl PLC {
    pub fn new(ip: String, system_name: String) -> Self {
        PLC {ip,
            reset_man_tag: EPath::parse_tag( format!("B_{}_reset_man_c", system_name)).unwrap(),
            reset_auto_tag: EPath::parse_tag( format!("B_{}_reset_auto_c", system_name)).unwrap(),
            state: State::Disconnected,
            active_alarms_tag: EPath::parse_tag( format!("B_{}_active_alarms_hb", system_name)).unwrap()
        }
    }

    async fn try_connect(&mut self)  {
        match AbEipClient::new_host_lookup(self.ip.clone()).await {
            Ok(client) => {
                self.state = State::Connected(client.with_connection_path(PortSegment::default()))
            }
            Err(_) => {}
        }
    }

    pub async fn try_reset(mut self) {
        if let State::Disconnected = self.state {self.try_connect().await;}
        match self.state {
            State::Disconnected => {}
            State::Connected(mut client) => {
                let _ = client.write_tag(self.reset_auto_tag.clone(),true).await;
                let _ = client.write_tag(self.reset_auto_tag.clone(), true).await;
            }
        }
    }

    pub async fn alarms_active(mut self) -> Option<bool> {
        if let State::Disconnected = self.state {self.try_connect().await;}
        match self.state {
            State::Disconnected => {None}
            State::Connected(mut client) => {
                match client.read_tag::<EPath, bool>(self.active_alarms_tag.clone()).await {
                    Ok(result) => {Some(result)}
                    Err(_) => {None}
                }
            }
        }
    }
}