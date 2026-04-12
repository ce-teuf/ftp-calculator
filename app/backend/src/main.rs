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

static DIST: Dir<'static> = include_dir!("$CARGO_MANIFEST_DIR/../dashboard/dist");

async fn serve_frontend(req: Request) -> Response {
    let path = req.uri().path().trim_start_matches('/');

    // Try exact match first, then fall back to index.html for SPA routing
    let file = DIST.get_file(path)
        .or_else(|| DIST.get_file("index.html"));

    match file {
        Some(f) => {
            let mime = mime_guess::from_path(f.path())
                .first_or_octet_stream();
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
        .unwrap_or_else(|_| {
            "postgresql://ftp:ftp_local@127.0.0.1:5432/ftp_simulator".to_string()
        });

    let pool = PgPoolOptions::new()
        .max_connections(10)
        .connect(&database_url)
        .await
        .expect("Failed to connect to PostgreSQL. Make sure DATABASE_URL is set.");

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
        // Curves
        .route("/api/curves", get(api::curves::list_curves))
        .route("/api/curves", post(api::curves::create_curve))
        .route("/api/curves/{id}", get(api::curves::get_curve))
        .route("/api/curves/{id}", put(api::curves::update_curve))
        .route("/api/curves/{id}", delete(api::curves::delete_curve))
        // Portfolios
        .route("/api/portfolios", get(api::portfolio::list_portfolios))
        .route("/api/portfolios", post(api::portfolio::create_portfolio))
        .route("/api/portfolios/{id}", get(api::portfolio::get_portfolio))
        .route("/api/portfolios/{id}", delete(api::portfolio::delete_portfolio))
        // Positions
        .route("/api/portfolios/{id}/positions", get(api::portfolio::list_positions))
        .route("/api/portfolios/{id}/positions", post(api::portfolio::add_position))
        .route("/api/portfolios/{id}/positions/bulk", post(api::portfolio::bulk_import_positions))
        .route("/api/positions/{id}", delete(api::portfolio::delete_position))
        // Runoff models
        .route("/api/runoff", get(api::runoff::list_runoff_models))
        .route("/api/runoff", post(api::runoff::create_runoff_model))
        .route("/api/runoff/{id}", get(api::runoff::get_runoff_model))
        .route("/api/runoff/{id}", delete(api::runoff::delete_runoff_model))
        // Executions
        .route("/api/executions", get(api::executions::list_executions))
        .route("/api/executions/diff", get(api::executions::diff_executions))
        .route("/api/executions/{id}", get(api::executions::get_execution))
        .route("/api/executions/{id}/inputs", get(api::executions::get_execution_inputs))
        // Compute (runs + persists execution)
        .route("/api/compute", post(api::compute::run_calculation))
        // Analytics
        .route("/api/analytics/portfolio-nim", get(api::analytics::portfolio_nim))
        // Datasets
        .route("/api/datasets", get(api::datasets_api::list_datasets))
        .route("/api/datasets", post(api::datasets_api::create_dataset))
        .route("/api/datasets/{id}", delete(api::datasets_api::delete_dataset))
        .route("/api/datasets/{id}/contracts", get(api::datasets_api::list_dataset_contracts))
        .route("/api/datasets/{id}/freeze", get(api::datasets_api::freeze_dataset))
        .route("/api/datasets/{id}/items", post(api::datasets_api::add_items_to_dataset))
        .route("/api/datasets/{id}/summary", get(api::datasets_api::dataset_summary))
        .route("/api/datasets/upload", post(api::datasets_api::upload_contracts_csv))
        .route("/api/datasets/available", get(api::datasets_api::list_available_datasets))
        .route("/api/datasets/fs/{folder}/load", post(api::datasets_api::load_fs_dataset))
        .route("/api/datasets/{id}/export-zip", get(api::datasets_api::export_dataset_zip))
        .route("/api/datasets/ingest", post(api::datasets_api::ingest_files))
        // Rate series (historical time series for curve building)
        .route("/api/rate-series/names", get(api::rate_series::list_series_names))
        .route("/api/rate-series", get(api::rate_series::query_rate_series))
        // Portfolios V3
        .route("/api/portfolios-v3", get(api::portfolios_v3::list_portfolios))
        .route("/api/portfolios-v3", post(api::portfolios_v3::create_portfolio))
        .route("/api/portfolios-v3/{id}", get(api::portfolios_v3::get_portfolio))
        .route("/api/portfolios-v3/{id}", delete(api::portfolios_v3::delete_portfolio))
        .route("/api/portfolios-v3/{id}/rows/upload", post(api::portfolios_v3::upload_portfolio_row))
        .route("/api/portfolios-v3/{id}/rows/{row_id}", get(api::portfolios_v3::get_portfolio_row))
        .route("/api/portfolio-rows/{row_id}", delete(api::portfolios_v3::delete_portfolio_row))
        // Executions V3
        .route("/api/executions-v3", get(api::executions_v3::list_executions))
        .route("/api/executions-v3", post(api::executions_v3::run_execution))
        .route("/api/executions-v3/{id}", get(api::executions_v3::get_execution))
        .route("/api/executions-v3/{id}", delete(api::executions_v3::delete_execution))
        // Studies (V3)
        .route("/api/studies", get(api::studies::list_studies))
        .route("/api/studies", post(api::studies::create_study))
        .route("/api/studies/{id}", get(api::studies::get_study))
        .route("/api/studies/{id}", put(api::studies::update_study))
        .route("/api/studies/{id}", delete(api::studies::delete_study))
        .route("/api/studies/{id}/linkers", post(api::studies::add_linker_to_study))
        .route("/api/studies/{id}/linkers/{linker_id}", delete(api::studies::remove_linker_from_study))
        // Linkers (V3)
        .route("/api/linkers", get(api::linkers::list_linkers))
        .route("/api/linkers", post(api::linkers::create_linker))
        .route("/api/linkers/{id}", get(api::linkers::get_linker))
        .route("/api/linkers/{id}", delete(api::linkers::delete_linker))
        // Curve Cubes (V3)
        .route("/api/cubes", get(api::cubes::list_cubes))
        .route("/api/cubes", post(api::cubes::create_cube))
        .route("/api/cubes/{id}", get(api::cubes::get_cube))
        .route("/api/cubes/{id}", put(api::cubes::update_cube))
        .route("/api/cubes/{id}", delete(api::cubes::delete_cube))
        // Curve Stacks (V3)
        .route("/api/stacks", get(api::stacks::list_stacks))
        .route("/api/stacks", post(api::stacks::create_stack))
        .route("/api/stacks/generate-combinations", post(api::stacks::generate_combinations))
        .route("/api/stacks/{id}", get(api::stacks::get_stack))
        .route("/api/stacks/{id}", put(api::stacks::update_stack))
        .route("/api/stacks/{id}", delete(api::stacks::delete_stack))
        // Export
        .route("/api/export", get(api::export::export_backup))
        .layer(cors)
        .with_state(state)
        .fallback(serve_frontend);

    let listen_addr = std::env::var("LISTEN_ADDR")
        .unwrap_or_else(|_| "127.0.0.1:3000".to_string());
    let addr: SocketAddr = listen_addr.parse()
        .expect("Invalid LISTEN_ADDR");
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    tracing::info!("FTP Simulator backend listening on http://{}", addr);
    tracing::info!("Dashboard available at http://{}", addr);

    axum::serve(listener, app).await.unwrap();
}
