// mfl_manager/src/main.rs
use actix_cors::Cors;
use actix_web::{web, App, HttpServer};
// Import *from the library crate*
use mfl_manager_lib::{
    app_state,
    handler_middleware,
    handler_models,
    handlers,
};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use log::info;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Initialize the logger
    env_logger::init();
    
    // Initialization code...
    let session_store: app_state::SessionStore = Arc::new(Mutex::new(HashMap::<String, handler_models::SessionData>::new()));
    // Setup logger, dotenv etc.

    // Define host and port variables
    let host = "0.0.0.0";
    let port = 8080;
    let server_addr = format!("{}:{}", host, port);

    info!("Starting server at http://{}", server_addr); // Log the server address
    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(session_store.clone()))
            .wrap(
                Cors::default()
                    .allow_any_origin()              // Allow all origins (unsafe for production)
                    .send_wildcard()
                    .allowed_methods(vec!["GET", "POST", "PUT", "DELETE", "OPTIONS"])
                    .allowed_headers(vec!["Authorization", "Content-Type"])
                    .max_age(3600),                 // Cache OPTIONS responses for 1 hour
            )
            // Register public services first
            .service(handlers::login_handler)
            .service(web::resource("/health").route(web::get().to(handlers::health_check)))

            // Now, create a new scope for services that require authentication
            .service(
                web::scope("") // Using an empty scope to keep original paths
                    .wrap(handler_middleware::AuthMiddleware)
                    .service(handlers::get_free_agents_handler)
                    // Add other protected services here in the future
            )
        // ... other services
    })
        .bind((host, port))?
        .run()
        .await
}