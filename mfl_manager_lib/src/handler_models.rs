// src/handler_models
use serde::{Deserialize, Serialize};
use crate::mfl_api::MflApi;

#[derive(Deserialize,Serialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
    pub league_id: String,
    pub year: String,
}

#[derive(Serialize, Deserialize)]
pub struct LoginResponse {
    pub token: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct PlayerResponse {
    pub id: String,
    pub name: String,
    pub position: String,
    pub team: Option<String>
}

#[derive(Debug, Clone)] // Added Clone
pub struct SessionData {
    pub mfl_api: MflApi, // Store the initialized MflApi
    //pub league_host: String, // and other session-specific data
    pub league_id: String,
    pub year: String
}