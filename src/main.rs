use chrono::{prelude::*, Duration};
use tokio::sync::Mutex;
use ntex_session::CookieSession;
use ntex::web::{self, get, App, Error as WebError, HttpRequest, HttpResponse, HttpServer};
use serde_json::{from_reader, to_writer};
use core::panic;
use std::{
    fs::{create_dir, File},
    path::Path,
};
#[allow(unused_imports)]
use tracing::{info, debug, trace, error, warn};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use http::{index, files, get_from_subdir};

mod http;

#[derive(Serialize, Deserialize, Clone)]
struct JsonData {
    state: Vec<JsonState>
}


#[derive(Serialize, Deserialize, Clone)]
struct JsonState {
    date: String,
    last_count: i32,
    last_time: String,
}

#[derive(Clone)]
struct AppData {
    state: Vec<AppState>
}

#[derive(Clone)]
struct AppState {
    date: String,
    counter: i32,
    time: String,
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

async fn can_user_enter(session: ntex_session::Session) -> Result<bool, WebError> {
    if session.get::<String>("session_time")?.is_some() {
        let time_since_last_visit: String = session.get("session_time")?.expect("Can't get cookie despite being some.");
        let time_here = DateTime::parse_from_rfc3339(&time_since_last_visit).expect("Can't parse from rfc3339");
        let time_difference = Local::now().signed_duration_since(&time_here);
        if time_difference < Duration::try_hours(22).expect("Can't get hours") {
            return Ok(false);
        }
    } else {
        session.set("session_time", Local::now().to_rfc3339())?;
    }
    Ok(true)
}


async fn read_from_json(path: &Path) -> Result<JsonData, std::io::Error> {
    if ! path.is_dir() {
        create_dir(path)?;
    }
    let file_path = path.join("data.json");
    if ! file_path.is_file() {
        return Err(std::io::ErrorKind::NotFound.into());
    }
    let file = File::open(file_path)?;
    let json_data: JsonData = from_reader(file)?;
    Ok(json_data)    
}

async fn write_to_json(path: &Path, json_data: JsonData) -> Result<(), std::io::Error> {
    if ! path.is_dir() {
        create_dir(path)?;
    }
    let file_path = path.join("data.json");
    let file = File::create(file_path)?;
    to_writer(&file, &json_data)?;
    Ok(())
}


#[get("/header")]
async fn header(session: ntex_session::Session, data: web::types::State<Arc<Mutex<AppData>>>, req: HttpRequest) -> Result<HttpResponse, WebError> {
    
    if req.headers().get("HX-Request").is_none() {
        return Ok(HttpResponse::Forbidden().finish());
    }

    let data = &data.try_lock().expect("poisoned_lock").state;

    let response = if can_user_enter(session).await? {
        format!("Success, you can now close this page, or check out other data below.")
    } else {
        format!("You've already checked in, come back tomorrow!")
    };

    let json_data = JsonData { 
        state: Vec::new(),
    };
    let mut json_state = json_data.state; 

    for entry in data {
        json_state.push(JsonState {
            date: entry.date.clone(),
            last_count: entry.counter,
            last_time: entry.time.clone()
        })
    }

    let json_data = JsonData {
        state: json_state
    };

    let path = Path::new("./state");

    write_to_json(&path, json_data).await?;

    Ok(HttpResponse::Ok().content_type("text/html").body(response))
}


#[get("/visitor")]
async fn visitor(data: web::types::State<Arc<Mutex<AppData>>>, session: ntex_session::Session, req: HttpRequest) -> Result<HttpResponse, WebError> {
    
    if req.headers().get("HX-Request").is_none() {
        return Ok(HttpResponse::Forbidden().finish());
    }

    if ! can_user_enter(session).await? {
        return Ok(HttpResponse::Ok().finish());
    }

    let mut data = data.try_lock().expect("poisoned_lock");
    let current_date = Local::now().date_naive().to_string();
    
    // Correctly handle the scope of `entry`
    let entry_index = data.state.iter().position(|e| e.date == current_date);

    let visitor = match entry_index {
        Some(index_of_entry) => {
            // Correctly access the entry
            let entry = &mut data.state[index_of_entry];
            entry.counter += 1;
            entry.time = Local::now().time().to_string();
            Some(entry.counter)
        },
        None => {
            // Create a new entry if it doesn't exist
            data.state.push(AppState {
                date: current_date,
                counter: 1, // Assuming the first visitor of the day starts with a counter of 1
                time: Local::now().time().to_string(),
            });
            Some(1)
        },
    };

    let visitor = if let Some(visitor) = visitor {
        visitor
    } else {
        panic!("visitor number missing");
    };

    let response = format!("You're visitor no. {}", visitor);
    Ok(HttpResponse::Ok().content_type("text/html").body(response))
}

#[get("/last_time")]
async fn lasttime(data: web::types::State<Arc<Mutex<AppData>>>, req: HttpRequest) -> Result<HttpResponse, WebError> {
    
    if req.headers().get("HX-Request").is_none() {
        return Ok(HttpResponse::Forbidden().finish());
    }

    let data = &data.try_lock().expect("poisoned_lock").state;
    let current_data = data.last().expect("Can't get latest entry");
    let time = &current_data.time;
    let date = &current_data.date;
    
    let response = format!("Last time someone checked in: {} {}", date, time);
    Ok(HttpResponse::Ok().content_type("text/html").body(response))
}

#[get("/your_time")]
async fn yourtime(data: web::types::State<Arc<Mutex<AppData>>>, session: ntex_session::Session, req: HttpRequest) -> Result<HttpResponse, WebError> {

    if req.headers().get("HX-Request").is_none() {
        return Ok(HttpResponse::Forbidden().finish());
    }

    if ! can_user_enter(session).await? {
        return Ok(HttpResponse::Ok().finish());
    }

    let mut data = data.try_lock().expect("poisoned_lock");
    let current_date = Local::now().date_naive().to_string();
    let date_matches = data.state.first().map_or(false, |entry| entry.date == current_date);

    let (date, time) = match date_matches {
        true => {
            // If the date matches, update the time of the first entry
            let entry = data.state.first_mut().expect("State is empty");
            entry.time = Local::now().time().to_string();
            (entry.date.clone(), entry.time.clone())
        },
        false => {
            // If the date doesn't match, create a new entry for the current date
            data.state.push(AppState {
                date: current_date,
                counter: 1, // Assuming the first visitor of the day starts with a counter of 1
                time: Local::now().time().to_string(),
            });
            let current_date = Local::now().date_naive().to_string();
            let current_time = Local::now().time().to_string();
            (current_date, current_time)
        },
    };
    // Update the time
    let response = format!("Time you checked in: {} {}", &date, &time);
    Ok(HttpResponse::Ok().content_type("text/html").body(response))    
}

#[ntex::main]
async fn main() -> std::io::Result<()> {
    tracing_subscriber::fmt::init();
    let current_dir = std::env::current_dir()?;
    let state_path = current_dir.join("state");

    let last_data = if read_from_json(&state_path).await.is_ok() {
        read_from_json(&state_path).await?.state
    } else {
        JsonData::default().state
    };

    
    let app_data = AppData { 
        state: Vec::new(),
    };
    let mut app_state = app_data.state; 

    for entry in last_data {
        app_state.push(AppState {
            date: entry.date,
            counter: entry.last_count,
            time: entry.last_time
        })
    }

    let state = Arc::new(Mutex::new(AppData {
        state: app_state
    }));


    HttpServer::new(move || {
        App::new()
            .wrap(web::middleware::Compress::default())
            .service(header)
            .service(index)
            .service(
                web::scope("/body")
                    .service(visitor)
                    .service(lasttime)
                    .service(yourtime)
            )
            .service(get_from_subdir)
            .route("/{filename}*", get().to(files))
            .state(state.clone())
            .wrap(
                CookieSession::private(&[0; 32]).name("qrcode").secure(false)
            )
            .wrap(
                ntex::web::middleware::Logger::default()
            )
    }).bind("127.0.0.1:8080")?.run().await
}
