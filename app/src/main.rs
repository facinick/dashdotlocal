mod services;
mod types;

use axum::{
    Json, Router,
    extract::Path,
    extract::Query,
    response::IntoResponse,
    response::sse::{Event, Sse},
    routing::get,
};
use futures_core::Stream;
use serde::Deserialize;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio::sync::broadcast;
use tokio_stream::StreamExt;
use tokio_stream::wrappers::BroadcastStream;
use tower_http::cors::{Any, CorsLayer};

use crate::services::{MacServiceDiscovery, get_service, scan_services};
use crate::types::{Service, ServiceDiscovery};

#[derive(Deserialize)]
struct ListServicesParams {
    sort_by: Option<String>,
    sort_order: Option<String>,
    page: Option<usize>,
    page_size: Option<usize>,
}

#[derive(serde::Serialize)]
struct PaginatedServices {
    data: Vec<Service>,
    total: usize,
    page: usize,
    page_size: usize,
}

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
    Query(params): Query<ListServicesParams>,
    axum::extract::State((service_cache, ..)): axum::extract::State<(
        Arc<Mutex<Vec<Service>>>,
        Arc<dyn ServiceDiscovery>,
        Arc<broadcast::Sender<Vec<Service>>>,
    )>,
) -> Json<PaginatedServices> {
    let cache = service_cache.lock().unwrap();
    let mut services = cache.clone();
    let total = services.len();

    // Sorting
    let sort_by = params.sort_by.as_deref().unwrap_or("port");
    let sort_order = params.sort_order.as_deref().unwrap_or("asc");
    let valid_fields = [
        "port",
        "status",
        "process",
        "pid",
        "user",
        "protocol",
        "local_address",
        "fd",
        "type_field",
        "device",
        "size_off",
        "node",
        "command_line",
        "exe_path",
        "start_time",
        "ppid",
    ];
    if valid_fields.contains(&sort_by) {
        services.sort_by(|a, b| {
            let ord = match sort_by {
                "port" => a.port.cmp(&b.port),
                "status" => a.status.cmp(&b.status),
                "process" => a.process.cmp(&b.process),
                "pid" => a.pid.cmp(&b.pid),
                "user" => a.user.cmp(&b.user),
                "protocol" => a.protocol.cmp(&b.protocol),
                "local_address" => a.local_address.cmp(&b.local_address),
                "fd" => a.fd.cmp(&b.fd),
                "type_field" => a.type_field.cmp(&b.type_field),
                "device" => a.device.cmp(&b.device),
                "size_off" => a.size_off.cmp(&b.size_off),
                "node" => a.node.cmp(&b.node),
                "command_line" => a.command_line.cmp(&b.command_line),
                "exe_path" => a.exe_path.cmp(&b.exe_path),
                "start_time" => a.start_time.cmp(&b.start_time),
                "ppid" => a.ppid.cmp(&b.ppid),
                _ => std::cmp::Ordering::Equal,
            };
            if sort_order == "desc" {
                ord.reverse()
            } else {
                ord
            }
        });
    }

    // Pagination
    let page_size = params.page_size.unwrap_or(20).min(100);
    let page = params.page.unwrap_or(1).max(1);
    let start = (page - 1) * page_size;
    let paged = services.into_iter().skip(start).take(page_size).collect();
    Json(PaginatedServices {
        data: paged,
        total,
        page,
        page_size,
    })
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
