use axum::{Router, response::Html, routing::get};
use handlebars::Handlebars;
use serde::Serialize;
use std::collections::HashMap;

// https://www.nrk.no/norge/toppsaker.rss
// https://www.nrk.no/nyheter/siste.rss
// https://feeds.bbci.co.uk/news/world/rss.xml

mod nrk;
mod rss;

#[derive(Debug, Serialize)]
#[allow(dead_code)]
pub struct Publication {
    pub name: String,
    pub url: String,
    pub articles: Vec<Article>,
}

#[derive(Debug, Serialize)]
#[allow(dead_code)]
pub struct Article {
    pub title: String,
    pub link: String,
    pub published_time: String,
    pub image: Option<String>,
}

#[tokio::main]
async fn main() {
    let app = Router::new().route("/", get(index));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:2341").await.unwrap();
    println!("Listening on http://localhost:2341");
    axum::serve(listener, app).await.unwrap();
}

async fn index() -> Html<String> {
    let mut template = Handlebars::new();
    template
        .register_template_file("index.html.hbs", "templates/index.html.hbs")
        .unwrap();

    let bbc = rss::rss("https://feeds.bbci.co.uk/news/world/rss.xml").await;
    let nrk = nrk::nrk().await;

    let publications = vec![
        Publication {
            name: "NRK".to_string(),
            url: "https://www.nrk.no/nyheter".to_string(),
            articles: nrk,
        },
        Publication {
            name: "BBC".to_string(),
            url: "https://www.bbc.com/news/world".to_string(),
            articles: bbc,
        },
    ];

    let rendered = template
        .render(
            "index.html.hbs",
            &HashMap::from([("publications", publications)]),
        )
        .unwrap();
    Html(rendered)
}
