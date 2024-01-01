use axum::response::Html;
use axum::Router;
use axum::routing::get;
use tower_http::trace::TraceLayer;
use tracing_subscriber;

use crate::html::{Anchor, AttributesBuilder, DivBuilder, Hr, ImgBuilder, IntoHtml, UlistBuilder};
use crate::html::Attribute::{CLASS, WIDTH};

mod html;

const ICON: &[u8] = include_bytes!("../assests/k_logo.dev.png");
const LOGO: &[u8] = include_bytes!("../assests/klamer.dev.png");
const BASE_CSS: &str = include_str!("../css/base.css");

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();
    let app = Router::new()
        .route("/", get(home_page))
        .route("/blog", get(blog_page))
        .route("/favicon.png", get(icon))
        .route("/logo.png", get(logo))
        .route("/base.css", get(base_css))
        .layer(TraceLayer::new_for_http());

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    tracing::debug!("Starting server");
    tokio::spawn(async move {
        println!("Listening on http://localhost:3000");
    });
    axum::serve(listener, app).await.unwrap();
}

async fn home_page() -> Html<String> {
    page(vec!["Home content".into()])
}

// write axum handlers needed to set up a blog
async fn blog_page() -> Html<String> {
    page(vec!["Blog content".into()])
}

fn page(mut content: Vec<Box<dyn IntoHtml>>) -> Html<String> {
    //content.insert(0, Box::new(Hr));
    let page_components: Vec<Box<dyn IntoHtml>> = vec![
        Box::new(DivBuilder::default()
            .element(Anchor("/".to_string(), ImgBuilder::default()
                .uri("/logo.png".to_string())
                .alt_text("Klamer.dev logo".to_string())
                .attributes(AttributesBuilder::default()
                    .attribute(WIDTH(150))
                    .build().unwrap())
                .build().unwrap()))
            .attributes(AttributesBuilder::default()
                .attribute(CLASS(vec!["Logo".to_string()]))
                .build().unwrap())
            .build().unwrap()),
        Box::new(
            DivBuilder::default()
                .element(UlistBuilder::default()
                    .item(Anchor("/".to_string(), "Home"))
                    .item(Anchor("/blog".to_string(), "Blog"))
                    .attributes(AttributesBuilder::default()
                        .attribute(CLASS(vec!["section-items".to_string()]))
                        .build().unwrap())
                    .item_attributes(AttributesBuilder::default()
                        .attribute(CLASS(vec!["section-item".to_string()]))
                        .build().unwrap())
                    .build().unwrap())
                .attributes(AttributesBuilder::default()
                    .attribute(CLASS(vec!["Sections".to_string()]))
                    .build().unwrap())
                .build().unwrap()
        ),
        Box::new(
            DivBuilder::default()
                .elements(content)
                .attributes(AttributesBuilder::default()
                    .attribute(CLASS(vec!["Content".to_string()]))
                    .build().unwrap())
                .build().unwrap()
        )
    ];
    Html("<html>".to_string()
        + "<head>
          <title>Klamer.dev</title>
          <link rel=\"icon\" type=\"image/png\" href=\"/favicon.png\">
          <link rel=\"stylesheet\" href=\"/base.css\">
          <script src=\"https://unpkg.com/htmx.org@1.9.10\" integrity=\"sha384-D1Kt99CQMDuVetoL1lrYwg5t+9QdHe7NLX/SoJYkXDFfX37iInKRy5xLSi8nO7UC\" crossorigin=\"anonymous\"></script>
          </head>"
        + "<body>"
        + DivBuilder::default()
        .elements(page_components)
        .attributes(AttributesBuilder::default()
            .attribute(CLASS(vec!["Container".to_string(),"center".to_string()]))
            .build().unwrap()).build().unwrap().html_string().as_str()
        + "</body>"
        + "</html>")
}




async fn icon() -> &'static [u8] {
    ICON
}

async fn logo() -> &'static [u8] {
    LOGO
}

async fn base_css() -> &'static str {
    BASE_CSS
}