// mfl_manager/src/main.rs
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

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Initialization code...
    let session_store: app_state::SessionStore = Arc::new(Mutex::new(HashMap::<String, handler_models::SessionData>::new()));
    // Setup logger, dotenv etc.

    // Define host and port variables
    let host = "0.0.0.0";
    let port = 8080;
    let server_addr = format!("{}:{}", host, port);

    println!("Starting server at http://{}", server_addr); // Log the server address
    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(session_store.clone()))
            .wrap(handler_middleware::AuthMiddleware) // From lib
            // Reference handlers from the lib crate
            .service(handlers::login_handler)
            .service(handlers::get_free_agents_handler)
            .service(web::resource("/health").route(web::get().to(handlers::health_check)))
        // ... other services
    })
        .bind((host, port))?
        .run()
        .await
}