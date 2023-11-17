use crate::config::{app_config::ApplicationConfig, redirection_reader::RedirectionReader};
use log::{log, Level};
use tokio::{fs, io};
use std::{convert::Infallible, net::SocketAddr, sync::Arc};
use hyper::{service::{make_service_fn, service_fn}, Body, Client, Request, Response, Server, Uri};
use tokio::sync::RwLock;
use std::time::{SystemTime};

mod config;

// const PUBLICIZED_HOST: &'static str = "0.0.0.0";

async fn shutdown_signal() {
    // Wait for the CTRL+C signal
    tokio::signal::ctrl_c()
        .await
        .expect("Failed to install CTRL+C signal handler");
}

async fn handle_request(req: Request<Body>, target: Arc<RedirectionReader>) -> Result<Response<Body>, Infallible> {
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
                log!(Level::Error, "{}", error_response);
                match Response::builder()
                    .status(500)
                    .header("Content-Type", "application/json")
                    .body(Body::from(format!("{{ \"error\": \"{}\" }}", error_response))){
                    Ok(v) => v,
                    Err(fatal_err) => panic!("Fatal error encountered during remapping: {}", fatal_err.to_string())
                }
            }
        };
        let resolved = SystemTime::now();
        log!(Level::Info, "Remapped \"{}({}) {} -> {}\" in {} us",
            method.to_string(),
            res.status().as_str(),
            uri,
            &target_url,
            resolved.duration_since(epoch).unwrap().as_micros());

        // Return the response from the target server
        return Ok(res);

    }
    let remap_error = format!("Failed to find a remap target for {}", &uri);
    log!(Level::Error, "{}", &remap_error);
    return Ok(match Response::builder()
        .status(404)
        .header("Content-Type", "application/json")
        .body(Body::from(format!("{{ \"error\": \"{}\" }}", remap_error))){
        Ok(v) => v,
        Err(fatal_err) => panic!("Fatal error encountered during remapping: {}", fatal_err.to_string())
    });
}

async fn async_helper<T>(ret: T) -> T {
    ret
}

#[tokio::main]
async fn main() -> io::Result<()> {
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
    log!(Level::Info, "Merchant O' Rust will be online at {}", addr.to_string());
    let pinned0 = Arc::new(RedirectionReader::new(RwLock::new(config.map)));
    let pinned_fn = Arc::new(move |req: Request<Body>| {
        let pinned1 = Arc::clone(&pinned0);
        handle_request(req, pinned1)
    });
    let make_svc = make_service_fn(move |_conn| {
        // service_fn converts our function into a `Service`
        let func = pinned_fn.as_ref().to_owned();
        async_helper(Ok::<_, Infallible>(service_fn(func)))
    });
    let server = Server::bind(&addr).serve(make_svc);
    let graceful = server.with_graceful_shutdown(shutdown_signal());
    if let Err(e) = graceful.await {
        log!(Level::Error, "Server error: {}", e);
    }
    log!(Level::Info, "Shutting down...");
    Ok(())
}
