use feed_rs::parser;
use jiff::Timestamp;
use rusqlite::{Connection, Error, Transaction};
use crate::video::Video;

#[derive(Debug)]
pub struct Channel {
    id: u64,
    yt_id: String,
    name: String,
    feed_url: String,
    last_fetched: Timestamp,
}

pub fn fetch_outdated(db: &Connection, fetch_from: Timestamp) -> Result<Vec<Channel>, Error> {
    let mut statement = db
        .prepare("SELECT id, yt_id, name, feed_url, last_fetched FROM channel WHERE datetime(last_fetched) < datetime(?);")
        .expect("SQL statement to fetch outdated channels is not valid");

    statement.query_map([fetch_from.to_string()], |row| {
        let last_fetched_string: String = row.get(4).unwrap();
        let last_fetched: Timestamp = last_fetched_string.parse().unwrap();
        Ok(Channel{
            id: row.get(0).unwrap(),
            yt_id: row.get(1).unwrap(),
            name: row.get(2).unwrap(),
            feed_url: row.get(3).unwrap(),
            last_fetched,
        })
    }).unwrap().collect()
}

pub fn add(db: &Connection, channel_name: &str, id: &str) -> Result<usize, Error> {
    let mut feed_url: String = "https://www.youtube.com/feeds/videos.xml?channel_id=".into();
    feed_url.push_str(id);

    db.execute("
        REPLACE INTO channel (id, yt_id, name, feed_url, last_fetched)
        VALUES ((SELECT id FROM channel WHERE feed_url = ?3), ?1, ?2, ?3, ?4)",
        (id, channel_name, feed_url, Timestamp::now().to_string())
    )
}

// TODO: Needs better types
pub fn update_videos(tx: &Transaction, channel: &Channel) {
    println!("updating videos for {}", channel.id);
    let response = minreq::get(&channel.feed_url)
        .send()
        .unwrap();
    let feed = parser::parse(response.as_bytes()).unwrap();

    tx.execute("
        REPLACE INTO channel (id, yt_id, name, feed_url, last_fetched)
        VALUES ((SELECT id FROM channel WHERE feed_url = ?3), ?1, ?2, ?3, ?4)",
        (&channel.name, &channel.yt_id, &channel.feed_url, Timestamp::now().to_string())
    ).unwrap();

    let videos = feed.entries.iter()
        .map(crate::video::from_feed_entry)
        .collect();

    insert_videos(tx, channel.id, videos);
}

fn insert_videos(tx: &Transaction, channel_id: u64, videos: Vec<Video>) {
    videos.iter().for_each(|video| {
        tx.execute("INSERT INTO video (id, url, title, downloaded, channel_id)
            VALUES ((SELECT id FROM video WHERE url = ?1), ?1, ?2, ?3, ?4)
            ON CONFLICT DO NOTHING",
            (&video.url, &video.title, 0, channel_id)
        ).unwrap();
    });
}
