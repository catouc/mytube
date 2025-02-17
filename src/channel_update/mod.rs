use jiff::Timestamp;
use feed_rs::parser;
use rusqlite::{Connection, Error};

pub struct Channel {
    id: u64,
    feed_url: String,
    last_fetched: Timestamp,
}

pub fn fetch_outdated_channels(db: &Connection, fetch_from: Timestamp) -> Result<Vec<Channel>, Error> {
    let mut statement = db
        .prepare("SELECT id, url, last_fetched FROM channels WHERE last_fetched < ?;")
        .expect("SQL statement to fetch outdated channels is not valid");

    statement.query_map([fetch_from.to_string()], |row| {
        let last_fetched_string: String = row.get(2).unwrap();
        let last_fetched: Timestamp = last_fetched_string.parse().unwrap();
        Ok(Channel{
            id: row.get(0).unwrap(),
            feed_url: row.get(1).unwrap(),
            last_fetched,
        })
    }).unwrap().collect()
}

fn add_new_channel(db: &Connection, url: String) -> Result<usize, Error> {
   db.execute("
        INSERT INTO OR REPLACE INTO channels (id, channel_url, last_fetched)
        VALUES ((SELECT id FROM channels WHERE channel_url = ?1), ?1, ?2)",
        (&url, Timestamp::now().to_string())
    )
}

// TODO: Needs better types
pub fn get_videos_from_channel(db: &Connection, channel: &Channel) -> Vec<String> {
    let response = minreq::get(&channel.feed_url)
        .send()
        .unwrap();
    let feed = parser::parse(response.as_bytes()).unwrap();

    db.execute("
        INSERT INTO OR REPLACE INTO channels (id, channel_url, last_fetched)
        VALUES ((SELECT id FROM channels WHERE channel_url = ?1), ?1, ?2)",
        (&channel.feed_url, Timestamp::now().to_string())
    ).unwrap();

    feed.entries.iter()
        .flat_map(|e| extract_links_from_entry(e))
        .collect()
}

fn extract_links_from_entry(entry: &feed_rs::model::Entry) -> Vec<String> {
    entry.links.iter()
        .map(|l| l.clone().href)
        .collect()
}

// TODO: This should work on a batch and not create one SQL query per video
fn update_video_from_link(link: String, channel: ) {
    db.execute("
        INSERT INTO OR REPLACE INTO videos (id, url, channel_id)
        VALUES ((SELECT id FROM channels WHERE channel_url = ?1), ?1, ?2)",
        (&channel.feed_url, Timestamp::now().to_string())
    ).unwrap();
}
