use std::env;
use rusqlite::Connection;
use mytube::channel_update::add_new_channel;

fn main() {
    let db = Connection::open("mytube.db").unwrap();
    env::args().skip(1).for_each(|c| {
        add_new_channel(&db, &c).unwrap();
    });
}
