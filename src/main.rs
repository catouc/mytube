#![warn(clippy::pedantic)]

use crate::channel::Channel;
use clap::{Parser, Subcommand};
use jiff::{Timestamp, ToSpan};
use rusqlite::Connection;
use youtube_dl::YoutubeDl;

mod channel;
mod storage;
mod video;

#[derive(Parser)]
#[command(version, about, long_about = None)] // Read from `Cargo.toml`
struct Cli {
    #[command(subcommand)]
    command: Commands,

    #[arg(short, long, default_value = "/var/mytube/mytube.db")]
    db_file: String,
}

#[derive(Subcommand)]
enum Commands {
    Download { destination: String },
    AddChannel { name: String, url: String },
    UpdateChannels { since: i32 },
    ListVideos { channel: String },
    MarkVideoDownloaded { url: String },
}

fn main() {
    let cli = Cli::parse();

    let db = Connection::open(&cli.db_file).expect("cannot find DB file");
    let storage = storage::Storage::new(&cli.db_file).expect("failed to set up storage");

    match &cli.command {
        Commands::Download { destination } => {
            let mut statement = db
                .prepare("SELECT url FROM video WHERE downloaded == 0;")
                .expect("SQL statement to fetch undownloaded videos is not valid");

            let mut mark_video_downloaded = db
                .prepare("UPDATE video SET downloaded = 1 WHERE url = ?1;")
                .expect("SQL statement to set video as downloaded is not valid");

            statement
                .query_map([], |row| {
                    let url: String = row.get(0).unwrap();
                    Ok(url)
                })
                .unwrap()
                .filter_map(std::result::Result::ok)
                .for_each(|v| {
                    println!("Downloading {v}");
                    YoutubeDl::new(&v)
                        .socket_timeout("15")
                        .format("vcodec=av01/bestvideo*+bestaudio/best")
                        .download_to(destination)
                        .unwrap();

                    mark_video_downloaded.execute((&v,)).unwrap();
                });
        }

        Commands::AddChannel { name, url } => {
            let channel_html = minreq::get(url).send().unwrap();
            let channel_id_match = regex::Regex::new(r#"www\.youtube\.com\/channel\/(?<channel_id>\S+)""#)
                .expect("regex for channel ID does not compile");

            let Some(matches) = channel_id_match.captures(channel_html.as_str().unwrap()) else {
                eprintln!("failed to locate channel id from YouTube!");
                return
            };

            let channel_id = &matches["channel_id"];

            let mut feed_url: String =
                "https://www.youtube.com/feeds/videos.xml?channel_id=".into();
            feed_url.push_str(channel_id);

            storage.insert_channels(&[Channel {
                id: 0,
                yt_id: channel_id.to_string(),
                name: name.clone(),
                feed_url,
                last_fetched: Timestamp::now(),
                undownloaded_videos: Vec::new(),
            }]);
        }

        Commands::UpdateChannels { since } => {
            let mut channels = storage
                .channels(Timestamp::now() - since.hours())
                .unwrap();

            for c in &mut channels {
                c.update_videos().unwrap();
            }

            storage.insert_channels(&channels);
        }

        Commands::ListVideos { channel } => storage
            .channel(channel)
            .expect("failed to find channel")
            .undownloaded_videos
            .iter()
            .for_each(|v| println!("{}\t{}\t{}", &v.title, &v.url, &v.thumbnail_url)),

        Commands::MarkVideoDownloaded { url } => storage
            .mark_video_downloaded(url)
            .expect("failed to mark video downloaded"),
    }
}
