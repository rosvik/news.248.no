use crate::{Article, Publication};
use rusqlite::Connection;

pub const NRK_ID: &str = "NRK";
pub const BBC_ID: &str = "BBC";

pub fn init() -> Connection {
    let conn = Connection::open("./news.db").unwrap();
    conn.execute_batch("PRAGMA foreign_keys = ON;").unwrap();
    conn.execute_batch(include_str!("init.sql")).unwrap();
    conn
}

pub fn add_article(article: Article, publication_id: &str) {
    let conn = Connection::open("./news.db").unwrap();

    // Skip if article already exists
    let existing = conn
        .query_row(
            "SELECT COUNT(*) FROM articles WHERE id = ?",
            [&article.id],
            |row| row.get::<_, i64>(0),
        )
        .unwrap();
    if existing > 0 {
        return;
    }

    conn.execute(
        include_str!("add_article.sql"),
        (
            article.id,
            publication_id,
            article.title,
            article.link,
            article.published_time.to_string(),
            article.formatted_published_time,
            article.image,
        ),
    )
    .unwrap();
}

pub fn add_publication(id: &str, publication: Publication) {
    let conn = Connection::open("./news.db").unwrap();
    // Skip if publication already exists
    let existing = conn
        .query_row(
            "SELECT COUNT(*) FROM publications WHERE id = ?",
            [&id],
            |row| row.get::<_, i64>(0),
        )
        .unwrap();
    if existing > 0 {
        return;
    }
    conn.execute(
        include_str!("add_publication.sql"),
        (id, publication.name, publication.url),
    )
    .unwrap();
}
