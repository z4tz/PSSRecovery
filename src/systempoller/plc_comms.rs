use std::collections::HashMap;
use std::time::Duration;
use rseip::client::ab_eip::*;
use rseip::ClientError;
use rseip::precludes::*;
use tokio::task::JoinSet;
use tokio::time::sleep;

pub async fn read_and_reset(plc_infos: Vec<(String, String, bool)>) -> HashMap<String, Option<bool>> {
    let mut map = HashMap::from_iter(plc_infos.iter().map(|(name,_,_)|(name.to_string(), None)));
    let mut set = JoinSet::new();
    for (system_name, ip_address, reset) in plc_infos {
        set.spawn(async move {alarms_active(system_name, &ip_address, reset).await});
    }
    
    // ugly, but timeout on rseip calls are ~20s, instead we break after a reasonable duration
    sleep(Duration::from_millis(1000)).await;
    set.abort_all();
    while let Some(tokio_result) = set.join_next().await{
        match tokio_result {
            Ok(plc_result) => {
                match plc_result {
                    Ok((system_name, value)) => {
                        map.insert(system_name, Some(value));        
                    }
                    Err(_) => {}
                }
            },
            Err(_) => {}
        }
    }
    map
}


async fn alarms_active(system_name: String, ip_address: &str, reset: bool) -> Result<(String, bool),ClientError> {
    match AbEipClient::new_host_lookup(ip_address).await {
        Ok(client) => {
            let mut client = client.with_connection_path(PortSegment::default());

            let tag = EPath::parse_tag(format!("B_{}_SumAlarm_hb", system_name)).unwrap();
            
            let res: TagValue<bool> = client.read_tag(tag.clone()).await?;  // exit early if error

            if reset {  // if read failed don't try to reset
                let auto_reset = EPath::parse_tag(format!("B_{}_Alarm_Reset_Auto_C", system_name)).unwrap();
                let man_reset = EPath::parse_tag(format!("B_{}_Alarm_Reset_Man_C", system_name)).unwrap();

                let value = TagValue {
                    tag_type: TagType::Bool,
                    value: true,
                };
                let _ = client.write_tag(man_reset, value.clone()).await;
                let _ = client.write_tag(auto_reset, value).await;
            }
            let _ = client.close().await;
            Ok((system_name, res.value))
        }
        Err(err) => {Err(err.into()) },
    }
}