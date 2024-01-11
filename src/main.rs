use ntex::{web::{self, App, HttpResponse, HttpServer, resource, Error as WebError, HttpRequest}, server};
use std::sync::{Mutex, atomic::{AtomicBool, Ordering}, Arc};
use std::fs::read_to_string;
use std::path::{PathBuf, Path};
use std::thread;
use std::time::Duration;
use chrono::prelude::*;
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
enum IpAddrKind {
    V4,
    V6,
}

#[derive(Serialize, Deserialize)]
struct JsonState {
    last_ip: String,
    kind: IpAddrKind,
    last_count: i32,
    last_time: String,
}

struct AppStateWithCounter {
    time: Mutex<String>,
    counter: Mutex<i32>, // <- Mutex is necessary to mutate safely across threads
}

async fn write_to_json(data: web::types::State<AppStateWithCounter>) {
    todo!()
}

async fn index(data: web::types::State<AppStateWithCounter>, req: HttpRequest) -> Result<HttpResponse, WebError> {
    let mut counter = data.counter.try_lock().expect("poisoned lock");
    let mut time = data.time.try_lock().expect("poisoned lock");
    let ip = if req.connection_info().remote().is_some() {
        req.connection_info().remote().unwrap().to_string()
    } else {
        "Could not get IP".to_string()
    };
    let last_time = time.clone();
    *time = Local::now().to_rfc3339();

    *counter += 1; // <- access counter inside Mutex
    let body = format!("Hello world, you are visitor number: {} \nYour IP address is: {}, \nLast time: {} \nTime you visited: {}", counter, ip, last_time, time);
    Ok(HttpResponse::Ok().body(body))
}

#[ntex::main]
async fn main() -> std::io::Result<()> {
    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();
    ctrlc::set_handler(move || {
        r.store(false, Ordering::SeqCst);
    }).expect("Error setting Ctrl-C handler");

    let current_dir = std::env::current_dir()?;
    let path = PathBuf::from(current_dir).join("./visitors.json");
    let last_count = if Path::exists(&path) {
        let json = read_to_string("./visitors.json")?;
        let data: JsonState = serde_json::from_str(&json)?;
        data.last_count
    } else {
        0
    };
    let last_time = if Path::exists(&path) {
        let json = read_to_string("./visitors.json")?;
        let data: JsonState = serde_json::from_str(&json)?;
        data.last_time
    } else {
        Local::now().to_rfc3339()
    };


    let server = HttpServer::new(move || {
        App::new()
        .service(resource("/").to(index))
        .state(AppStateWithCounter {
            counter: Mutex::new(last_count),
            time: Mutex::new(last_time.clone()),
        })
    })
    .bind("127.0.0.1:8080")?;

    let server_handle = server.run();

    // Spawn a new thread to listen for the SIGINT signal
    let running_clone = running.clone();
    let server_handle_clone = server_handle.clone();
    tokio::task::spawn_blocking(move || {
        while running_clone.load(Ordering::SeqCst) {
            thread::sleep(Duration::from_millis(100));
        }
        println!("Stopping server...");
        let new_count = server_handle_clone.;
        futures::executor::block_on(server_handle_clone.stop(true));
    });

    server_handle.await
}
