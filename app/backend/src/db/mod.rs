pub mod models;

use sqlx::PgPool;

/// Shared application state injected into all Axum handlers.
#[derive(Clone)]
pub struct AppState {
    pub pool: PgPool,
}
