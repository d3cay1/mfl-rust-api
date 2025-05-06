// tests/live_api_tests.rs (or similar)

#[cfg(test)]
mod live_tests {
    use actix_web::http::header::{HeaderValue, CONTENT_TYPE};
    use actix_web::{
        http::header::AUTHORIZATION,
        http::StatusCode,
        test::{self, TestRequest}, web::{self},
        App,
        HttpMessage,
    };
    use chrono::Local;
    // REMOVE: use std::env; // No longer reading environment variables
    use serde_json;
    use std::collections::HashMap;
    use std::sync::{Arc, Mutex};
    // Import the Local timezone provider


    // --- Import necessary items from your crate ---
    use mfl_manager_lib::{ // Assuming your crate is named mfl_stats
                           app_state::SessionStore,
                           handler_middleware::AuthMiddleware,
                           handler_models::{LoginRequest, LoginResponse, PlayerResponse, SessionData},
                           handlers::{get_free_agents_handler, login_handler},
    };

    // --- Test Function ---
    // !!! WARNING !!! WARNING !!! WARNING !!!
    // This test uses EMBEDDED credentials for the LIVE MFL API.
    // Replace placeholders ONLY for temporary local validation.
    // DO NOT COMMIT this file with real credentials.
    // It requires MFL API access and depends on external factors.
    // Marked #[ignore] as it's slow and depends on external factors.
    #[actix_web::test]
    //#[ignore] // Ignore by default, run with `cargo test -- --ignored`
    async fn test_get_free_agents_live_api_embedded_creds() { // Renamed slightly
        // --- 1. Embedded Credentials ---
        // !!! WARNING: Replace with your actual credentials for testing !!!
        // !!! DO NOT COMMIT REAL CREDENTIALS !!!
        let mfl_user = "d3cay".to_string();
        let mfl_password = "R49G&#T*cS2@".to_string();
        let mfl_league_id = "74560".to_string(); // e.g., "12345"
        let mfl_year = "2025".to_string(); // e.g., "2025" (Must be current/valid year for API)

        // --- Safety Check: Prevent running with placeholder values ---
        // If you try running without changing these, the test will panic.
        // if mfl_user == "YOUR_MFL_USERNAME" || mfl_password == "YOUR_MFL_PASSWORD" || mfl_league_id == "YOUR_LEAGUE_ID" || mfl_year == "YOUR_LEAGUE_YEAR" {
        //     // Use panic to make it obvious the test needs editing before running.
        //     panic!("Placeholder credentials detected! You MUST replace the placeholder values in the test code with your actual MFL credentials for temporary local validation. DO NOT COMMIT real credentials.");
        // }
        println!("INFO: Running live API test with embedded credentials (DO NOT COMMIT).");


        // --- 2. Set up Full App --- (Same as before)
        let session_store: SessionStore = Arc::new(Mutex::new(HashMap::<String, SessionData>::new()));
        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(session_store.clone())) // Share session store
                .wrap(AuthMiddleware) // Include the real middleware
                .service(login_handler) // Login handler
                .service(get_free_agents_handler) // Target handler
        ).await;

        // --- 3. Perform Login --- (Uses embedded creds now)
        let login_req_body = LoginRequest {
            username: mfl_user, // Uses embedded value
            password: mfl_password, // Uses embedded value
            league_id: mfl_league_id.clone(), // Uses embedded value
            year: mfl_year.clone(), // Uses embedded value
        };

        let login_req = TestRequest::post()
            .uri("/login")
            .set_json(&login_req_body)
            .to_request();

        let login_resp = test::call_service(&app, login_req).await;
        // Added more context to login failure message
        assert_eq!(login_resp.status(), StatusCode::OK,
                   "Login request failed. Check embedded credentials, MFL API status, and league/year validity ({}).", mfl_year);

        // Extract token
        let login_resp_body: LoginResponse = test::read_body_json(login_resp).await;
        let token = login_resp_body.token;
        assert!(!token.is_empty(), "Login did not return a token");
        println!("Obtained temporary auth token for test using embedded credentials.");

        // --- 4. Call Target Endpoint --- (Same as before)
        let target_position = "QB"; // Or any position you want to test
        let req = TestRequest::get()
            .uri(&format!("/free-agents/{}", target_position))
            .insert_header((AUTHORIZATION, format!("Bearer {}", token))) // Use obtained token
            .to_request();

        println!("Making request to /free-agents/{}", target_position);
        let resp = test::call_service(&app, req).await;

        // --- 5. Assert Response --- (Same cautious assertions as before)
        assert_eq!(resp.status(), StatusCode::OK, "Expected status OK for /free-agents");

        let expected_hv = HeaderValue::from_static(mime::APPLICATION_JSON.as_ref());

        let content_type = resp.headers().get(CONTENT_TYPE);
        assert_eq!(content_type, Some(&expected_hv), "Expected Content-Type application/json");

        // Read body Bytes first to handle potential parsing errors gracefully
        let body_bytes = test::read_body(resp).await;

        // Attempt to deserialize - this validates the structure
        let parse_result: Result<Vec<PlayerResponse>, _> = serde_json::from_slice(&body_bytes);

        match parse_result {
            Ok(players) => {

                // Get the current local date and time
                let now = Local::now();

                // Format the time according to your desired format string
                // "%a %b %d %H:%M:%S %Y" corresponds to "Sat Apr 12 22:51:36 2025"
                let formatted_time = now.format("%a %b %d %H:%M:%S %Y").to_string();

                println!("Successfully deserialized {} players for position {} (Live MFL data as of approx. {})",
                         players.len(), target_position, formatted_time);

                // **Cautious Assertions:** Avoid asserting specific players.
                // Assertions depend heavily on the current MFL state.

                // Example: Check if player positions are consistent (allowing for empty string default)
                for player in &players {
                    assert!(player.position == target_position || player.position.is_empty(),
                            "Player {} ({}) returned via live API has unexpected position '{}' when requesting '{}'",
                            player.name, player.id, player.position, target_position);
                }
                println!("Validated structure and basic position consistency for {} players.", players.len());
            },
            Err(e) => {
                // Provide more context on deserialization failure
                let body_string = String::from_utf8_lossy(&body_bytes);
                panic!("Failed to deserialize response body into Vec<PlayerResponse>. Error: {}. Body: {}", e, body_string);
            }
        }
        // --- 6. Cleanup ---
        // Explicitly clear the session store before the test function ends.
        // This ensures the MflApi/reqwest::Client inside SessionData is dropped
        // *before* the test runtime starts its restricted teardown phase.
        { // Added scope for the lock guard
            let mut store = session_store.lock().unwrap();
            store.clear();
            println!("INFO: Cleared session store explicitly before test completion.");
        } // Mutex guard is dropped here
    }
}