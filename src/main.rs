/*
Random data generation and selection microservice for the PIjN protocol project
Developer: Urban Egor
Server version: 3.7.30 a
Random module version: 4.5.38 a
*/



use actix_web::{post, get, web, App, HttpServer, Responder, HttpResponse, middleware::Logger, HttpRequest};
use serde::{Deserialize, Serialize};
use serde_json;
use chrono::Local;
use std::time::Instant;
use reqwest;
use tracing::{info, warn, error};
use tracing_subscriber;

mod random_module;

const MAX_LENGTH: usize = 256;
const MAX_COUNT: usize = 100;

#[derive(Deserialize)]
struct GenerateParams {
    use_digits: bool,
    use_lowercase: bool,
    use_uppercase: bool,
    use_spec: bool,
    length: usize,
}

#[derive(Serialize, Deserialize)]
struct ApiResponse<T> {
    success: bool,
    data: T,
}

#[derive(Deserialize)]
struct ChooseParams<T> {
    items: Vec<T>,
    count: usize,
}

// ------------------ Handlers --------------------

#[post("/generate_random_string")]
async fn generate_handler(req: HttpRequest, params: web::Json<GenerateParams>) -> impl Responder {
    let start = Instant::now();
    let peer = req.peer_addr().map(|a| a.to_string()).unwrap_or_else(|| "Unknown".into());
    info!(target: "generate_handler", "Request from: {}", peer);

    if params.length == 0 || params.length > MAX_LENGTH {
        let msg = format!("Invalid length: {} (must be 1–{})", params.length, MAX_LENGTH);
        warn!(target: "generate_handler", "{}", msg);
        return HttpResponse::BadRequest().json(ApiResponse { success: false, data: msg });
    }

    if !(params.use_digits || params.use_lowercase || params.use_uppercase || params.use_spec) {
        let msg = "At least one charset must be enabled (digits, lowercase, uppercase, special).";
        warn!(target: "generate_handler", "{}", msg);
        return HttpResponse::BadRequest().json(ApiResponse { success: false, data: msg.to_string() });
    }

    let result = std::panic::catch_unwind(|| {
        random_module::generate_random_string(
            params.use_digits,
            params.use_lowercase,
            params.use_uppercase,
            params.use_spec,
            params.length,
        )
    });

    match result {
        Ok(output) => {
            let duration = start.elapsed().as_millis();
            info!(target: "generate_handler", "Generation completed in {} ms", duration);
            HttpResponse::Ok().json(ApiResponse { success: true, data: output })
        }
        Err(_) => {
            error!(target: "generate_handler", "Panic occurred during string generation");
            HttpResponse::InternalServerError().json(ApiResponse { success: false, data: "Internal server error".to_string() })
        }
    }
}

#[post("/generate_random_choose")]
async fn choose_handler(req: HttpRequest, params: web::Json<ChooseParams<String>>) -> impl Responder {
    let start = Instant::now();
    let peer = req.peer_addr().map(|a| a.to_string()).unwrap_or_else(|| "Unknown".into());
    info!(target: "choose_handler", "Request from: {}", peer);

    if params.count == 0 || params.count > MAX_COUNT {
        let msg = format!("Invalid count: {} (must be 1–{})", params.count, MAX_COUNT);
        warn!(target: "choose_handler", "{}", msg);
        return HttpResponse::BadRequest().json(ApiResponse { success: false, data: msg });
    }

    if params.count > params.items.len() {
        let msg = "Count must be <= item count.";
        warn!(target: "choose_handler", "{}", msg);
        return HttpResponse::BadRequest().json(ApiResponse { success: false, data: msg.to_string() });
    }

    let result = std::panic::catch_unwind(|| {
        random_module::generate_random_choose(params.items.clone(), params.count)
    });

    match result {
        Ok(selected) => {
            let duration = start.elapsed().as_millis();
            info!(target: "choose_handler", "Random choice completed in {} ms", duration);
            HttpResponse::Ok().json(ApiResponse { success: true, data: selected })
        }
        Err(_) => {
            error!(target: "choose_handler", "Panic occurred during random choose");
            HttpResponse::InternalServerError().json(ApiResponse { success: false, data: "Internal server error".to_string() })
        }
    }
}

#[get("/status")]
async fn get_module_status(req: HttpRequest) -> impl Responder {
    let client_addr = req.peer_addr()
        .map(|a| a.to_string())
        .unwrap_or_else(|| "Unknown".to_string());

    info!(target: "status_handler", "Client {} requested status", client_addr);
    HttpResponse::Ok().json(serde_json::json!({ "success": true, "data": null }))
}

// ------------------ Main --------------------

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    init_tracing();

    let Some(port) = fetch_port().await else {
        error!(target: "main", "Can't get port. Random data module won't start.");
        std::process::exit(1);
    };

    let ip = "127.0.0.1";

    info!(target: "main", "Starting random module microservice on {}:{}", ip, port);
    info!(target: "main", "Version: 3.6.21 b / Random module version: 4.5.38 a");

    HttpServer::new(|| {
        App::new()
            .wrap(Logger::default())
            .service(generate_handler)
            .service(choose_handler)
            .service(get_module_status)
    })
    .workers(4)
    .bind((ip, port))?
    .run()
    .await
}

async fn fetch_port() -> Option<u16> {
    let url = "http://127.0.0.1:1030/getport/random_module_microservice";
    info!(target: "port_resolver", "Requesting port from {}", url);

    match reqwest::get(url).await {
        Ok(resp) => {
            if resp.status().is_success() {
                match resp.json::<ApiResponse<serde_json::Value>>().await {
                    Ok(json) => {
                        if json.success {
                            if let Some(port_val) = json.data.as_u64() {
                                let port = port_val as u16;
                                info!(target: "port_resolver", "Got port: {}", port);
                                return Some(port);
                            } else {
                                error!(target: "port_resolver", "No port in response data");
                            }
                        } else {
                            warn!(target: "port_resolver", "Error from server: {:?}", json.data);
                        }
                    }
                    Err(e) => error!(target: "port_resolver", "JSON parse error: {}", e),
                }
            } else {
                warn!(target: "port_resolver", "Response status: {}", resp.status());
            }
        }
        Err(e) => error!(target: "port_resolver", "Request error: {}", e),
    }

    None
}

fn init_tracing() {
    let date = Local::now().format("%d_%m_%Y").to_string();
    let log_path = format!("./logs/random_module_microservice_{}.log", date);
    std::fs::create_dir_all("./logs").ok();

    tracing_subscriber::fmt()
        .with_target(true)
        .with_writer(std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(log_path)
            .expect("Cannot open log file"))
        .with_thread_names(true)
        .with_ansi(false)
        .init();
}
