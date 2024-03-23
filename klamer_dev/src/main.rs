#[macro_use]
extern crate lazy_static;

use std::collections::HashMap;
use std::iter::Iterator;
use std::net::{Ipv4Addr, SocketAddr, TcpListener};
use std::time::Duration;

use axum::{BoxError, Router};
use axum::extract::{Host, Path};
use axum::handler::HandlerWithoutStateExt;
use axum::http::{StatusCode, Uri};
use axum::response::{Html, Redirect};
use axum::routing::get;
use clap::Parser;
use futures::StreamExt;
use rustls_acme::AcmeConfig;
use tokio::signal::ctrl_c;
use tower_governor::governor::GovernorConfigBuilder;
use tower_governor::GovernorLayer;
use tower_http::trace::TraceLayer;
use tracing_subscriber;

use blog_files_macro::list_blog_files;
use rustls_acme_cache::{AcmeS3Cache, NoAccountAcmeS3Cache};

use crate::html::{Anchor, AttributesBuilder, DivBuilder, Header2, ImgBuilder, IntoHtml, UlistBuilder};
use crate::html::Attribute::{CLASS, HeightEm, HeightPercent, WIDTH, WidthPercent, WidthVw};

mod html;

const ICON: &[u8] = include_bytes!("../assests/k_logo.dev.png");
const LOGO: &[u8] = include_bytes!("../assests/klamer.dev.png");
const BASE_CSS: &str = include_str!("../css/base.css");
const FOUR04: &str = include_str!("../assests/404.html");
const BLOG_POST_CONTENT: &[(&'static str, &'static str)] = &list_blog_files!();

lazy_static! {
    static ref POST_NAMES: Vec<&'static str> = BLOG_POST_CONTENT.iter().map(|b| get_post_name(b.0)).collect();
    static ref POSTS: HashMap<&'static str, &'static str> = BLOG_POST_CONTENT.iter().map(|b| (get_post_name(b.0), b.1)).collect();
}

#[derive(Parser, Debug)]
struct TlsArgs {
    /// Domains
    #[clap(short, required = true)]
    domains: Vec<String>,

    /// Contact info
    #[clap(short)]
    email: Vec<String>,

    // cert store s3 bucket
    #[clap(short, long, required = true)]
    bucket: String,

    /// Use Let's Encrypt production environment
    /// (see https://letsencrypt.org/docs/staging-environment/)
    #[clap(long)]
    prod: bool,

    #[clap(short, long, default_value = "443")]
    port: u16,

    #[clap(long, default_value = "80")]
    http_port: u16,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();
    tracing::debug!("Initing server");
    let deployed_env =  std::env::var("DEPLOYED_ENV").is_ok();
    tracing::debug!("Deployed env: {}", deployed_env);

    // little rate limiting
    // Allow bursts with up to 10 requests per IP address
    // and replenishes one element every two hundred millis
    let governor_conf = Box::new(
        GovernorConfigBuilder::default()
            .per_millisecond(200)
            .burst_size(10)
            .finish()
            .unwrap(),
    );

    let governor_limiter = governor_conf.limiter().clone();
    tokio::spawn(async move {
        loop {
            tokio::time::sleep(Duration::from_secs(30)).await;
            tracing::debug!("rate limiting storage size: {}", governor_limiter.len());
            governor_limiter.retain_recent();
        }
    });


    let app = Router::new()
        .route("/", get(home_page))
        .route("/blog", get(blog_page))
        .route("/blog/:post_name}", get(blog_post))
        .route("/annie", get(annie_page))
        .route("/favicon.png", get(icon))
        .route("/logo.png", get(logo))
        .route("/base.css", get(base_css))
        .fallback(four04)
        .layer(GovernorLayer{ config: Box::leak(governor_conf)})
        .layer(TraceLayer::new_for_http());

    tracing::info!("Starting server");
    if deployed_env {
        let args = TlsArgs::parse();
        tracing::info!("Args: {:?}", args);
        let addr = SocketAddr::from((Ipv4Addr::UNSPECIFIED, args.port));
        tokio::spawn(async move {
            tracing::debug!("Running with TLS");
            tracing::debug!("Listening on 0.0.0.0:{:?}", args.port);
        });
        let cert_cache= AcmeS3Cache::new(args.bucket, "certs".to_string());

        let mut state = AcmeConfig::new(args.domains)
            .contact(args.email.iter().map(|e| format!("mailto:{}", e)))
            .directory_lets_encrypt(args.prod)
            .cache_compose(cert_cache, NoAccountAcmeS3Cache)
            .state();
        let acceptor = state.axum_acceptor(state.default_rustls_config());
        tokio::spawn(async move {
            loop {
                match state.next().await.unwrap() {
                    Ok(ok) => {
                        tracing::info!("event: {:?}", ok);
                    }
                    Err(err) => {
                        tracing::error!("event: {:?}", err);
                    }
                }
            }
        });
        tokio::spawn(redirect_http_to_https(args.port, args.http_port));
        axum_server::from_tcp(TcpListener::bind(addr).unwrap())
            .acceptor(acceptor)
            .serve(app.into_make_service_with_connect_info::<SocketAddr>())
            .await.unwrap();
    } else {
        let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
        tokio::spawn(async move {
            tracing::debug!("Running without TLS");
            tracing::debug!("Listening on http://localhost:3000");
        });
        axum::serve(listener, app.into_make_service_with_connect_info::<SocketAddr>())
            .with_graceful_shutdown(shutdown_signal_http())
            .await.unwrap();
    }
}

