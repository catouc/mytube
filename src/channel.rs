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
