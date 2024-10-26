use tokio::time;
use sysinfo::{System};
use serde::{Deserialize};
use std::fs;


const ZWIFT_APP_NAME: &str = "ZwiftAppSilicon";
const HA_ENDPOINT: &str = "http://homeassistant.local:8123/";
const STATES_PATH: &str = "api/states/";
const SWITCH_PATH: &str = "api/services/switch/turn_";
const FAN_NAME: &str = "switch.kasa_plug_purple";

#[derive(Deserialize, Debug)]
struct HaEntity {
  entity_id: String,
  state: String,
}

async fn is_fan_on(ha_token: String) -> Result<bool, reqwest::Error>  {
    let ha_client = reqwest::Client::new();

    let url = format!("{HA_ENDPOINT}{STATES_PATH}{FAN_NAME}");

   println!("{}", format!("Bearer {ha_token}"));

    let resp = ha_client.get(url)
        .header("Authorization", format!("Bearer {ha_token}"))
        .header("Content-Type", "application/json")
        .send()
        .await?;

    let result = resp
        .json::<HaEntity>()
        .await?;
    
    println!("Fan is {}", result.state);
    Ok(&result.state == "on")
}

async fn set_state(state: &str, ha_token: String) {
    println!("setting state to {state}");
    let ha_client = reqwest::Client::new();

    let url = format!("{HA_ENDPOINT}{SWITCH_PATH}{state}");

    let resp = ha_client.post(url)
        .header("Authorization", format!("Bearer {ha_token}"))
        .header("Content-Type", "application/json")
        //.body(format!("{{\"state\":\"{state}\"}}"))
        .body(format!("{{\"entity_id\":\"{FAN_NAME}\"}}"))
        .send()
        .await;

    match resp {
        Ok(resp) => {
            println!("{:?}", resp);
            match resp.text().await {
                Ok(r) => println!("r: {:?}", r),
                Err(e) => println!("e: {:?}", e),
            }
        }
        Err(e) => println!("Failed to set fan state {:?}", e)
    }
}

async fn turn_on_fan(ha_token: String) {
    set_state("on", ha_token).await;
}

async fn turn_off_fan(ha_token: String) {
    set_state("off", ha_token).await;
}

async fn zwift_running() -> bool {
    let system = System::new_all();
    let zwift_procs = system.processes_by_exact_name(ZWIFT_APP_NAME.as_ref());
    if zwift_procs.count() > 0 {
       println!("zwift is running!");
       return true;
    } else {
       println!("zwift not running.");
       return false;
    }
}


#[tokio::main]
async fn main() {
    let ha_token = fs::read_to_string("/Users/aramishu1/ha_token.txt")
        .expect("ha_token.txt not found");
    let ha_token = ha_token.trim().to_string();
    println!("{}", ha_token);
    let mut interval = time::interval(time::Duration::from_secs(10));
    loop {
        interval.tick().await;
        let fan_on = is_fan_on(ha_token.clone()).await;
        match fan_on {
            Ok(fan_on) => {
                if zwift_running().await {
                    if !fan_on { turn_on_fan(ha_token.clone()).await };
                 } else {
                    if fan_on { turn_off_fan(ha_token.clone()).await };
                 }
            },
           Err(e) => {
                 println!("Failed to determint fan state {:?}", e);
            }
        } 
    }
}
