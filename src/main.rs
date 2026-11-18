#![warn(clippy::pedantic)]

use clap::{Parser, Subcommand};
use jiff::{Timestamp, ToSpan};
use youtube_dl::YoutubeDl;

mod channel;
mod storage;
mod video;
mod filter;
mod series;

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
    Download {
        channel: String,
        destination: String,
        #[arg(short, long)]
        episode: bool,
    },
    AddChannel {
        name: String,
        url: String,
    },
    AddSeries {
        name: String,
        channel_name: String,
        folder: String,
        pattern: String,
        #[arg(short, long)]
        filter_name: Option<String>,
    },
    UpdateChannels {
        since: i32,
    },
    ListVideos {
        #[arg(short, long)]
        channel: Option<String>,
        #[arg(short, long)]
        series: Option<String>,
    },
    MarkVideoDownloaded {
        url: String,
    },
}

#[derive(Subcommand)]
enum Download {
    Video {
        channel: String,
        destination: String,
    },
    Single {
        channel: String,
        destination: String,
    },
}

fn main() {
    let cli = Cli::parse();
    let storage = storage::Storage::new(&cli.db_file).expect("failed to set up storage");

    match &cli.command {
        Commands::Download {
            channel,
            destination,
            episode,
        } => {
            let chan = &storage.channel(channel).expect("did not find channel");
            let mut downloaded_videos = 1;

            for v in &chan.undownloaded_videos {
                let mut yt = YoutubeDl::new(&v.url);
                yt.extra_arg("--embed-chapters")
                    .extra_arg("--embed-thumbnail")
                    .extra_arg("--embed-info-json")
                    .extra_arg("--embed-metadata")
                    .extra_arg("--write-thumbnail")
                    .extra_arg("--write-info-json");
                // .youtube_dl_path("/home/pb/.local/bin/yt-dlp");

                if *episode {
                    let episode_number = chan.downloaded_video_count + downloaded_videos;
                    let episode = format!("S01E{episode_number:0>3} %(title)s.%(ext)s");
                    yt.output_template(episode)
                        .extra_arg("--exec=\"nfo-writer --file-type=single %(infojson_filename)q\"")
                        .download_to(destination)
                        .expect("failed to download");
                } else {
                    yt.output_template("%(title)s.%(ext)s")
                        .download_to(destination)
                        .expect("failed to download");
                }

                println!("Downloading: {:#?}", &v.url);

                storage
                    .mark_video_downloaded(&v.url)
                    .expect("failed to mark video downloaded");

                downloaded_videos += 1;
            }
        }

        Commands::AddChannel { name, url } => {
            let chan = channel::from_url(url, name).unwrap();
            storage.insert_channels(&[chan]);
        }

        Commands::AddSeries { name, channel_name, folder, pattern, filter_name } => {
            let channel = &storage
                .channel(channel_name)
                .expect("did not find channel");

            let filter = if let Some(filter_name) = filter_name {
                    &storage.filter_from_name(filter_name).expect("failed to find filter")
                } else {
                    let f = filter::Filter{
                       id: 0,
                       name: name.clone(),
                       field: "title".into(),
                       pattern: pattern.clone(),
                    };
                    
                    storage.insert_filters(&[f])
                        .expect("failed to create filter");

                    &storage.filter_from_name(name).expect("failed to find filter")
            };

            let s = series::Series{
               id: 0,
               name: name.clone(),
               channel_id: channel.id,
               filter_id: filter.id,
               destination: folder.clone(),
            };

            storage.insert_series(&[s]).expect("failed to insert series");
        }

        Commands::UpdateChannels { since } => {
            let mut channels = storage.channels(Timestamp::now() - since.hours()).unwrap();

            for c in &mut channels {
                c.update_videos(&storage).unwrap();
            }

            storage.insert_channels(&channels);
        }

        Commands::ListVideos { channel, series } => {
            if let Some(channel) = channel {
                for v in &storage
                    .channel(channel)
                    .expect("did not find channel")
                    .undownloaded_videos
                {
                    let series = &storage.series_from_id(&v.series_id).unwrap();
                    if let Some(series) = series {
                        println!("{}\t{}\t{}\t{}\t{}",
                            &channel, &v.title, &v.url, &v.thumbnail_url, series.destination,
                        );
                    } else {
                        println!("{}\t{}\t{}\t{}",
                            &channel, &v.title, &v.url, &v.thumbnail_url,
                        );
                    }
                }
            }

            else {
                for channel in &storage
                    .channels(Timestamp::now())
                    .expect("failed to find channels")
                {
                    for v in &channel.undownloaded_videos {
                        let series = &storage.series_from_id(&v.series_id).unwrap();
                        if let Some(series) = series {
                            println!("{}\t{}\t{}\t{}\t{}",
                                &channel.name, &v.title, &v.url, &v.thumbnail_url, series.destination,
                            );
                        } else {
                            println!("{}\t{}\t{}\t{}",
                                &channel.name, &v.title, &v.url, &v.thumbnail_url,
                            );
                        }
                    }
                }
            }
        }

        Commands::MarkVideoDownloaded { url } => storage
            .mark_video_downloaded(url)
            .expect("failed to mark video downloaded"),
    }
}
