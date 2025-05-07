use std::collections::HashMap;
use std::env;

use tokio::process::{Command};
use tokio::task::JoinSet;

pub struct Multipinger{
    pub addresses:Vec<String>,
    pub arguments: Vec<String>,
}
impl Multipinger {
    pub fn new(addresses:Vec<String>) -> Multipinger {
        match env::consts::OS {
            "windows" => {Multipinger {addresses, arguments: vec!["-n".to_string(), "2".to_string(), "-w".to_string(), "1000".to_string()]}},
            "linux" =>   {Multipinger {addresses, arguments: vec!["-c".to_string(), "2".to_string(), "-W".to_string(), "1".to_string()]}},
            "macos" =>   {Multipinger {addresses, arguments: vec!["-c".to_string(), "2".to_string(), "-t".to_string(), "1".to_string()]}},
            _ =>         {Multipinger {addresses, arguments: vec!["-c".to_string(), "2".to_string(), "-W".to_string(), "1".to_string()]}},

        }
    }

    pub async fn ping_all(&self) -> HashMap<String, bool>{
        let mut set = JoinSet::new();
        for address in self.addresses.clone() {
            let argument_clone = self.arguments.clone();
            set.spawn(async move {execute_ping(address, argument_clone).await});
        }
        let mut map = HashMap::new();
        while let Some(res) = set.join_next().await{
            match res {
                Ok((address,result)) => {map.insert(address, result);},
                Err(_) => {}
            }
        }
        map
    }
}


async fn execute_ping(target: String, mut arguments: Vec<String>) -> (String, bool) {
    let mut cmd = Command::new("ping");
    arguments.push(target.clone());
    cmd.args(arguments);
    let res =cmd.output().await.unwrap();
    (target, res.status.success())
}
