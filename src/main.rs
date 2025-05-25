mod services;
mod types;

use axum::{
    Json, Router,
    extract::Path,
    response::IntoResponse,
    response::sse::{Event, Sse},
    routing::get,
};
use futures_core::Stream;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio::sync::broadcast;
use tokio_stream::StreamExt;
use tokio_stream::wrappers::BroadcastStream;
use tower_http::cors::{Any, CorsLayer};

use crate::services::{MacServiceDiscovery, get_service, scan_services};
use crate::types::{Service, ServiceDiscovery};

#[tokio::main]
async fn main() {
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods([axum::http::Method::GET]);

    let discovery: Arc<dyn ServiceDiscovery> = Arc::new(MacServiceDiscovery);
    let service_cache: Arc<Mutex<Vec<Service>>> = Arc::new(Mutex::new(Vec::new()));
    let (tx, _rx) = broadcast::channel::<Vec<Service>>(16);
    let tx = Arc::new(tx);

    // Background task to refresh service cache and broadcast changes
    let discovery_bg = discovery.clone();
    let cache_bg = service_cache.clone();
    let tx_bg = tx.clone();
    tokio::spawn(async move {
        let mut last_state = Vec::new();
        loop {
            let services = scan_services(&*discovery_bg).await;
            {
                let mut cache = cache_bg.lock().unwrap();
                *cache = services.clone();
            }
            if services != last_state {
                let _ = tx_bg.send(services.clone());
                last_state = services;
            }
            tokio::time::sleep(Duration::from_secs(5)).await;
        }
    });

    let app = Router::new()
        .route("/", get(list_services))
        .route("/:port", get(service_detail))
        .route("/events", get(sse_events))
        .layer(cors)
        .with_state((service_cache, discovery, tx));

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    println!("🚀 Listening on http://{}", addr);
    axum::serve(
        tokio::net::TcpListener::bind(addr).await.unwrap(),
        app.into_make_service(),
    )
    .await
    .unwrap();
}

async fn list_services(
    axum::extract::State((service_cache, ..)): axum::extract::State<(
        Arc<Mutex<Vec<Service>>>,
        Arc<dyn ServiceDiscovery>,
        Arc<broadcast::Sender<Vec<Service>>>,
    )>,
) -> Json<Vec<Service>> {
    let cache = service_cache.lock().unwrap();
    Json(cache.clone())
}

async fn service_detail(
    Path(port): Path<u16>,
    axum::extract::State((.., discovery, _)): axum::extract::State<(
        Arc<Mutex<Vec<Service>>>,
        Arc<dyn ServiceDiscovery>,
        Arc<broadcast::Sender<Vec<Service>>>,
    )>,
) -> impl IntoResponse {
    match get_service(&*discovery, port).await {
        Some(service) => axum::Json(service).into_response(),
        None => axum::http::StatusCode::NOT_FOUND.into_response(),
    }
}

async fn sse_events(
    axum::extract::State((_, _, tx)): axum::extract::State<(
        Arc<Mutex<Vec<Service>>>,
        Arc<dyn ServiceDiscovery>,
        Arc<broadcast::Sender<Vec<Service>>>,
    )>,
) -> Sse<impl Stream<Item = Result<Event, std::convert::Infallible>>> {
    let rx = tx.subscribe();
    let stream = BroadcastStream::new(rx).filter_map(|result| match result {
        Ok(services) => {
            let json = serde_json::to_string(&services).unwrap();
            Some(Ok(Event::default().data(json)))
        }
        Err(_) => None,
    });
    Sse::new(stream).keep_alive(axum::response::sse::KeepAlive::default())
}
