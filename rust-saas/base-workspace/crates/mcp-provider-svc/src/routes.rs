use axum::Router;
use tower_http::cors::{CorsLayer, Any};
// use crate::state::{AppState};

use crate::tools_handler::McpToolsHandler;
use rmcp::transport::{StreamableHttpServerConfig,StreamableHttpService};
use rmcp::transport::streamable_http_server::session::local::LocalSessionManager;
use core::time::Duration;


pub fn create_router() -> Router {

    let hosts:Vec<String> = vec!["localhost".into(), "127.0.0.1".into(), "::1".into(),"192.168.0.108".into()];
    let origins:Vec<String> = vec![];

     let service: StreamableHttpService<McpToolsHandler, LocalSessionManager> = StreamableHttpService::new(|| Ok(McpToolsHandler::new()),
        Default::default(),
        StreamableHttpServerConfig::default()
            .with_stateful_mode(false)
            .with_allowed_hosts(hosts)
            .with_sse_keep_alive(Some(core::time::Duration::from_secs(60)))
            ,
    );


    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    Router::new()
        .nest_service("/", service)
        // .with_state(AppState::new())
        .layer(cors)
}