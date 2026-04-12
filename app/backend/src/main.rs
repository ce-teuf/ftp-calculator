use std::net::SocketAddr;
use axum::{
    body::Body,
    extract::Request,
    http::{header, StatusCode},
    response::Response,
    routing::{delete, get, post, put},
    Router,
};
use include_dir::{include_dir, Dir};
use sqlx::postgres::PgPoolOptions;
use tower_http::cors::{Any, CorsLayer};

mod api;
mod compute;
mod db;

use db::AppState;

static DIST: Dir<'static> = include_dir!("$CARGO_MANIFEST_DIR/../web-app/dist");

async fn serve_frontend(req: Request) -> Response {
    let path = req.uri().path().trim_start_matches('/');

    let file = DIST.get_file(path)
        .or_else(|| DIST.get_file("index.html"));

    match file {
        Some(f) => {
            let mime = mime_guess::from_path(f.path()).first_or_octet_stream();
            Response::builder()
                .status(StatusCode::OK)
                .header(header::CONTENT_TYPE, mime.as_ref())
                .body(Body::from(f.contents()))
                .unwrap()
        }
        None => Response::builder()
            .status(StatusCode::NOT_FOUND)
            .body(Body::from("Not found"))
            .unwrap(),
    }
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgresql://ftp_dev:ftp_dev@127.0.0.1:5432/ftp_simulator_dev".to_string());

    let pool = PgPoolOptions::new()
        .max_connections(10)
        .connect(&database_url)
        .await
        .expect("Failed to connect to PostgreSQL. Is DATABASE_URL set and the DB running?");

    sqlx::migrate!("src/db/migrations")
        .run(&pool)
        .await
        .expect("Failed to run database migrations");

    tracing::info!("Database migrations applied");

    let state = AppState { pool };

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let app = Router::new()
        // Module 1 — Risk types
        .route("/api/risk-types", get(api::rate_matrices::list_risk_types))
        // Module 1 — Rate matrices
        .route("/api/rate-matrices",      get(api::rate_matrices::list_rate_matrices))
        .route("/api/rate-matrices",      post(api::rate_matrices::create_rate_matrix))
        .route("/api/rate-matrices/{id}", get(api::rate_matrices::get_rate_matrix))
        .route("/api/rate-matrices/{id}", put(api::rate_matrices::update_rate_matrix))
        .route("/api/rate-matrices/{id}", delete(api::rate_matrices::delete_rate_matrix))
        // Module 2 — Hypercubes
        .route("/api/hypercubes",                          get(api::hypercubes::list_hypercubes))
        .route("/api/hypercubes",                          post(api::hypercubes::create_hypercube))
        .route("/api/hypercubes/{id}",                     get(api::hypercubes::get_hypercube))
        .route("/api/hypercubes/{id}",                     put(api::hypercubes::update_hypercube))
        .route("/api/hypercubes/{id}",                     delete(api::hypercubes::delete_hypercube))
        .route("/api/hypercubes/{id}/combinations",        get(api::hypercubes::get_combinations))
        // Module 3 — Portfolios
        .route("/api/portfolios",                          get(api::portfolios::list_portfolios))
        .route("/api/portfolios",                          post(api::portfolios::create_portfolio))
        .route("/api/portfolios/{id}",                     get(api::portfolios::get_portfolio))
        .route("/api/portfolios/{id}",                     put(api::portfolios::update_portfolio))
        .route("/api/portfolios/{id}",                     delete(api::portfolios::delete_portfolio))
        .route("/api/portfolios/{id}/vectors",             post(api::portfolios::add_vector_to_portfolio))
        .route("/api/portfolios/{id}/vectors/{vid}",       delete(api::portfolios::remove_vector_from_portfolio))
        .route("/api/portfolios/{id}/schedules",           post(api::portfolios::add_schedule_to_portfolio))
        .route("/api/portfolios/{id}/schedules/{sid}",     delete(api::portfolios::remove_schedule_from_portfolio))
        .route("/api/portfolios/{id}/pairs",               post(api::portfolios::create_pair))
        .route("/api/portfolios/{id}/pairs/{pid}",         delete(api::portfolios::delete_pair))
        // Module 3 — Outstanding vectors
        .route("/api/outstanding-vectors",                 get(api::portfolios::list_vectors))
        .route("/api/outstanding-vectors",                 post(api::portfolios::create_vector))
        .route("/api/outstanding-vectors/{id}",            get(api::portfolios::get_vector))
        .route("/api/outstanding-vectors/{id}",            put(api::portfolios::update_vector))
        .route("/api/outstanding-vectors/{id}",            delete(api::portfolios::delete_vector))
        // Module 3 — Amortization schedules
        .route("/api/amort-schedules",                     get(api::portfolios::list_schedules))
        .route("/api/amort-schedules",                     post(api::portfolios::create_schedule))
        .route("/api/amort-schedules/{id}",                get(api::portfolios::get_schedule))
        .route("/api/amort-schedules/{id}",                put(api::portfolios::update_schedule))
        .route("/api/amort-schedules/{id}",                delete(api::portfolios::delete_schedule))
        // Module 6 — Executions
        .route("/api/executions",        get(api::executions::list_executions))
        .route("/api/executions",        post(api::executions::create_execution))
        .route("/api/executions/{id}",   get(api::executions::get_execution))
        .route("/api/executions/{id}",   delete(api::executions::delete_execution))
        // Module 5 — Studies
        .route("/api/studies",                          get(api::studies::list_studies))
        .route("/api/studies",                          post(api::studies::create_study))
        .route("/api/studies/{id}",                     get(api::studies::get_study))
        .route("/api/studies/{id}",                     put(api::studies::update_study))
        .route("/api/studies/{id}",                     delete(api::studies::delete_study))
        .route("/api/studies/{id}/units",               post(api::studies::add_unit))
        .route("/api/studies/{id}/units/{uid}",         delete(api::studies::remove_unit))
        // Module 4 — Study units
        .route("/api/study-units",                              get(api::study_units::list_study_units))
        .route("/api/study-units",                              post(api::study_units::create_study_unit))
        .route("/api/study-units/{id}",                         get(api::study_units::get_study_unit))
        .route("/api/study-units/{id}",                         put(api::study_units::update_study_unit))
        .route("/api/study-units/{id}",                         delete(api::study_units::delete_study_unit))
        .route("/api/study-units/{id}/validate",                post(api::study_units::validate_study_unit))
        .route("/api/study-units/{id}/assignments",             post(api::study_units::create_assignment))
        .route("/api/study-units/{id}/assignments/{aid}",       put(api::study_units::update_assignment))
        .route("/api/study-units/{id}/assignments/{aid}",       delete(api::study_units::delete_assignment))
        .layer(cors)
        .with_state(state)
        .fallback(serve_frontend);

    let listen_addr = std::env::var("LISTEN_ADDR")
        .unwrap_or_else(|_| "127.0.0.1:3000".to_string());
    let addr: SocketAddr = listen_addr.parse().expect("Invalid LISTEN_ADDR");
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    tracing::info!("FTP Simulator backend listening on http://{}", addr);

    axum::serve(listener, app).await.unwrap();
}
