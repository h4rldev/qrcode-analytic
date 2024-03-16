use serde::{Deserialize, Serialize};
use chrono::prelude::*;

#[derive(Serialize, Deserialize, Clone)]
pub struct JsonData {
    pub state: Vec<JsonState>
}


#[derive(Serialize, Deserialize, Clone)]
pub struct JsonState {
    pub date: String,
    pub last_count: i32,
    pub last_time: String,
}

#[derive(Clone)]
pub struct AppData {
    pub state: Vec<AppState>
}

#[derive(Clone)]
pub struct AppState {
    pub last_date: String,
    pub date: String,
    pub counter: i32,
    pub time: String,
    pub last_time: String,
}

impl Default for JsonState {
    fn default() -> Self {
         JsonState { date: Local::now().date_naive().to_string(), last_count: 0_i32, last_time: Local::now().time().to_string()  }
    }
}

impl Default for JsonData {
    fn default() -> Self {
        JsonData { state: vec![JsonState::default()] }
    }
}
