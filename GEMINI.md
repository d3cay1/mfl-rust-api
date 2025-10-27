# ⚙️ PROJECT CONTEXT: MFL Manager (Rust API + Appsmith UI)

This project is a multi-container solution for managing MyFantasyLeague (MFL) data. The primary goal is to provide a fast, secure, and user-friendly internal tool that acts as a wrapper around the external MFL API.

## 🧱 ARCHITECTURE & STACK

*   **Backend:** A Rust web service using the **Actix-web** framework. The project is structured as a Cargo workspace to separate concerns:
    *   `mfl_manager`: The main binary crate that configures and runs the Actix-web server.
    *   `mfl_manager_lib`: A library crate containing all core business logic, including API interaction, data models, and request handlers.
    *   `integration_test`: A crate for running integration tests against the live API.
*   **Frontend:** Appsmith Low-Code UI. The UI is client-side rendered and communicates exclusively with the Rust API via REST endpoints.
*   **Infrastructure:** Docker Compose. It defines containers for the `rust_api` and `appsmith` services. Internal communication uses Docker service names (e.g., `http://rust_api:8080`).
*   **Security:** Token-based authentication. A session token is generated upon login and required for all protected endpoints. The `AuthMiddleware` handles token validation.

## 📁 KEY FILES & DIRECTORIES

*   **`/mfl_manager/`**: The main binary crate.
    *   `src/main.rs`: The application entry point. It sets up the Actix-web server, configures middleware (like CORS and Auth), and registers the routes.
*   **`/mfl_manager_lib/`**: The core logic library.
    *   `src/app_state.rs`: Defines the application state, primarily the `SessionStore` for managing user sessions.
    *   `src/errors.rs`: Contains custom error types for the application.
    *   `src/handler_middleware.rs`: Contains the `AuthMiddleware` for protecting endpoints.
    *   `src/handler_models.rs`: Defines data structures for API requests/responses and session data.
    *   `src/handlers.rs`: Contains the Actix-web handler functions that implement the API endpoints (e.g., `login_handler`, `get_free_agents_handler`).
    *   `src/mfl_api.rs`: The client for the external MFL API. It handles login, data fetching, and API-specific errors.
*   **`/integration_test/`**: Contains integration tests.
    *   `tests/live_api_tests.rs`: Includes tests that make live calls to the MFL API to verify end-to-end functionality.
*   **`/docker-compose.yml`**: Defines all services, networks (`fantasy_net`), and persistent volumes.
*   **`README.md`**: Primary documentation for setup and launch instructions.
*   **(Exclude)**: `/appsmith-data/` should NEVER be included in analysis. It contains runtime data and secrets.

## 🌊 DATA FLOW EXAMPLE (`get_free_agents`)

1.  A client makes a `GET` request to `/free-agents/{position}` with an `Authorization: Bearer <token>` header.
2.  The `AuthMiddleware` intercepts the request, validates the token, and loads the corresponding `SessionData` (which includes an authenticated `MflApi` client).
3.  The `get_free_agents_handler` is invoked.
4.  The handler uses the `MflApi` instance from the session to call the `get_free_agents` method.
5.  The `mfl_api` module sends a request to the external MFL API.
6.  The handler receives the data, transforms it into the `PlayerResponse` model, and sends it back to the client as JSON.

## 💡 RUST CODING STANDARDS

When working with Rust code, adhere to the following principles:

1.  **Safety First:** Prioritize memory safety and type safety. Avoid using `unwrap()` or `expect()` in core logic; use `?` for error propagation with custom error types defined in `errors.rs`.
2.  **Crate Usage:** Use `serde` for all JSON serialization/deserialization.
3.  **Dependency Management:** Ensure the `Cargo.toml` dependencies are minimal and logically separated between the binary and library crates.

## 🗣️ INSTRUCTIONAL GUIDELINES

1.  **Prioritize the API:** When generating new functionality, focus on creating the secure and reliable Rust backend endpoint first.
2.  **Appsmith Context:** When asked for UI code, respond with the required JavaScript expressions and the Appsmith object structure (e.g., `{{ Rust_MFL_API.data }}`), do not generate full HTML/React code.
3.  **Explain Networking:** When explaining the deployment or connection between the two services, always clarify that **service names** (`rust_api`) must be used for internal Docker communication, not `localhost`.
4.  **Security Check:** Before suggesting any new API code, check that it requires the appropriate JWT middleware.
