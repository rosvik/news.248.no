use crate::Article;
use scraper::{Html, Selector};
use serde::Deserialize;

const OPENGRAPH_URL: &str = "https://og.248.no/api?url=";

pub async fn nrk(url: &str) -> Vec<Article> {
    let feed_links = get_feed_links(url).await;
    let mut articles = Vec::new();

    for link in feed_links {
        let article = get_opengraph_data(&link).await;
        if let Some(article) = article {
            articles.push(article);
        }
    }
    articles
}

pub async fn get_feed_links(url: &str) -> Vec<String> {
    println!("Fetching feed links from {url}");
    let resp = reqwest::get(url).await.unwrap();

    let html = Html::parse_document(resp.text().await.unwrap().as_str());
    let selector = Selector::parse(".bulletin-time a").unwrap();

    html.select(&selector)
        .map(|e| e.value().attr("href").unwrap().to_string())
        .collect::<Vec<String>>()
}

#[derive(Deserialize)]
pub struct OpengraphTag {
    pub property: String,
    pub content: String,
}
pub type OpengraphData = Vec<OpengraphTag>;

pub async fn get_opengraph_data(url: &str) -> Option<Article> {
    let resp = reqwest::get(format!("{OPENGRAPH_URL}{url}")).await.unwrap();
    let json = resp.json::<OpengraphData>().await.unwrap();
    let title = json
        .iter()
        .find(|i| i.property == "og:title")
        .unwrap_or(&OpengraphTag {
            property: "og:title".to_string(),
            content: "".to_string(),
        })
        .content
        .clone();
    let published_time = json
        .iter()
        .find(|i| i.property == "article:published_time")
        .unwrap_or(&OpengraphTag {
            property: "article:published_time".to_string(),
            content: "".to_string(),
        })
        .content
        .clone()
        .parse::<chrono::DateTime<chrono::Utc>>()
        .unwrap();

    let formatted_published_time = published_time
        .with_timezone(&chrono_tz::Europe::Oslo)
        .format("%Y-%m-%d %H:%M:%S")
        .to_string();

    let image = json
        .iter()
        .find(|i| i.property == "og:image")
        .map(|i| i.content.clone())
        .clone();

    let id = url.split("/").last().unwrap().to_string();
    // Santiy check, ID should start with "1."
    if id[..2] != *"1." {
        println!("Got invalid ID: {id} for url {url}");
        return None;
    }

    Some(Article {
        id,
        title,
        link: url.to_string(),
        published_time,
        formatted_published_time,
        image,
    })
}
