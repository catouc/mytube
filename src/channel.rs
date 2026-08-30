use crate::video::Video;
use feed_rs::parser;
use jiff::Timestamp;

#[derive(Debug)]
pub struct Channel {
    pub id: u32,
    pub yt_id: String,
    pub name: String,
    pub feed_url: String,
    pub last_fetched: Timestamp,
    pub undownloaded_videos: Vec<Video>,
    pub downloaded_video_count: u32,
}

impl Channel {
    pub fn update_videos(&mut self) -> Result<(), minreq::Error> {
        let response = minreq::get(&self.feed_url).send()?;
        let feed = parser::parse(response.as_bytes()).unwrap();

        self.undownloaded_videos = feed
            .entries
            .iter()
            .filter_map(Video::from_feed_entry)
            .collect();

        Ok(())
    }
}

#[derive(Debug)]
pub enum ChannelFetchError {
    MissingID,
}

pub fn from_url(url: &str, name: &str) -> Result<Channel, ChannelFetchError> {
    let channel_html = minreq::get(url).send().unwrap();
    let channel_id_match =
        regex::Regex::new(r#"www\.youtube\.com\/channel\/(?<channel_id>[-a-zA-Z0-9]+).+""#)
            .expect("regex for channel ID does not compile");

    let Some(matches) = channel_id_match.captures(channel_html.as_str().unwrap()) else {
        return Err(ChannelFetchError::MissingID);
    };

    let channel_id = &matches["channel_id"];

    let mut feed_url: String = "https://www.youtube.com/feeds/videos.xml?channel_id=".into();
    feed_url.push_str(channel_id);

    Ok(Channel {
        id: 0,
        yt_id: channel_id.to_string(),
        name: name.to_string(),
        feed_url,
        last_fetched: Timestamp::now(),
        undownloaded_videos: Vec::new(),
        downloaded_video_count: 0,
    })
}
