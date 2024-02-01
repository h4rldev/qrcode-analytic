use chrono::{prelude::*, Duration};
use ntex_session::CookieSession;
use ntex::web::{self, get, resource, App, Error as WebError, HttpRequest, HttpResponse, HttpServer};
use serde_json::{from_reader, to_writer};
use std::{
    fs::{create_dir, File},
    path::Path,
};
#[allow(unused_imports)]
use tracing::{info, debug, trace, error, warn};
use serde::{Deserialize, Serialize};
use std::sync::{
    Arc, Mutex,
};

use http::{index, files, get_from_subdir};

mod http;

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
    let json_data: JsonState = from_reader(file)?;
    Ok(json_data)    
}

async fn write_to_json(path: &Path, json_data: JsonState) -> Result<(), std::io::Error> {
    if ! path.is_dir() {
        create_dir(path)?;
    }
    let file_path = path.join("data.json");
    let file = File::create(file_path)?;
    to_writer(&file, &json_data)?;
    Ok(())
}


#[get("/header")]
async fn header() -> Result<HttpResponse, WebError> {
    Ok(HttpResponse::Ok().content_type("text/html").body("gam"))
}


#[get("/body")]
async fn data(
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
            let forbidden_body = format!("<h1 class=\"problem\"> Du har redan skannat idag, kom tillbaks om: {} timmar </h1>", return_in);
            return Ok(HttpResponse::Ok().content_type("text/html").body(forbidden_body));
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
    let body = format!("<p class=\"person\"> Du är person nummer {}! </p>\n<p class=\"last_time\"> Sista skann: {} </p>\n<p class=\"your_time\"> Tid när du skanna: {}</p>", data.counter, last_time, data.time);
    Ok(HttpResponse::Ok().content_type("text/html").body(body))
}

#[ntex::main]
async fn main() -> std::io::Result<()> {
    tracing_subscriber::fmt::init();
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
            .service(header)
            .service(index)
            .service(data)
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
