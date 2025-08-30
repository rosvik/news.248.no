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
            Article {
                title: item.title.unwrap(),
                link: item.link.unwrap(),
                published_time: item.pub_date.unwrap(),
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
