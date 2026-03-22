#[derive(Debug)]
pub struct Video {
    pub title: String,
    pub url: String,
    pub thumbnail_url: String,
}

impl Video {
    /// This expects that the feed entry structure of a YouTube RSS feed is
    /// at least the following:
    /// <entry>
    ///   <title>
    ///     some title
    ///   </title>
    ///   <link rel="alternate" href="youtube video watch link"/>
    /// </entry>
    pub fn from_feed_entry(entry: &feed_rs::model::Entry) -> Option<Video> {
        let url_list: Vec<String> = entry
            .links
            .iter()
            .filter_map(|l| {
                if l.href.contains("shorts") {
                    None
                } else {
                    Some(l.clone().href)
                }
            })
            .collect();

        if url_list.is_empty() {
            return None;
        }

        if url_list.len() > 1 {
            eprintln!(
                "entry has too many links, taking the first: \"{}\"",
                url_list.join(",")
            );
        }

        let url = url_list[0].clone();

        let title = match &entry.title {
            Some(title) => title.content.clone(),
            None => String::new(),
        };

        let thumbnail_urls: Vec<String> = entry
            .media
            .iter()
            .map(|m| m.thumbnails[0].image.uri.clone())
            .collect();

        Some(Video {
            url,
            title,
            thumbnail_url: thumbnail_urls[0].clone(),
        })
    }
}
