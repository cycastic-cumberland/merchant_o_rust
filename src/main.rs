use crate::config::app_config::{ApplicationConfig, RedirectionReader};
use log::{log, Level};
use tokio::fs;
use std::{convert::Infallible, net::SocketAddr, sync::Arc};
use hyper::{service::{make_service_fn, service_fn}, Body, Client, Request, Response, Server, Uri};
use std::time::{SystemTime, UNIX_EPOCH};
use hyper::server::conn::AddrStream;

mod config;

async fn shutdown_signal() {
    // Wait for the CTRL+C signal
    tokio::signal::ctrl_c()
        .await
        .expect("Failed to install CTRL+C signal handler");
}

fn make_error_response(client_addr: &String, uri: &String, error: &String, status: u16) -> Response<Body> {
    log!(Level::Error, "| {:<15} | {}", client_addr, error);
    let curr = SystemTime::now();
    match Response::builder()
        .status(status)
        .header("Content-Type", "application/json")
        .body(Body::from(format!("{{ \"error\": \"{}\", \
        \"status\": {}, \"path\": \"{}\", \"timestamp\": {} }}",
                                 error,
                                 status,
                                 uri,
                                 curr.duration_since(UNIX_EPOCH).unwrap().as_millis()))){
        Ok(v) => v,
        Err(fatal_err) => panic!("Fatal error encountered during remapping: {}", fatal_err.to_string())
    }
}

async fn handle_request(req: Request<Body>, target: Arc<RedirectionReader>, client_addr: String) -> Result<Response<Body>, Infallible> {
    let epoch = SystemTime::now();
    let uri = req.uri().to_string();
    if let Some(target_url) = target.match_uri(&uri).await{
        let target_uri: Uri = target_url.parse().unwrap();
        let method = req.method().clone();
        let mut builder = Request::builder()
            .uri(target_uri)
            .method(&method)
            .version(req.version());
        if let Some(headers) = builder.headers_mut(){
            *headers = req.headers().clone();
        }
        let forwarded_req = builder
            .body(req.into_body())
            .expect("Failed to build forwarded request");

        // Create a new Hyper client
        let client = Client::new();
        // Forward the request to the target URI and get the response
        let res = match client.request(forwarded_req).await {
            Ok(v) => v,
            Err(e) => {
                let error_response = format!("Error encountered during remapping: {}", e.to_string());
                make_error_response(&client_addr, &uri, &error_response, 500)
            }
        };
        let resolved = SystemTime::now();
        log!(Level::Info, "| {:<15} | Remapped \"{} {} {} -> {}\" in {} us",
            &client_addr,
            res.status().as_str(),
            method.to_string(),
            uri,
            &target_url,
            resolved.duration_since(epoch).unwrap().as_micros());

        // Return the response from the target server
        return Ok(res);

    }
    let remap_error = format!("Failed to find a remap target for {}", &uri);
    return Ok(make_error_response(&client_addr, &uri, &remap_error, 404));
}

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let cfg_path = std::env::var("APP_CONFIG_PATH").expect("Environment variable APP_CONFIG_PATH not set");
    let cfg_content = match fs::read_to_string(cfg_path).await {
        Ok(v) => v,
        Err(e) => panic!("Failed to read application configuration with exception: {}", e.to_string())
    };
    let config: ApplicationConfig = match serde_json::from_str(&cfg_content) {
        Ok(cfg) => cfg,
        Err(e) => panic!("Failed to deserialize application configuration with exception: {}", e.to_string())
    };
    const PORT: u16 = 8188;
    let addr = SocketAddr::from(([0, 0, 0, 0], PORT));
    std::env::set_var("RUST_LOG", config.log_level);
    std::env::set_var("RUST_BACKTRACE", "1");
    env_logger::init();
    log!(Level::Info, "| {:<15} | Merchant O' Rust will be online at {}", "internal", addr.to_string());
    let reader0 = Arc::new(RedirectionReader::new(config.map));
    let make_svc = make_service_fn(move |conn: &AddrStream| {
        let client_addr = Arc::new(conn.remote_addr().ip().to_string());
        let reader1 = Arc::clone(&reader0);
        async move {
            Ok::<_, Infallible>(service_fn(move |req| {
                let reader2 = Arc::clone(&reader1);
                handle_request(req, reader2, client_addr.clone().as_ref().to_owned())
            }))
        }
    });
    let server = Server::bind(&addr).serve(make_svc);
    let graceful = server.with_graceful_shutdown(shutdown_signal());
    if let Err(e) = graceful.await {
        log!(Level::Error, "| {:<15} | Server error: {}", "internal", e);
    }
    log!(Level::Info, "| {:<15} | Shutting down...", "internal");
    Ok(())
}
