use axum::Router;
use axum::body::Body;
use axum::http::header::CONTENT_TYPE;
use axum::http::HeaderValue;
use axum::response::{Html, IntoResponse, Response};
use axum::routing::get;
use futures::executor;
use tower::ServiceBuilder;
use tower::util::MapResponseLayer;

#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/", get(home_page))
        .route("/blog", get(blog_index))
        .layer(ServiceBuilder::new()
                   .layer(MapResponseLayer::new(include_htmx)));

    // run our app with hyper, listening globally on port 3000
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    tokio::spawn(async move {
        println!("Listening on http://localhost:3000");
    });
    axum::serve(listener, app).await.unwrap();
}

fn include_htmx(response: Response<Body>) -> Response<Body> {
    if response.headers().get(CONTENT_TYPE) == Some(&HeaderValue::from_static("text/html; charset=utf-8")) {
        let mut body = executor::block_on_stream(response.into_body().into_data_stream());
        let mut bytes = Vec::new();
        while let Some(frame) = body.next() {
            bytes.extend_from_slice(&frame.unwrap());
        }
        bytes.extend_from_slice(b"\n<script src=\"https://unpkg.com/htmx.org@1.9.10\" integrity=\"sha384-D1Kt99CQMDuVetoL1lrYwg5t+9QdHe7NLX/SoJYkXDFfX37iInKRy5xLSi8nO7UC\" crossorigin=\"anonymous\"></script>");
        //let bytes = Bytes::from(bytes);
        Html(bytes).into_response()
    }else {
        response
    }
}

async fn home_page() -> Html<&'static str> {
    Html("<h1 hx-swap='outerHTML' hx-get='/blog'>Hello, World!</h1>")
}

// write axum handlers needed to set up a blog
async fn blog_index() -> Html<&'static str> {
    Html("<h1 hx-swap='outerHTML' hx-get='/'>Blog Index</h1>")
}