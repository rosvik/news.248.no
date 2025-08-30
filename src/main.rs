use axum::{Router, response::Html, routing::get};
use handlebars::Handlebars;
use serde::Serialize;
use std::collections::HashMap;
use std::str::FromStr;

// https://www.nrk.no/norge/toppsaker.rss
// https://www.nrk.no/nyheter/siste.rss
// https://feeds.bbci.co.uk/news/world/rss.xml

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
    image: Option<String>,
}
async fn rss() -> Vec<Article> {
    let resp = reqwest::get("https://feeds.bbci.co.uk/news/world/rss.xml")
        .await
        .unwrap();
    let body = resp.text().await.unwrap();
    let doc = rss::Channel::from_str(body.as_str()).unwrap();
    doc.items
        .into_iter()
        .map(|item| {
            let image = get_image_url(&item);
            Article {
                title: item.title.unwrap(),
                link: item.link.unwrap(),
                pub_date: item.pub_date.unwrap(),
                image,
            }
        })
        .collect::<Vec<Article>>()
}

fn get_image_url(item: &rss::Item) -> Option<String> {
    // 1) media:content (Media RSS)
    if let Some(media_ns) = item.extensions().get("media") {
        if let Some(contents) = media_ns.get("content") {
            for ext in contents {
                // Prefer media:content with medium="image"
                let is_image = ext
                    .attrs
                    .get("medium")
                    .map(|m| m == "image")
                    .unwrap_or(false);

                if is_image && let Some(url) = ext.attrs.get("url") {
                    return Some(url.clone());
                }

                // Also accept media:thumbnail if present
                if let Some(thumbnails) = ext.children.get("thumbnail") {
                    for t in thumbnails {
                        if let Some(url) = t.attrs.get("url") {
                            return Some(url.clone());
                        }
                    }
                }
            }
        }

        // Or a top-level media:thumbnail
        if let Some(thumbnails) = media_ns.get("thumbnail") {
            for t in thumbnails {
                if let Some(url) = t.attrs.get("url") {
                    return Some(url.clone());
                }
            }
        }
    }

    // 2) Fallback to <enclosure url="..."> if it’s an image
    if let Some(enc) = item.enclosure()
        && enc.mime_type().starts_with("image/")
    {
        return Some(enc.url().to_string());
    }

    None
}
