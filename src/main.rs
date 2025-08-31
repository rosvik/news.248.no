use axum::{Router, extract::Query, response::Html, routing::get};
use chrono::{DateTime, Datelike, Duration, Local, NaiveDate, TimeZone};
use chrono_tz::Tz;
use handlebars::Handlebars;
use serde::{Deserialize, Serialize};
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

#[derive(Debug, Serialize, Clone)]
#[allow(dead_code)]
pub struct Article {
    pub title: String,
    pub link: String,
    pub published_time: chrono::DateTime<chrono::Utc>,
    pub formatted_published_time: String,
    pub image: Option<String>,
}

#[tokio::main]
async fn main() {
    let app = Router::new().route("/", get(index));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:2341").await.unwrap();
    println!("Listening on http://localhost:2341");
    axum::serve(listener, app).await.unwrap();
}

#[derive(Debug, Deserialize)]
struct Parameters {
    pub date: Option<String>,
}
async fn index(Query(query): Query<Parameters>) -> Html<String> {
    let now = Local::now().with_timezone(&chrono_tz::Europe::Oslo);
    let midnight: DateTime<Tz> =
        match NaiveDate::parse_from_str(query.date.unwrap_or_default().as_str(), "%Y-%m-%d") {
            Ok(date) => chrono_tz::Europe::Oslo
                .with_ymd_and_hms(date.year(), date.month(), date.day(), 0, 0, 0)
                .unwrap(),
            Err(e) => {
                println!("Invalid date, using current date: {e}");
                chrono_tz::Europe::Oslo
                    .with_ymd_and_hms(now.year(), now.month(), now.day(), 0, 0, 0)
                    .unwrap()
            }
        };

    let bbc = rss::rss("https://feeds.bbci.co.uk/news/world/rss.xml").await;
    let nrk = nrk::nrk("https://www.nrk.no/nyheter").await;

    // Filter out articles published before midnight
    let filtered_nrk = nrk
        .iter()
        .filter(|a| {
            a.published_time >= midnight && a.published_time <= midnight + Duration::days(1)
        })
        .cloned()
        .collect::<Vec<_>>();
    let filtered_bbc = bbc
        .iter()
        .filter(|a| {
            a.published_time >= midnight && a.published_time <= midnight + Duration::days(1)
        })
        .cloned()
        .collect::<Vec<_>>();

    let publications = vec![
        Publication {
            name: "NRK".to_string(),
            url: "https://www.nrk.no/nyheter".to_string(),
            articles: filtered_nrk,
        },
        Publication {
            name: "BBC".to_string(),
            url: "https://www.bbc.com/news/world".to_string(),
            articles: filtered_bbc,
        },
    ];

    let mut template = Handlebars::new();
    template
        .register_template_file("index.html.hbs", "templates/index.html.hbs")
        .unwrap();

    let rendered = template
        .render(
            "index.html.hbs",
            &HashMap::from([("publications", publications)]),
        )
        .unwrap();
    Html(rendered)
}
