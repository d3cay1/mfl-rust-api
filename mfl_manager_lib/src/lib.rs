// mfl_manager_lib/src/lib.rs
pub mod app_state;
pub mod errors;
pub mod handler_models;
pub mod handlers;
pub mod handler_middleware;
pub mod mfl_api;

// You might also add a function here to configure and return the Actix App
// that main.rs can call, but simply exporting modules is often enough.