use serde_json::Value;
use std::collections::HashMap;


/// Main library API
pub struct Parceli {
    pub key: String,
    pub verbose: bool,
}

/// The parcel data type is used to store useful information about a package.
/// TODO: There needs to be options everywhere instead of empty strings
#[derive(Debug)]
pub struct Parcel {
    pub tracking_number: String,
    pub courier_code: String,
    pub city: String,
    pub location: String,
    pub events: Vec<Events>,
}

/// A sub component of Parcel, used for package history
#[derive(Debug)]
pub struct Events {
    pub status: String,
    pub location: String,
    pub datetime: String,
}

/// Creates a new instace of Parceli
pub fn new(key: &String, verbose: bool) -> Parceli {
    Parceli {
        key: key.clone().replace(char::is_whitespace, ""),
        verbose,
    }
}

/// Implementation for Parceli
impl Parceli {
    /// Takes in a json string, and gets an optional serde_json::Value back
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

    /// Takes in a json string, and gets an optional usize back for a Vec
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

    // Takes in a list of package ids, and returns the parcels data
    pub fn track(&self, ids: Vec<String>) -> Option<Vec<Parcel>> {
        let mut parcels: Vec<Parcel> = Vec::new();
        for id in ids {
            let mut body = HashMap::new();
            body.insert("trackingNumber", &id);

            let client = reqwest::blocking::Client::new();
            let res = client
                .post("https://api.ship24.com/public/v1/tracking/search")
                .json(&body)
                .bearer_auth(self.key.to_string())
                .header("Content-Type", "application/json; charset=utf-8")
                .send()
                .expect("Could not parse request");

            if self.verbose {
                println!("api status: {}", res.status().as_u16());
                println!("headers:");
                for (k, v) in res.headers() {
                    println!("\t{k}: {v:?}");
                }
            }

            if res.status().as_u16() != 200 && res.status().as_u16() != 201 {
                panic!("could not fetch parcel, your key may be invalid");
            }

            let text = res.text().expect("could not get text from response");

            let mut events: Vec<Events> = Vec::new();

            if let Some(len) = self.get_vec_len_by_path(&text, "data.trackings.0.events") {
                for num in 0..len {
                    let event = Events {
                        status: self
                            .get_value_by_path(
                                &text,
                                format!("data.trackings.0.events.{num}.status").as_str(),
                            )
                            .unwrap_or(Value::String(String::from("")))
                            .as_str()
                            .unwrap_or("")
                            .to_string(),
                        location: self
                            .get_value_by_path(
                                &text,
                                format!("data.trackings.0.events.{num}.location").as_str(),
                            )
                            .unwrap_or(Value::String(String::from("")))
                            .as_str()
                            .unwrap_or("")
                            .to_string(),
                        datetime: self
                            .get_value_by_path(
                                &text,
                                format!("data.trackings.0.events.{num}.datetime").as_str(),
                            )
                            .unwrap_or(Value::String(String::from("")))
                            .as_str()
                            .unwrap_or("")
                            .to_string(),
                    };
                    events.push(event);
                }
            }

            let parcel = Parcel {
                tracking_number: self
                    .get_value_by_path(&text, "data.trackings.0.events.0.trackingNumber")
                    .unwrap_or(Value::String(String::from("")))
                    .as_str()
                    .unwrap_or("")
                    .to_string(),

                courier_code: self
                    .get_value_by_path(&text, "data.trackings.0.events.0.courierCode")
                    .unwrap_or(Value::String(String::from("")))
                    .as_str()
                    .unwrap_or("")
                    .to_string(),

                city: self
                    .get_value_by_path(&text, "data.trackings.0.shipment.recipient.city")
                    .unwrap_or(Value::String(String::from("")))
                    .as_str()
                    .unwrap_or("")
                    .to_string(),

                location: self
                    .get_value_by_path(&text, "data.trackings.0.events.0.location")
                    .unwrap_or(Value::String(String::from("")))
                    .as_str()
                    .unwrap_or("")
                    .to_string(),

                events,
            };

            if self.verbose {
                println!("got parcel with id {} from api", &id);
            }

            parcels.push(parcel);
        }
        if parcels.len() == 0 {
            return None;
        }
        Some(parcels)
    }
}
