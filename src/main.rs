use rusqlite::Connection;
use jiff::Timestamp;
use channel_update::fetch_outdated_channels;

mod channel_update;

fn main() {
    let db = Connection::open("mytube.db").unwrap();
    db.execute(
        "CREATE TABLE IF NOT EXISTS channels (
            id INTEGER PRIMARY KEY,
            feed_url TEXT NOT NULL,
            last_fetched TEXT NOT NULL
        )",
        (),
    ).unwrap();

    db.execute(
        "CREATE TABLE IF NOT EXISTS videos (
            id INTEGER PRIMARY KEY,
            url TEXT NOT NULL,
            channel_id INTEGER NOT NULL,
            FOREIGN KEY(channel_id) REFERENCES channels(channel_id)
        )",
        (),
    ).unwrap();

    let channels = fetch_outdated_channels(&db, Timestamp::now());

}
