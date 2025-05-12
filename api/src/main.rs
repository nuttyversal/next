use std::sync::Arc;

use axum::Router;
use axum::routing::get;
use nuttyverse_core::content::api::router as content_router;
use nuttyverse_core::content::repository::ContentRepository;
use nuttyverse_core::content::service::ContentService;
use nuttyverse_core::navigator::api::router as navigator_router;
use nuttyverse_core::navigator::repository::NavigatorRepository;
use nuttyverse_core::navigator::service::NavigatorService;
use nuttyverse_core::utilities::api::state::AppState;
use sqlx::postgres::PgPoolOptions;

#[tokio::main]
async fn main() {
	// リンクスタート〜！
	println!("Starting the Nuttyverse server…");

	// Create the database connection pool.
	println!("Connecting to the Nuttyverse database…");
	let database_url = std::env::var("DATABASE_URL")
		.unwrap_or_else(|_| "postgres://nutty@localhost:5432/nuttyverse".to_string());

	let database_pool = PgPoolOptions::new()
		.max_connections(5)
		.connect(&database_url)
		.await
		.expect("Failed to connect to database");

	// Set up application state.
	let content_repository = ContentRepository::new(database_pool.clone());
	let content_service = ContentService::new(content_repository);
	let navigator_repository = NavigatorRepository::new(database_pool.clone());
	let navigator_service = NavigatorService::new(navigator_repository);

	let app_state = Arc::new(AppState {
		content_service,
		navigator_service,
	});

	let router = Router::new()
		.route("/", get(|| async { "Hello world!" }))
		.merge(content_router(app_state.clone()))
		.merge(navigator_router(app_state.clone()));

	let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
	println!("Listening @ 0.0.0.0:3000…");

	axum::serve(listener, router).await.unwrap();
}
