// src/app_state.rs
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use crate::handler_models::SessionData;

// Use a thread-safe HashMap for session storage
pub type SessionStore = Arc<Mutex<HashMap<String, SessionData>>>;