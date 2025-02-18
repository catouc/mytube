use jiff::Timestamp;
use feed_rs::parser;
use rusqlite::{Connection, Error, Transaction};

#[derive(Debug)]
pub struct Channel {
    id: u64,
    feed_url: String,
    last_fetched: Timestamp,
}

pub fn fetch_outdated_channels(db: &Connection, fetch_from: Timestamp) -> Result<Vec<Channel>, Error> {
    let mut statement = db
        .prepare("SELECT id, feed_url, last_fetched FROM channels WHERE datetime(last_fetched) < datetime(?);")
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

pub fn add_new_channel(db: &Connection, url: &str) -> Result<usize, Error> {
   db.execute("
        REPLACE INTO channels (id, feed_url, last_fetched)
        VALUES ((SELECT id FROM channels WHERE feed_url = ?1), ?1, ?2)",
        (&url, Timestamp::now().to_string())
    )
}

// TODO: Needs better types
pub fn update_videos_for_channel(tx: &Transaction, channel: &Channel) {
    println!("updating videos for {}", channel.id);
    let response = minreq::get(&channel.feed_url)
        .send()
        .unwrap();
    let feed = parser::parse(response.as_bytes()).unwrap();

    tx.execute("
        REPLACE INTO channels (id, feed_url, last_fetched)
        VALUES ((SELECT id FROM channels WHERE feed_url = ?1), ?1, ?2)",
        (&channel.feed_url, Timestamp::now().to_string())
    ).unwrap();

    let videos = feed.entries.iter()
        .flat_map(|e| extract_links_from_entry(e))
        .collect();

    println!("{:#?}", videos);
    insert_videos_for_channel(&tx, channel.id, videos);
}

fn extract_links_from_entry(entry: &feed_rs::model::Entry) -> Vec<String> {
    entry.links.iter()
        .map(|l| l.clone().href)
        .collect()
}

fn insert_videos_for_channel(tx: &Transaction, channel_id: u64, videos: Vec<String>) {
    videos.iter().for_each(|video| {
        println!("inserting {}", video);
        tx.execute("
            INSERT OR IGNORE INTO videos (id, url, downloaded, channel_id)
            VALUES ((SELECT id FROM videos WHERE url = ?1), ?1, ?2, ?3)",
            (video, 0, channel_id)
        ).unwrap();
    });
}
