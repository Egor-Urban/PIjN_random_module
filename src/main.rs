/*
Random data generation and selection microservice for the PIjN protocol project
Developer: Urban Egor
Server version: 3.6.21 b
Random module version: 4.5.38 a
*/



use actix_web::{post, get, web, App, HttpServer, Responder, HttpResponse, middleware::Logger, HttpRequest};
use serde::{Deserialize, Serialize};
use std::fs::{self, OpenOptions};
use std::io::Write;
use chrono::{Local};
use std::sync::Mutex;
use std::time::Instant;
use reqwest;
use once_cell::sync::Lazy;

mod random_module;


const MAX_LENGTH: usize = 256;
const MAX_COUNT: usize = 100;



struct LoggerService {
    file: Mutex<std::fs::File>,
}


impl LoggerService {
    fn new() -> Self {
        let date = Local::now().format("%d_%m_%Y");
        fs::create_dir_all("./logs").ok();
        let file_path = format!("./logs/random_module_microservice_{}.log", date);
        let file = OpenOptions::new().append(true).create(true).open(file_path)
            .expect("Failed to open log file");

        Self { file: Mutex::new(file) }
    }


    fn log(&self, source: &str, level: &str, message: &str) {
        let now = Local::now().format("%d.%m.%Y %H:%M:%S");
        let entry = format!("[{}][{}][{}] {}\n", now, source, level, message);
        print!("{}", entry);
        if let Ok(mut file) = self.file.lock() {
            let _ = file.write_all(entry.as_bytes());
        }
    }
}


static LOGGER: Lazy<LoggerService> = Lazy::new(LoggerService::new);



#[derive(Deserialize)]
struct GenerateParams {
    use_digits: bool,
    use_lowercase: bool,
    use_uppercase: bool,
    use_spec: bool,
    length: usize,
}


#[derive(Serialize, Deserialize, Debug)]
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
    let peer = req.peer_addr().map(|a| a.to_string()).unwrap_or("Unknown".to_string());
    LOGGER.log("generate_string_handler", "INFO", &format!("Request from: {}", peer));

    if params.length == 0 || params.length > MAX_LENGTH {
        let msg = format!("Invalid length: {} (must be 1–{})", params.length, MAX_LENGTH);
        LOGGER.log("generate_string_handler", "WARN", &msg);
        return HttpResponse::BadRequest().json(ApiResponse { success: false, data: msg });
    }

    if !(params.use_digits || params.use_lowercase || params.use_uppercase || params.use_spec) {
        let msg = "At least one charset must be enabled (digits, lowercase, uppercase, special).";
        LOGGER.log("generate_string_handler", "WARN", msg);
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
            let duration = start.elapsed();
            LOGGER.log("generate_string_handler", "INFO", &format!("Success in {:?}.", duration));
            HttpResponse::Ok().json(ApiResponse { success: true, data: output })
        }
        Err(_) => {
            LOGGER.log("generate_string_handler", "ERROR", "Generation panic occurred");
            HttpResponse::InternalServerError().json(ApiResponse { success: false, data: "Internal server error".to_string() })
        }
    }
}



#[post("/generate_random_choose")]
async fn choose_handler(req: HttpRequest, params: web::Json<ChooseParams<String>>) -> impl Responder {
    let start = Instant::now();
    let peer = req.peer_addr().map(|a| a.to_string()).unwrap_or("Unknown".to_string());
    LOGGER.log("generate_choose_handler", "INFO", &format!("Request from: {}", peer));

    if params.count == 0 || params.count > MAX_COUNT {
        let msg = format!("Invalid count: {} (must be 1–{})", params.count, MAX_COUNT);
        LOGGER.log("generate_choose_handler", "WARN", &msg);
        return HttpResponse::BadRequest().json(ApiResponse { success: false, data: msg });
    }

    if params.count > params.items.len() {
        let msg = "Count must be <= item count.";
        LOGGER.log("generate_choose_handler", "WARN", msg);
        return HttpResponse::BadRequest().json(ApiResponse { success: false, data: msg.to_string() });
    }

    let result = std::panic::catch_unwind(|| {
        random_module::generate_random_choose(params.items.clone(), params.count)
    });

    match result {
        Ok(selected) => {
            let duration = start.elapsed();
            LOGGER.log("generate_choose_handler", "INFO", &format!("Success in {:?}.", duration));
            HttpResponse::Ok().json(ApiResponse { success: true, data: selected })
        }
        Err(_) => {
            LOGGER.log("generate_choose_handler", "ERROR", "Random choose panic occurred");
            HttpResponse::InternalServerError().json(ApiResponse { success: false, data: "Internal server error".to_string() })
        }
    }
}


#[get("/status")]
async fn get_module_status(req: HttpRequest) -> impl Responder {
    let client_addr = req.peer_addr()
        .map(|a| a.to_string())
        .unwrap_or_else(|| "Unknown".to_string());


        LOGGER.log("get_module_status", "INFO", &format!("Client {} get status", client_addr));
        HttpResponse::Ok().json(serde_json::json!({ "success": true, "data": null }))
}



// ------------------ Main --------------------

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let Some(port) = fetch_port().await else {
        LOGGER.log("main", "ERROR", "Cant ger port. Cant start random data module");
        std::process::exit(1);
    };

    let ip = "127.0.0.1";

    LOGGER.log("main", "INFO", &format!("Stаrt random module microservice server on {}:{}", ip, port));
    LOGGER.log("main", "INFO", "Random module microservice version: 3.6.21 b");
    LOGGER.log("security", "INFO", "Cant saving confiderncial random data");

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
    LOGGER.log("port_resolver", "INFO", &format!("Requesting port for microservice from {}", url));

    match reqwest::get(url).await {
        Ok(resp) => { 
            let status = resp.status();
            if status.is_success() {
                match resp.json::<ApiResponse<serde_json::Value>>().await {
                    Ok(json) => {
                        if json.success {
                            if let Some(port_val) = json.data.as_u64() {
                                let port = port_val as u16;
                                LOGGER.log("port_resolver", "INFO", &format!("Requested port: {}", port));
                                return Some(port);
                            } else {
                                LOGGER.log("port_resolver", "ERROR", "Cant get port from data");
                            }
                        } else {
                            LOGGER.log("port_resolver", "WARN", &format!("Server returns error: {:?}", json.data));
                        }
                    }
                    Err(e) => {
                        LOGGER.log("port_resolver", "ERROR", &format!("Error with reading JSON: {}", e));
                    }
                }
            } else {
                LOGGER.log("port_resolver", "WARN", &format!("Port manager responding with error {}", status));
            }
        } 
        Err(e) => {
            LOGGER.log("port_resolver", "ERROR", &format!("Error with request data from port manager: {}", e));
        }
    }

    LOGGER.log("port_resolver", "INFO", "Cant start random data module");
    None
}

