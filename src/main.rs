// https://www.nrk.no/norge/toppsaker.rss
use std::str::FromStr;

use axum::{Router, routing::get};

#[tokio::main]
async fn main() {
    let app = Router::new().route(
        "/",
        get(|| async {
            let template = Template::new("index.html.hbs");
            template.render(HashMap::new())
        }),
    );

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();

    let resp = reqwest::get("https://www.nrk.no/norge/toppsaker.rss")
        .await
        .unwrap();
    let body = resp.text().await.unwrap();

    let doc = rss::Channel::from_str(body.as_str()).unwrap();
    for item in doc.items {
        println!("{:?}", item.title);
        println!("{:?}", item.link);
        println!("{:?}", item.pub_date);
        println!("{:?}", item.guid);
    }

    axum::serve(listener, app).await.unwrap();
}
