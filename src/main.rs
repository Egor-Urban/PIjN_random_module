/*
Random data generation and selection microservice for the PIjN protocol project
Developer: Urban Egor
Server version: 3.3.14 b
Random module version: 4.5.38 a
*/



use actix_web::{post, web, App, HttpServer, Responder, HttpResponse, middleware::Logger, HttpRequest};
use serde::{Deserialize, Serialize};
use std::fs::{self, OpenOptions};
use std::io::Write;
use chrono::Local;
use std::sync::Mutex;

mod random_module;



const MAX_LENGTH: usize = 256;   
const MAX_COUNT: usize = 100;



struct RandomModuleMicroserviceLogger {
    file: Mutex<std::fs::File>,
}



#[derive(Deserialize)]
struct GenerateParams {
    use_digits: bool,
    use_lowercase: bool,
    use_uppercase: bool,
    use_spec: bool,
    length: usize,
}



#[derive(Serialize)]
struct ApiResponse<T> {
    success: bool,
    data: T,
}



#[derive(Deserialize)]
struct ChooseParams<T> {
    items: Vec<T>,
    count: usize,
}



#[derive(Deserialize)]
struct Config {
    port: u16,
    ip: String,
}



impl RandomModuleMicroserviceLogger {
    fn new() -> Self {
        let date_str = Local::now().format("%d_%m_%Y").to_string();
        let log_dir = "./logs";
        fs::create_dir_all(log_dir).unwrap_or_else(|e| eprintln!("Failed to create logs directory: {}", e));
        let file_path = format!("{}/random_module_microservice_{}.log", log_dir, date_str);

        let file = OpenOptions::new()
            .append(true)
            .create(true)
            .open(file_path)
            .expect("Failed to open log file");

        Self {
            file: Mutex::new(file),
        }
    }


    fn log(&self, source: &str, level: &str, message: &str) {
        let now = Local::now().format("%d.%m.%Y %H:%M:%S").to_string();
        let log_entry = format!("[{}][{}][{}] {}\n", now, source, level, message);

        // Выводим в консоль
        print!("{}", log_entry);

        if let Ok(mut file) = self.file.lock() {
            let _ = file.write_all(log_entry.as_bytes());
        }
    }
}


static LOGGER: once_cell::sync::Lazy<RandomModuleMicroserviceLogger> = once_cell::sync::Lazy::new(RandomModuleMicroserviceLogger::new);



// Endpoints
#[post("/generate_random_string")]
async fn generate_handler(req: HttpRequest, params: web::Json<GenerateParams>) -> impl Responder {
    let peer_addr = req.peer_addr()
        .map(|addr| addr.to_string())
        .unwrap_or_else(|| "Unknown".to_string());

    LOGGER.log("generate_string_handler", "INFO", &format!("Request from: {}", peer_addr));

    if params.length == 0 || params.length > MAX_LENGTH {
        let msg = format!("Invalid length: {}. Must be between 1 and {}", params.length, MAX_LENGTH);
        LOGGER.log("generate_string_handler", "WARN", &msg);
        return HttpResponse::BadRequest().json(ApiResponse {
            success: false,
            data: msg,
        });
    }
    if !params.use_digits && !params.use_lowercase && !params.use_uppercase && !params.use_spec {
        let msg = "At least one character set must be enabled (digits, lowercase, uppercase, special).";
        LOGGER.log("generate_string_handler", "WARN", msg);
        return HttpResponse::BadRequest().json(ApiResponse {
            success: false,
            data: msg.to_string(),
        });
    }

    match std::panic::catch_unwind(|| {
        random_module::generate_random_string(
            params.use_digits,
            params.use_lowercase,
            params.use_uppercase,
            params.use_spec,
            params.length,
        )
    }) {
        Ok(result) => {
            LOGGER.log("generate_string_handler", "INFO", "Generated random string successfully.");
            HttpResponse::Ok().json(ApiResponse {
                success: true,
                data: result,
            })
        }
        Err(e) => {
            LOGGER.log("generate_string_handler", "ERROR", &format!("Failed to generate random string: {:?}", e));
            HttpResponse::InternalServerError().json(ApiResponse {
                success: false,
                data: "Internal server error".to_string(),
            })
        }
    }
}


#[post("/generate_random_choose")]
async fn choose_handler(req: HttpRequest, params: web::Json<ChooseParams<String>>) -> impl Responder {
    let peer_addr = req.peer_addr()
        .map(|addr| addr.to_string())
        .unwrap_or_else(|| "Unknown".to_string());

    LOGGER.log("generate_choose_handler", "INFO", &format!("Request from: {}", peer_addr));

    if params.count == 0 || params.count > MAX_COUNT {
        let msg = format!("Invalid count: {}. Must be between 1 and {}", params.count, MAX_COUNT);
        LOGGER.log("generate_choose_handler", "WARN", &msg);
        return HttpResponse::BadRequest().json(ApiResponse {
            success: false,
            data: msg,
        });
    }

    if params.count > params.items.len() {
        let msg = "Count must be less than or equal to number of items.";
        LOGGER.log("generate_choose_handler", "WARN", msg);
        return HttpResponse::BadRequest().json(ApiResponse {
            success: false,
            data: msg.to_string(),
        });
    }

    match std::panic::catch_unwind(|| {
        random_module::generate_random_choose(params.items.clone(), params.count)
    }) {
        Ok(selected) => {
            LOGGER.log("generate_choose_handler", "INFO", "Random choose successful.");
            HttpResponse::Ok().json(ApiResponse {
                success: true,
                data: selected,
            })
        }
        Err(e) => {
            LOGGER.log("generate_choose_handler", "ERROR", &format!("Failed to randomly choose items: {:?}", e));
            HttpResponse::InternalServerError().json(ApiResponse {
                success: false,
                data: "Internal server error".to_string(),
            })
        }
    }
}



#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let config_content = fs::read_to_string("config.json")
        .expect("Failed to read config.json");
    let config: Config = serde_json::from_str(&config_content)
        .expect("Failed to parse config.json");

    LOGGER.log("main", "INFO", &format!("Random module microservice server is starting on {}:{}", config.ip, config.port));
    LOGGER.log("main", "INFO", &format!("Random module microservice version: 3.3.14 b"));
    LOGGER.log("security", "INFO", &format!("To ensure confidentiality, randomly generated data is not saved"));

    HttpServer::new(|| {
        App::new()
            .wrap(Logger::default())
            .service(generate_handler)
            .service(choose_handler)
    })
    .workers(4)
    .bind((config.ip.as_str(), config.port))?  
    .run()
    .await
}
