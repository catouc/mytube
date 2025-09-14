use clap::{Parser, Subcommand};
use jiff::{Timestamp, ToSpan};
use rusqlite::Connection;
use youtube_dl::YoutubeDl;
use crate::channel::{fetch_outdated, update_videos, add};

mod channel;
mod video;

#[derive(Parser)]
#[command(version, about, long_about = None)] // Read from `Cargo.toml`
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Download {
        destination: String,
    },
    AddChannel {
        name: String,
        channel_id: String,
    },
    MigrateSQL,
    UpdateChannels {
        since: i32
    },
}

fn main() {
    let cli = Cli::parse();

    let mut db = Connection::open("mytube.db").expect("cannot find DB file");

    match &cli.command {
        Commands::Download{destination} => {
            let mut statement = db
                .prepare("SELECT url FROM video WHERE downloaded == 0;")
                .expect("SQL statement to fetch undownloaded videos is not valid");

            let mut mark_video_downloaded = db
                .prepare("UPDATE video SET downloaded = 1 WHERE url = ?1;")
                .expect("SQL statement to set video as downloaded is not valid");

            statement.query_map([], |row| {
                let url: String = row.get(0).unwrap();
                Ok(url)
            })
                .unwrap()
                .filter_map(|v| v.ok())
                .for_each(|v| {
                    println!("Downloading {v}");
                    YoutubeDl::new(&v)
                        .socket_timeout("15")
                        .download_to(destination)
                        .unwrap();

                    mark_video_downloaded
                        .execute((&v,))
                        .unwrap();
                });
        },
        Commands::AddChannel{name, channel_id: feed_url} => {
            add(&db, name, feed_url).expect("failed to add channel");
        },
        Commands::MigrateSQL => {
            db.execute(
                "CREATE TABLE IF NOT EXISTS channel (
                    id INTEGER PRIMARY KEY,
                    yt_id TEXT NOT NULL,
                    feed_url TEXT NOT NULL,
                    name TEXT NOT NULL,
                    last_fetched TEXT NOT NULL
                )",
                (),
            ).unwrap();

            db.execute(
                "CREATE TABLE IF NOT EXISTS video (
                    id INTEGER PRIMARY KEY,
                    url TEXT NOT NULL,
                    title TEXT NOT NULL,
                    downloaded INTEGER NOT NULL,
                    channel_id INTEGER NOT NULL,
                    FOREIGN KEY(channel_id) REFERENCES channel(channel_id)
                )",
                (),
            ).unwrap();
        },
        Commands::UpdateChannels{since} => {
            println!("fetching channels");
            fetch_outdated(&db, Timestamp::now() - since.hours())
                .unwrap()
                .iter()
                .for_each(|channel| {
                    println!("updating channel");
                    let tx = db.transaction().expect("couldn't open DB transaction");
                    update_videos(&tx, channel);
                    tx.commit().expect("failed to update videos for {channel}");
                })
        },
    }
}
