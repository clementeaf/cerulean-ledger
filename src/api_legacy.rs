//! Legacy API router — all handlers migrated to `api::handlers::*`.
//!
//! This module only provides `config_routes` which creates the `/api/v1` scope
//! and delegates to `ApiRoutes::register` for all endpoint registration.

use actix_web::web;

/// Configures the `/api/v1` scope with all scaffold routes.
pub fn config_routes(cfg: &mut web::ServiceConfig) {
    let scope = crate::api::routes::ApiRoutes::register(web::scope("/api/v1"));
    cfg.service(scope);
}
