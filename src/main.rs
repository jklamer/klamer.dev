use axum::body::Body;
use axum::handler::Handler;
use axum::http::header::CONTENT_TYPE;
use axum::http::HeaderValue;
use axum::response::{Html, IntoResponse, Response};
use axum::Router;
use axum::routing::get;
use futures::executor;
use tower_http::trace::TraceLayer;
use tracing_subscriber;

use crate::html::{Anchor, Header, Img, IntoHtml};

mod html;

const ICON: &[u8] = include_bytes!("../assests/k_logo.dev.png");
const LOGO: &[u8] = include_bytes!("../assests/klamer.dev.png");

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();
    let app = Router::new()
        .route("/", get(home_page))
        .route("/blog", get(blog_page))
        .route("/favicon.png", get(icon))
        .route("/logo.png", get(logo))
        .layer(TraceLayer::new_for_http());

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    tracing::debug!("Starting server");
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
    } else {
        response
    }
}

async fn home_page() -> Html<String> {
    page(vec![
        Box::new(Img {
            uri: "/logo.png".to_string(),
            alt_text: "Klamer.dev logo".to_string()
        }),
        Box::new(Anchor("/blog".to_string(), Header("Hello world".to_string())))
    ])
}

// write axum handlers needed to set up a blog
async fn blog_page() -> Html<String> {
    page(vec![
            Box::new(Anchor("/".to_string(), Header("Blog".to_string())))
    ])
}

fn page(components: Vec<Box<dyn IntoHtml>>) -> Html<String> {
    Html("<html>".to_string()
        + "<head>
          <title>Klamer.dev</title>
          <link rel=\"icon\" type=\"image/png\" href=\"/favicon.png\">
          <script src=\"https://unpkg.com/htmx.org@1.9.10\" integrity=\"sha384-D1Kt99CQMDuVetoL1lrYwg5t+9QdHe7NLX/SoJYkXDFfX37iInKRy5xLSi8nO7UC\" crossorigin=\"anonymous\"></script>
          </head>"
        + "<body>"
        + components.iter().map(|x| x.html_string()).collect::<Vec<String>>().join("").as_str()
        + "</body>"
        + "</html>")
}


async fn icon() -> &'static [u8] {
    ICON
}

async fn logo() -> &'static [u8] {
    LOGO
}