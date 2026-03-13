use clap::{Parser, Subcommand};
use jiff::{Timestamp, ToSpan};
use rusqlite::Connection;
use youtube_dl::YoutubeDl;
use crate::channel::{fetch_outdated, update_videos, add};
use crate::video::Video;

mod channel;
mod video;

#[derive(Parser)]
#[command(version, about, long_about = None)] // Read from `Cargo.toml`
struct Cli {
    #[command(subcommand)]
    command: Commands,

    #[arg(short,long, default_value="/var/mytube/mytube.db")]
    db_file: String,
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
    ListVideos {
        channel: String,
    },
    MarkVideoDownloaded {
        url: String,
    },
}

fn main() {
    let cli = Cli::parse();

    let mut db = Connection::open(&cli.db_file).expect("cannot find DB file");

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
                        .format("vcodec=av01/bestvideo*+bestaudio/best")
                        .download_to(destination)
                        .unwrap();

                    mark_video_downloaded
                        .execute((&v,))
                        .unwrap();
                });
        },
        Commands::AddChannel{name, channel_id } => {
            add(&db, name, channel_id).expect("failed to add channel");
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
                    thumbnail_url TEXT NOT NULL,
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
        Commands::ListVideos{channel} => {
            let mut statement = db.prepare(
                "SELECT url, title, thumbnail_url FROM video WHERE downloaded = 0 AND channel_id = (SELECT id FROM channel WHERE name = ?1)",
            ).expect("invalid SQL");

            let video_iter = statement.query_map([channel], |row| {
                Ok(Video{
                    url: row.get(0).unwrap(),
                    title: row.get(1).unwrap(),
                    thumbnail_url: row.get(2).unwrap(),
                }) 
            }).unwrap();

            for video in video_iter {
                let v = video.unwrap();
                println!("{}\t{}\t{}", &v.title, &v.url, &v.thumbnail_url)
            }
        },
        Commands::MarkVideoDownloaded{url} => {
            let mut mark_video_downloaded = db
                .prepare("UPDATE video SET downloaded = 1 WHERE url = ?1;")
                .expect("SQL statement to set video as downloaded is not valid");

             mark_video_downloaded
                .execute((&url,))
                .unwrap();
        },
    }
}
