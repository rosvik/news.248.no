use axum::{Router, response::Html, routing::get};
use handlebars::Handlebars;
use serde::Serialize;
use std::collections::HashMap;
use std::str::FromStr;

#[tokio::main]
async fn main() {
    let app = Router::new().route("/", get(index().await));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    println!("Listening on http://localhost:3000");
    axum::serve(listener, app).await.unwrap();
}

async fn index() -> Html<String> {
    let mut template = Handlebars::new();
    template
        .register_template_file("index.html.hbs", "templates/index.html.hbs")
        .unwrap();
    let rendered = template
        .render(
            "index.html.hbs",
            &HashMap::from([("articles", rss().await)]),
        )
        .unwrap();
    Html(rendered)
}

#[derive(Debug, Serialize)]
#[allow(dead_code)]
struct Article {
    title: String,
    link: String,
    pub_date: String,
}
async fn rss() -> Vec<Article> {
    let resp = reqwest::get("https://www.nrk.no/norge/toppsaker.rss")
        .await
        .unwrap();
    let body = resp.text().await.unwrap();
    let doc = rss::Channel::from_str(body.as_str()).unwrap();
    doc.items
        .into_iter()
        .map(|item| Article {
            title: item.title.unwrap(),
            link: item.link.unwrap(),
            pub_date: item.pub_date.unwrap(),
        })
        .collect::<Vec<Article>>()
}
