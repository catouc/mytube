pub struct Video {
    pub title: String,
    pub url: String,
}

/// This expects that the feed entry structure of a YouTube RSS feed is
/// at least the following:
/// <entry>
///   <title>
///     some title
///   </title>
///   <link rel="alternate" href="youtube video watch link"/>
/// </entry>
pub fn from_feed_entry(entry: &feed_rs::model::Entry) -> Video {
    let url_list: Vec<String> = entry.links.iter()
        .map(|l| l.clone().href)
        .collect();

    if url_list.len() > 1 {
        eprintln!("entry has too many links, taking the first: \"{}\"", url_list.join(","));
    };

    let url = url_list[0].clone();

    let title = match &entry.title {
       Some(title) => {
            title.content.clone()
       },
       None => "".to_owned(),
    };

    Video{url, title}
}
