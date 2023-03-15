use std::collections::HashMap;
use serde_json::{Value};

pub struct Parceli {
    pub key: String,
}

#[derive(Debug)]
pub struct Parcel {
    pub tracking_number: String,
    pub courier_code: String,
    pub city: String,
    pub location: String,
    pub events: Vec<Events>,
}

#[derive(Debug)]
pub struct Events {
    pub status: String,
    pub location: String,
    pub datetime: String,
}

pub fn new(key: &String) -> Parceli {
    Parceli { 
        key: key.clone(),
    }
}

impl Parceli {
    fn get_value_by_path(&self, json_str: &str, path: &str) -> Option<Value> {
        let value: Value = serde_json::from_str(json_str).ok()?;
        let mut current = &value;
        for key in path.split('.') {
            if let Some(index) = key.parse::<usize>().ok() {
                current = current.get(index)?;
            } else {
                current = current.get(key)?;
            }
        }
        Some(current.clone())
    }

    fn get_vec_len_by_path(&self, json_str: &str, path: &str) -> Option<usize> {
        let value: Value = serde_json::from_str(json_str).ok().unwrap();
        let mut current = &value;
        for key in path.split('.') {
            if let Some(index) = key.parse::<usize>().ok() {
                current = current.get(index)?;
            } else {
                current = current.get(key)?;
            }
        }
        Some(current.as_array().unwrap().len())
    }

    fn uncuck_string(&self, str: String) -> String {
        str.trim_start_matches('"').trim_end_matches('"').replace("null", "").to_string()
    }

    pub fn track(&self, ids: Vec<String>) -> Vec<Parcel> {
        let mut parcels: Vec<Parcel> = Vec::new();
        for id in ids {
            let mut body = HashMap::new();
            body.insert("trackingNumber", id);
            
            let client = reqwest::blocking::Client::new();
            let res = client.post("https://api.ship24.com/public/v1/tracking/search")
                .json(&body)
                .bearer_auth(self.key.to_string())
                .header("Content-Type", "application/json; charset=utf-8")
                .send();

            let res = match res {
                Ok(res) => res,
                Err(_) => {
                    println!("Could not parse request");
                    break;
                }
            };

            let text = match res.text() {
                Ok(text) => text,
                Err(_) => {
                    println!("Could not get text from response");
                    break;
                }
            };

            let mut events: Vec<Events> = Vec::new();

            if let Some(len) = self.get_vec_len_by_path(&text, "data.trackings.0.events") {
                for num in 0..len {
                    let event = Events {
                        status: self.uncuck_string(self.get_value_by_path(&text, format!("data.trackings.0.events.{num}.status").as_str()).unwrap_or(Value::String(String::from(""))).to_string()),
                        location: self.uncuck_string(self.get_value_by_path(&text, format!("data.trackings.0.events.{num}.location").as_str()).unwrap_or(Value::String(String::from(""))).to_string()),
                        datetime: self.uncuck_string(self.get_value_by_path(&text, format!("data.trackings.0.events.{num}.datetime").as_str()).unwrap_or(Value::String(String::from(""))).to_string()),
                    };
                    events.push(event);
                }   
            }

            let parcel = Parcel {
                tracking_number: self.uncuck_string(self.get_value_by_path(&text, "data.trackings.0.events.0.trackingNumber").unwrap_or(Value::String(String::from(""))).to_string()),
                courier_code: self.uncuck_string(self.get_value_by_path(&text, "data.trackings.0.events.0.courierCode").unwrap_or(Value::String(String::from(""))).to_string()),
                city: self.uncuck_string(self.get_value_by_path(&text, "data.trackings.0.shipment.recipient.city").unwrap_or(Value::String(String::from(""))).to_string()),
                location: self.uncuck_string(self.get_value_by_path(&text, "data.trackings.0.events.0.location").unwrap_or(Value::String(String::from(""))).to_string()),
                events
            };
            
            // println!("{:?}", parcel);

            parcels.push(parcel);
        }
        parcels
    }
}