async fn shutdown_signal_http() {
    ctrl_c().await.expect("failed to install ctrl+c handler");
    tracing::info!("Shutting down signal received, shutting down server");
}

#[allow(dead_code)]
async fn redirect_http_to_https(https_port: u16, http_port: u16) {
    fn make_https(host: String, uri: Uri, https_port: u16, http_port: u16) -> Result<Uri, BoxError> {
        let mut parts = uri.into_parts();

        parts.scheme = Some(axum::http::uri::Scheme::HTTPS);

        if parts.path_and_query.is_none() {
            parts.path_and_query = Some("/".parse().unwrap());
        }

        let https_host = host.replace(&http_port.to_string(), &https_port.to_string());
        parts.authority = Some(https_host.parse()?);

        Ok(Uri::from_parts(parts)?)
    }

    let redirect = move |Host(host): Host, uri: Uri| async move {
        match make_https(host, uri, https_port, http_port) {
            Ok(uri) => Ok(Redirect::permanent(&uri.to_string())),
            Err(error) => {
                tracing::warn!(%error, "failed to convert URI to HTTPS");
                Err(StatusCode::BAD_REQUEST)
            }
        }
    };

    let addr = SocketAddr::from((Ipv4Addr::UNSPECIFIED, http_port));
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    tracing::debug!("listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, redirect.into_make_service())
        .with_graceful_shutdown(shutdown_signal_http())
        .await
        .unwrap();
}

fn get_post_name(file_name: &str) -> &str {
    std::path::Path::new(file_name).file_name().unwrap().to_str().unwrap().trim_end_matches(".html")
}

async fn home_page() -> Html<String> {
    page(vec!["Home content".into()], false)
}

// write axum handlers needed to set up a blog
async fn blog_page() -> Html<String> {
    let mut post_list_builder = UlistBuilder::default()
        .item_attributes(AttributesBuilder::default()
            .attribute(CLASS(vec!["post-list".to_string()]))
            .build().unwrap());
    for post_name in POST_NAMES.iter() {
        post_list_builder = post_list_builder.item(Anchor(format!("/blog/{post_name}"), *post_name))
    }

    page(vec![Header2("Posts".to_string()).into(), post_list_builder.build().unwrap().into()], true)
}

async fn blog_post(Path(post_name): Path<String>) -> Html<String> {
    page(vec![POSTS.get(post_name.as_str()).unwrap_or(&FOUR04).into()], true)
}

async fn annie_page() -> Html<String> {
    page(vec!["She's the best".into()], true)
}

async fn four04() -> Html<String> {
    page(vec![FOUR04.into()], false)
}

fn page(content: Vec<Box<dyn IntoHtml>>, include_footer: bool) -> Html<String> {
    let top_nav: Vec<Box<dyn IntoHtml>> = vec![
        Box::new(DivBuilder::default()
            .element(Anchor("/".to_string(), ImgBuilder::default()
                .uri("/logo.png".to_string())
                .alt_text("Klamer.dev logo".to_string())
                .attributes(AttributesBuilder::default()
                    .attribute(WidthVw(150))
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
        .element(DivBuilder::default()
            .elements(top_nav)
            .attributes(AttributesBuilder::default()
                .attribute(CLASS(vec!["Container".to_string()]))
                .build().unwrap())
            .build().unwrap())
        .element(DivBuilder::default()
            .elements(content)
            .attributes(AttributesBuilder::default()
                .attribute(CLASS(vec!["Content".to_string()]))
                .build().unwrap())
            .build().unwrap())
        .element(if include_footer { "<footer>©2024 Jack Klamer<p>Source: " } else { "" })
        .element(if include_footer { Anchor("https://github.com/jklamer/klamer.dev".to_string(), "https://github.com/jklamer/klamer.dev") } else { Anchor("https://github.com/jklamer/klamer.dev".to_string(), "") })
        .attributes(AttributesBuilder::default()
            .attribute(CLASS(vec!["center".to_string()]))
            .build().unwrap())
        .build().unwrap()
        .html_string().as_str()
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

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_post_name() {
        assert_eq!(get_post_name("/somebullshit/postname"), "postname");
        assert_eq!(get_post_name("/somebullshit/postname.html"), "postname");
        assert_eq!(get_post_name("somebullshit/postname"), "postname");
        assert_eq!(get_post_name("somebullshit/postname.html"), "postname");
        assert_eq!(get_post_name("a/b/c/d/e/r/postname"), "postname");
        assert_eq!(get_post_name("a/b/c/d/e/r/posthtml"), "posthtml");
    }
}