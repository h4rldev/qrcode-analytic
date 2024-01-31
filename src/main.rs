use chrono::{prelude::*, Duration};
use ntex_session::CookieSession;
use ntex::web::{self, resource, App, Error as WebError, HttpRequest, HttpResponse, HttpServer};
use serde_json::{from_reader, to_writer};
use std::{
    fs::{create_dir, File},
    path::Path,
};
use serde::{Deserialize, Serialize};
use std::sync::{
    Arc, Mutex,
};

#[derive(Serialize, Deserialize)]
struct JsonState {
    last_count: i32,
    last_time: String,
}

struct AppState {
    time: String,
    counter: i32, // <- Mutex is necessary to mutate safely across threads
}

impl Default for JsonState {
    fn default() -> Self {
         JsonState { last_count: 0_i32, last_time: Local::now().to_rfc3339()  }
    }
}


async fn read_from_json(path: &Path) -> Result<JsonState, std::io::Error> {
    if ! path.is_dir() {
        create_dir(path)?;
    }
    let file_path = path.join("data.json");
    if ! file_path.is_file() {
        return Err(std::io::ErrorKind::NotFound.into());
    }
    let file = File::open(file_path)?;
    let data: JsonState = from_reader(file)?;
    Ok(data)    
}

async fn write_to_json(path: &Path, data: JsonState) -> Result<(), std::io::Error> {
    if ! path.is_dir() {
        create_dir(path)?;
    }
    let file_path = path.join("data.json");
    let file = File::create(file_path)?;
    to_writer(&file, &data)?;
    Ok(())
}


async fn index(
    data: web::types::State<Arc<Mutex<AppState>>>,
    session: ntex_session::Session
) -> Result<HttpResponse, WebError> {
    let mut data = data.try_lock().expect("poisoned_lock");
    let current_dir = std::env::current_dir()?;
    let state_path = current_dir.join("state");

    if session.get::<String>("session_time")?.is_some() {
        let time_since_last_visit: String = session.get("session_time")?.unwrap();
        let time_here = DateTime::parse_from_rfc3339(&time_since_last_visit).expect("Can't convert string to time");
        let time_difference = Local::now().signed_duration_since(&time_here);
        let return_in = (Duration::hours(22)-time_difference).num_hours();
        if time_difference < Duration::hours(22) {
            let forbidden_body = format!("Forbidden, return in: {}h", return_in);
            return Ok(HttpResponse::Forbidden().body(forbidden_body));
        } else {
            session.set("session_time", Local::now().to_rfc3339())?;
        }
    } else {
        session.set("session_time", Local::now().to_rfc3339())?;
    }

    let last_time = data.time.clone();
    data.time = Local::now().to_rfc3339();
    data.counter += 1; // <- access counter inside Mutex

    let new_data = JsonState {
        last_time: data.time.clone(),
        last_count: data.counter.clone(),
    };
    write_to_json(&state_path, new_data).await?;
     
    let body = format!("Hello world, you are visitor number: {} \nLast time: {} \nTime you visited: {}", data.counter, last_time, data.time);
    
    Ok(HttpResponse::Ok().body(body))
}

#[ntex::main]
async fn main() -> std::io::Result<()> {
    let current_dir = std::env::current_dir()?;
    let state_path = current_dir.join("state");

    let last_data = if read_from_json(&state_path).await.is_ok() {
        read_from_json(&state_path).await?
    } else {
        JsonState::default()
    };

    let last_count = last_data.last_count;
    let last_time = last_data.last_time;
    
    let state = Arc::new(Mutex::new(AppState {
        counter: last_count,
        time: last_time
    }));

    HttpServer::new(move || {
        App::new()
            .service(resource("/").to(index))
            .state(state.clone())
            .wrap(
                CookieSession::private(&[0; 32]).name("qrcode").secure(false)
            )
    }).bind("127.0.0.1:8080")?.run().await
}
