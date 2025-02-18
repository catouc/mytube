use rusqlite::Connection;

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
            downloaded INTEGER NOT NULL,
            channel_id INTEGER NOT NULL,
            FOREIGN KEY(channel_id) REFERENCES channels(channel_id)
        )",
        (),
    ).unwrap();
}
