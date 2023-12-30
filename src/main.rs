mod html;

use axum::Router;
use axum::body::Body;
use axum::http::header::CONTENT_TYPE;
use axum::http::HeaderValue;
use axum::response::{Html, IntoResponse, Response};
use axum::routing::get;
use futures::executor;
use crate::html::{Anchor, Attribute, Attributes, AttributesBuilder, Div, Header, HtmxAttributes, IntoHtml};

#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/", get(home_page))
        .route("/blog", get(blog_page));

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
    } else {
        response
    }
}

async fn home_page() -> Html<String> {
    page(vec![
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
    Html("<html>
        <head>
            <title>Klamer.dev</title>
            <script src=\"https://unpkg.com/htmx.org@1.9.10\" integrity=\"sha384-D1Kt99CQMDuVetoL1lrYwg5t+9QdHe7NLX/SoJYkXDFfX37iInKRy5xLSi8nO7UC\" crossorigin=\"anonymous\"></script>
        </head>
        <body>
     ".to_string()
    + components.iter().map(|x| x.html_string()).collect::<Vec<String>>().join("").as_str()
    + "</body></html>")
}