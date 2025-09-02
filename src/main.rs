use axum::{Router, extract::Query, response::Html, routing::get};
use chrono::{DateTime, Datelike, Duration, Local, NaiveDate, TimeZone};
use chrono_tz::Tz;
use handlebars::Handlebars;
use serde::{Deserialize, Serialize};

// https://www.nrk.no/norge/toppsaker.rss
// https://www.nrk.no/nyheter/siste.rss
// https://feeds.bbci.co.uk/news/world/rss.xml

mod db;
mod nrk;
mod rss;
mod scheduler;

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
    pub id: String,
    pub title: String,
    pub link: String,
    pub published_time: chrono::DateTime<chrono::Utc>,
    pub formatted_published_time: String,
    pub image: Option<String>,
}

#[tokio::main]
async fn main() {
    db::init();
    db::add_publication(
        db::NRK_ID,
        Publication {
            name: "NRK".to_string(),
            url: "https://www.nrk.no/nyheter".to_string(),
            articles: vec![],
        },
    );
    db::add_publication(
        db::BBC_ID,
        Publication {
            name: "BBC".to_string(),
            url: "https://www.bbc.com/news/world".to_string(),
            articles: vec![],
        },
    );
    scheduler::start_scheduler().await.unwrap();

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
    let now_date = chrono_tz::Europe::Oslo
        .with_ymd_and_hms(now.year(), now.month(), now.day(), 0, 0, 0)
        .unwrap();
    let selected_date: DateTime<Tz> =
        match NaiveDate::parse_from_str(query.date.unwrap_or_default().as_str(), "%Y-%m-%d") {
            Ok(date) => chrono_tz::Europe::Oslo
                .with_ymd_and_hms(date.year(), date.month(), date.day(), 0, 0, 0)
                .unwrap(),
            Err(e) => {
                println!("Invalid date, using current date: {e}");
                now_date
            }
        };

    let bbc = db::get_articles(db::BBC_ID);
    let nrk = db::get_articles(db::NRK_ID);

    // Filter out articles published before midnight
    let mut filtered_nrk = nrk
        .iter()
        .filter(|a| {
            a.published_time >= selected_date
                && a.published_time <= selected_date + Duration::days(1)
        })
        .cloned()
        .collect::<Vec<_>>();
    filtered_nrk.sort_by(|a, b| b.published_time.cmp(&a.published_time));

    let mut filtered_bbc = bbc
        .iter()
        .filter(|a| {
            a.published_time >= selected_date
                && a.published_time <= selected_date + Duration::days(1)
        })
        .cloned()
        .collect::<Vec<_>>();
    filtered_bbc.sort_by(|a, b| b.published_time.cmp(&a.published_time));

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

    let next_date = match selected_date < now_date {
        true => Some(
            (selected_date + Duration::days(1))
                .format("%Y-%m-%d")
                .to_string(),
        ),
        false => None,
    };

    #[derive(Serialize)]
    struct PageCtx {
        publications: Vec<Publication>,
        date: String,
        previous_date: String,
        next_date: Option<String>,
    }
    let ctx = PageCtx {
        publications,
        date: selected_date.format("%Y-%m-%d").to_string(),
        previous_date: (selected_date - Duration::days(1))
            .format("%Y-%m-%d")
            .to_string(),
        next_date,
    };

    let rendered = template.render("index.html.hbs", &ctx).unwrap();
    Html(rendered)
}
