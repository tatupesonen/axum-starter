use axum::{
    routing::{get, post},
    http::{StatusCode},
    response::{IntoResponse},
    Json, Router,
};
use opentelemetry::global;
use tower_http::{trace::TraceLayer};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use serde::{Deserialize, Serialize};
use tracing::{instrument};
use std::{net::SocketAddr, io::stdout};

#[tokio::main]
async fn main() {
    // Create a new OpenTelemetry pipeline
    let tracer = opentelemetry_jaeger::new_agent_pipeline().with_service_name("tracing-test").install_simple().unwrap();
    tracing_subscriber::registry()
    .with(
        tracing_subscriber::EnvFilter::try_from_default_env()
            .unwrap_or_else(|_| "axumtest=debug,tower_http=debug".into()),
    )
    .with(tracing_subscriber::fmt::layer())
    .with(tracing_opentelemetry::layer().with_tracer(tracer))
    .init();

    let app = Router::new()
        // `GET /` goes to `root`
        .route("/", get(root))
        // `POST /users` goes to `create_user`
        .route("/users", post(create_user))
        .layer(TraceLayer::new_for_http());

    // run our app with hyper
    // `axum::Server` is a re-export of `hyper::Server`
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    tracing::info!("listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service_with_connect_info::<SocketAddr>())
        .await
        .unwrap();
}

// basic handler that responds with a static string
#[instrument]
async fn root() -> &'static str {
    tracing::info!("Calling root");

    "Hello, World!"
}

#[instrument]
async fn create_user(
    Json(payload): Json<CreateUser>,
) -> impl IntoResponse {
    let user = User {
        id: 1337,
        username: payload.username,
    };

    (StatusCode::CREATED, Json(user))
}



#[derive(Deserialize, Debug)]
struct CreateUser {
    username: String,
}

#[derive(Serialize, Debug)]
struct User {
    id: u64,
    username: String,
}
