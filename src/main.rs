/*
Random Module Microservice (PIjN Protocol)
Developer: Urnan Egor
Version: 5.7.44 r

*/

use actix_web::{dev::{ServiceRequest, ServiceResponse, Transform, Service}, get, post, web, App, HttpRequest, HttpResponse, HttpServer, Responder, Error, body::BoxBody};
use serde::{Deserialize, Serialize};
use futures::future::{ok, Ready, LocalBoxFuture};
use std::task::{Context, Poll};
use std::net::IpAddr;
use std::rc::Rc;
use std::time::Instant;
use tokio::time::Duration;
use tracing::{info, warn, error};

mod status;
mod utils;
mod random_module;

use status::get_status;
use utils::{fetch_port, init_tracing, load_config, get_local_ip};



const MAX_LENGTH: usize = 256;
const MAX_COUNT: usize = 100;



// --- local network protect ---


pub struct LocalNetworkOnly;


impl<S> Transform<S, ServiceRequest> for LocalNetworkOnly
where
    S: Service<ServiceRequest, Response = ServiceResponse<BoxBody>, Error = Error> + 'static,
    S::Future: 'static,
{
    type Response = ServiceResponse<BoxBody>;
    type Error = Error;
    type InitError = ();
    type Transform = LocalNetworkOnlyMiddleware<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ok(LocalNetworkOnlyMiddleware {
            service: Rc::new(service),
        })
    }
}


pub struct LocalNetworkOnlyMiddleware<S> {
    service: Rc<S>,
}


impl<S> Service<ServiceRequest> for LocalNetworkOnlyMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<BoxBody>, Error = Error> + 'static,
    S::Future: 'static,
{
    type Response = ServiceResponse<BoxBody>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(&self, ctx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.service.poll_ready(ctx)
    }

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let svc = Rc::clone(&self.service);

        let ip_opt = req.connection_info().realip_remote_addr()
            .and_then(|addr| addr.split(':').next())
            .and_then(|ip_str| ip_str.parse::<IpAddr>().ok());

        let allowed = match ip_opt {
            Some(ip) => is_local_ip(&ip),
            None => false,
        };

        if allowed {
            Box::pin(async move { svc.call(req).await })
        } else {
            Box::pin(async move {
                Err(actix_web::error::PayloadError::Io(
                    std::io::Error::new(std::io::ErrorKind::ConnectionAborted, "not local ip")
                ).into())
            })
        }
    }
}


fn is_local_ip(ip: &IpAddr) -> bool {
    match ip {
        IpAddr::V4(ipv4) => ipv4.is_loopback() || ipv4.is_private(),
        IpAddr::V6(ipv6) => ipv6.is_loopback(),
    }
}


// --- local network protect ---



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



#[get("/status")]
async fn status_handler(start: web::Data<Instant>, req: HttpRequest) -> impl Responder {
    let client_addr = req
        .peer_addr()
        .map(|a| a.to_string())
        .unwrap_or_else(|| "Unknown".to_string());
    let status_json = get_status(*start.get_ref());
    let status = serde_json::json!({ "success": true, "data": status_json });

    info!(target: "status_handler", "Client {} requested status: {}", client_addr, status);

    HttpResponse::Ok().json(status)
}


#[get("/stop")]
async fn stop_handler() -> impl Responder {
    info!(target: "control", "Received /stop request. Exiting...");

    tokio::spawn(async {
        tokio::time::sleep(Duration::from_secs(1)).await;
        std::process::exit(0);
    });

    HttpResponse::Ok().json(serde_json::json!({ "success": true, "data": null }))
}


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



#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let start = Instant::now();
    let start_data = web::Data::new(start);
    let config = load_config();

    init_tracing(&config.logs_dir, &config.name_for_port_manager);

    let Some(port) = fetch_port(&config).await else {
        error!(target: "main", "Failed to retrieve port. {} will not start.", &config.name_for_port_manager);
        std::process::exit(1);
    };

    let ip = get_local_ip().map(|addr| addr.to_string()).unwrap_or("ERROR".to_string());

    info!(target: "main", "Starting {} on {}:{}", &config.name_for_port_manager, ip, port);

    HttpServer::new(move || {
        App::new()
            .app_data(start_data.clone())
            .wrap(LocalNetworkOnly)  
            .service(status_handler)
            .service(stop_handler)
    })
    .workers(config.workers_count)
    .bind((ip.as_str(), port))?
    .run()
    .await
}