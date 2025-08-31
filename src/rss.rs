use crate::Article;
use std::str::FromStr;

pub async fn rss(url: &str) -> Vec<Article> {
    println!("Fetching RSS from {}", url);
    let resp = reqwest::get(url).await.unwrap();
    let body = resp.text().await.unwrap();
    let doc = rss::Channel::from_str(body.as_str()).unwrap();
    doc.items
        .into_iter()
        .map(|item| {
            let image = get_image_url(&item);

            let title = match item.title {
                Some(title) => title,
                None => return None,
            };
            let link = match item.link {
                Some(link) => link,
                None => return None,
            };
            let published_time: chrono::DateTime<chrono::Utc> = match item.pub_date {
                Some(pub_date) => match chrono::DateTime::parse_from_rfc2822(&pub_date) {
                    Ok(published_time) => published_time.into(),
                    Err(_) => return None,
                },
                None => return None,
            };
            let formatted_published_time = published_time
                .with_timezone(&chrono_tz::Europe::Oslo)
                .format("%Y-%m-%d %H:%M:%S")
                .to_string();

            Some(Article {
                title,
                link,
                published_time,
                formatted_published_time,
                image,
            })
        })
        .collect::<Vec<Option<Article>>>()
        .into_iter()
        .flatten()
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
