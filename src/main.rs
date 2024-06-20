use axum::{
    body::Body,
    extract::{Request, State},
    http::uri::Uri,
    response::{IntoResponse, Response},
    routing::any,
    Router,
};
use hyper::StatusCode;
use hyper_util::{client::legacy::connect::HttpConnector, rt::TokioExecutor};

type Client = hyper_util::client::legacy::Client<HttpConnector, Body>;

#[tokio::main]
async fn main() {
    let client: Client =
        hyper_util::client::legacy::Client::<(), ()>::builder(TokioExecutor::new())
            .build(HttpConnector::new());

    let app = Router::new()
        .route("/api", any(backend_handler))
        .route("/api/", any(backend_handler))
        .route("/api/*be", any(backend_handler))
        // .route("/", any(backend_handler))
        // .route("/*fe", any(backend_handler))
        .route("/", any(frontend_handler))
        .route("/*fe", any(frontend_handler))
        .with_state(client);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8080").await.unwrap();
    println!("proxy listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, app).await.unwrap();
}

async fn frontend_handler(
    State(client): State<Client>,
    mut req: Request,
) -> Result<Response, StatusCode> {
    let path = req.uri().path();
    let path_query = req
        .uri()
        .path_and_query()
        .map(|v| v.as_str())
        .unwrap_or(path);

    let uri = format!("http://0.0.0.0:4000{}", path_query);
    println!("requesting {uri}");

    *req.uri_mut() = Uri::try_from(uri).unwrap();
    dbg!(&req);

    let response = client
        .request(req)
        .await
        .map_err(|_| StatusCode::BAD_REQUEST)?
        .into_response();
    dbg!(&response);
    Ok(response)
}

async fn backend_handler(
    State(client): State<Client>,
    mut req: Request,
) -> Result<Response, StatusCode> {
    dbg!(&req);
    let path = req.uri().path();
    let path_query = req
        .uri()
        .path_and_query()
        .map(|v| v.as_str())
        .unwrap_or(path);

    let uri = format!("http://0.0.0.0:3000{}", path_query);
    println!("requesting {uri}");

    *req.uri_mut() = Uri::try_from(uri).unwrap();

    let response = client
        .request(req)
        .await
        .map_err(|_| StatusCode::BAD_REQUEST)?
        .into_response();
    dbg!(&response);
    Ok(response)
}
