use rusqlite::Connection;
use jiff::{Timestamp, ToSpan};
use mytube::channel_update::{fetch_outdated_channels, update_videos_for_channel};

fn main() {
    let db = Connection::open("mytube.db").unwrap();
    let fetch_from = Timestamp::now() - 1.hours();

    fetch_outdated_channels(&db, fetch_from).unwrap().iter()
        .for_each(|channel| {
            let mut conn = Connection::open("mytube.db").unwrap();
            let tx = conn.transaction().unwrap();
            println!("updating videos for channel");
            update_videos_for_channel(&tx, channel);
            tx.commit().unwrap();
        });
}

