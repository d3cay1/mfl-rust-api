use actix_web::{
    dev::{self, Service, ServiceRequest, ServiceResponse, Transform},
    Error, HttpMessage, web, // Added web for error::*
    error // Added error for specific error types
};
use futures_util::future::LocalBoxFuture;
use std::future::{ready, Ready};
use crate::app_state::SessionStore; // Assuming this path is correct

// --- AuthMiddleware struct (No changes needed) ---
pub struct AuthMiddleware;

// --- Transform implementation (No changes needed) ---
impl<S, B> Transform<S, ServiceRequest> for AuthMiddleware
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static, // This bound is crucial!
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Transform = AuthMiddlewareService<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(AuthMiddlewareService { service }))
    }
}

// --- AuthMiddlewareService struct (No changes needed) ---
pub struct AuthMiddlewareService<S> {
    service: S,
}


// --- Service implementation (Refactored) ---
impl<S, B> Service<ServiceRequest> for AuthMiddlewareService<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static, // Crucial: Guarantees self.service.call returns a 'static future
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    // The future returned by *this* call method must be 'static
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    dev::forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        // --- Bypass logic ---
        if req.path() == "/login" {
            // Directly call the next service and pin its future.
            // self.service.call() returns S::Future, which is 'static.
            return Box::pin(self.service.call(req));
        }

        // --- Authentication Logic ---
        // Clone session store handle (Arc) if necessary
        let sessions = match req.app_data::<web::Data<SessionStore>>() {
            Some(data) => data.clone(), // Cloning web::Data gets another Arc pointer
            None => {
                // If store is missing, return an error future immediately.
                // This async block is simple and doesn't capture problematic references.
                return Box::pin(async { Err(error::ErrorInternalServerError("Session store missing")) });
            }
        };

        // Extract token
        let token_opt = req
            .headers()
            .get("Authorization")
            .and_then(|hv| hv.to_str().ok())
            .and_then(|s| s.strip_prefix("Bearer ")); // Use strip_prefix for efficiency

        let mut authenticated = false; // Flag to track if authentication succeeded

        if let Some(token) = token_opt {
            // Lock should be held only while reading
            let sessions_lock = sessions.lock().unwrap(); // Handle potential poison error if needed
            if let Some(session_data) = sessions_lock.get(token) {
                // Clone the data *while* holding the lock
                let session_data_clone = session_data.clone();
                // Drop the lock explicitly or let it go out of scope here
                drop(sessions_lock);

                // Insert cloned data into request extensions
                req.extensions_mut().insert(session_data_clone);
                authenticated = true;
            }
            // else: token exists but not found in store -> remain unauthenticated
        }
        // else: token is None -> remain unauthenticated

        // --- Decision Point ---
        if authenticated {
            // If authenticated, call the next service. Its future is 'static.
            Box::pin(self.service.call(req))
        } else {
            // If not authenticated, return an unauthorized error future.
            // This async block is simple and doesn't capture 'self' problematically.
            Box::pin(async { Err(error::ErrorUnauthorized("Unauthorized")) })
        }
    }
}