// src/handlers.rs
use actix_web::{get, post, web, HttpMessage, HttpRequest, HttpResponse, Responder, Result};
use crate::handler_models::{LoginRequest, LoginResponse, SessionData};
use crate::app_state::SessionStore;

#[post("/login")]
pub async fn login_handler(
    req_body: web::Json<LoginRequest>,
    sessions: web::Data<SessionStore>,
) -> Result<impl Responder> {
    use crate::errors::ServiceError;
    use crate::mfl_api;
    // ... (rest of your login_handler logic from the previous example) ...
    // Get data and validate it.
    let login_data = req_body.into_inner();

    // Initialize MFL API.  Handle errors appropriately.
    let mut api = mfl_api::MflApi::new(login_data.year.clone()).map_err(ServiceError::MflApiError)?;

    api.login(&login_data.username, &login_data.password).await.map_err(ServiceError::MflLoginError)?;

    //let league_host = api.get_league_host().await.map_err(ServiceError::MflApiError)?;

    // Create a new session.
    let token = uuid::Uuid::new_v4().to_string();

    // Store session data.
    let session_data = SessionData {
        mfl_api: api,
        //league_host,
        league_id: login_data.league_id,
        year: login_data.year
    };

   sessions.lock().unwrap().insert(token.clone(), session_data);

    Ok(HttpResponse::Ok().json(LoginResponse { token }))
}

#[get("/free-agents/{position}")]
pub async fn get_free_agents_handler(
    position: web::Path<String>,
    req: HttpRequest,
) -> Result<impl Responder> {
    use crate::errors::ServiceError;
    // Import the specific model structs you provided
    use crate::mfl_api::{PlayersPlayers, PlayersPlayer};
    use crate::handler_models::PlayerResponse; // Your response model

    // --- Existing Setup ---
    let session_data = req.extensions().get::<SessionData>().ok_or_else(|| {
        ServiceError::Unauthorized("Unauthorized: Session data missing or invalid".to_string())
    })?.clone();
    let position_str = position.into_inner();
    log::info!("get_free_agents_handler position:{}", position_str);
    
    let free_agents = session_data.mfl_api.get_free_agents(
        &session_data.league_id,
        //&session_data.league_host,
        Some(&position_str)
    ).await.map_err(ServiceError::MflApiError)?; // Assuming get_free_agents returns Vec<{id: String}> or similar
    let player_ids = free_agents
        .iter()
        .map(|player| player.id.as_str()) // Assuming free_agents elements have an 'id' field
        .collect::<Vec<&str>>()
        .join(",");

    if player_ids.is_empty() {
        let empty_response: Vec<PlayerResponse> = Vec::new();
        return Ok(HttpResponse::Ok().json(empty_response));
    }
    // --- End Setup ---

    // --- Call get_players ---
    // Assume session_data.mfl_api.get_players directly returns Result<PlayersPlayers, _>
    let players_data: PlayersPlayers = session_data.mfl_api.get_players(
        &session_data.league_id,
        //&session_data.league_host,
        &player_ids
    ).await.map_err(ServiceError::MflApiError)?;
    // Now 'players_data' is the PlayersPlayers struct instance
    // --- End Call ---


    // --- Data Transformation ---
    // Access the inner 'player' field which IS Vec<PlayersPlayer>
    // Borrow the vector to avoid moving it if players_data is needed later
    let players_list: &Vec<PlayersPlayer> = &players_data.player;

    // Iterate over the BORROWED list (&Vec<PlayersPlayer>)
    // .into_iter() on a borrow yields references to items (&PlayersPlayer)
    let response_players: Vec<PlayerResponse> = players_list
        .into_iter()
        // Closure now correctly takes a reference to PlayersPlayer
        .map(|player: &PlayersPlayer| {
            // Map fields from &PlayersPlayer -> PlayerResponse
            PlayerResponse {
                // Clone fields from the borrowed 'player' to the owned 'PlayerResponse'
                id: player.id.clone(),
                name: player.name.clone(),
                // Handle Option<String> -> String for 'position'
                // Clone the Option, then unwrap with default ("" if None)
                position: player.position.clone().unwrap_or_default(),

                team: player.team.clone()       // Clones Option<String>
            }
        })
        .collect();
    // --- End Transformation ---

    Ok(HttpResponse::Ok().json(response_players))
}


pub async fn health_check() -> impl Responder {
    HttpResponse::Ok().body("OK")
}