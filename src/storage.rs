use crate::channel::Channel;
use crate::video::Video;
use jiff::Timestamp;
use rusqlite::{Connection, Error, Row};

pub struct Storage {
    db: Connection,
}

impl Storage {
    // Sets up a database, attempts to run the migrations after opening db_file.
    pub fn new(db_file: &str) -> Result<Storage, Error> {
        let db = Connection::open(db_file).expect("cannot open DB file");

        db.execute(
            "CREATE TABLE IF NOT EXISTS channel (
                id INTEGER PRIMARY KEY,
                yt_id TEXT NOT NULL,
                feed_url TEXT NOT NULL,
                name TEXT NOT NULL,
                last_fetched TEXT NOT NULL
            )",
            (),
        )?;

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
        )?;

        Ok(Storage { db })
    }

    pub fn channel(&self, channel_name: &str) -> Result<Channel, Error> {
        self.db.query_one(
            "SELECT id, yt_id, feed_url, name, last_fetched
             FROM channel
             WHERE name = ?1
            ",
            [channel_name],
            |r| self.channel_from_row(r),
        )
    }

    // return a Vec of Channel that is filtered to only include channels that have
    // been not been updated since fetch_from
    pub fn channels(&self, fetch_from: Timestamp) -> Result<Vec<Channel>, Error> {
        let mut fetch_channels = self
            .db
            .prepare(
                "
                SELECT id, yt_id, name, feed_url, last_fetched
                FROM channel WHERE datetime(last_fetched) < datetime(?);",
            )
            .expect("SQL statement to fetch outdated channels is not valid");

        fetch_channels
            .query_map([fetch_from.to_string()], |r| self.channel_from_row(r))?
            .collect()
    }

    pub fn insert_channels(&self, channels: &[Channel]) {
        for c in channels {
            println!("Updating videos for {}", &c.name);

            self.db
                .execute(
                    "
                    REPLACE INTO channel (id, yt_id, name, feed_url, last_fetched)
                    VALUES ((SELECT id FROM channel WHERE feed_url = ?3), ?1, ?2, ?3, ?4)",
                    (&c.yt_id, &c.name, &c.feed_url, Timestamp::now().to_string()),
                )
                .expect("failed to insert channel");

            c.undownloaded_videos.iter().for_each(|v| {
                println!("Processing video {}({})", &v.title, &v.url);

                self.db
                    .execute(
                        "INSERT INTO video (
                                id,
                                url,
                                title,
                                thumbnail_url,
                                downloaded,
                                channel_id
                            )
                            VALUES (
                                (SELECT id FROM video WHERE url = ?1),
                                ?1, ?2, ?3, ?4, ?5
                            )
                            ON CONFLICT DO NOTHING",
                        (&v.url, &v.title, &v.thumbnail_url, 0, c.id),
                    )
                    .unwrap();
            });
        }
    }

    pub fn mark_video_downloaded(&self, url: &str) -> Result<(), Error> {
        let mut mark_video_downloaded = self
            .db
            .prepare("UPDATE video SET downloaded = 1 WHERE url = ?1;")
            .expect("SQL statement to set video as downloaded is not valid");

        mark_video_downloaded.execute((&url,))?;

        Ok(())
    }

    fn undownloaded_videos_for_channel(&self, channel_id: u32) -> Result<Vec<Video>, Error> {
        let mut videos_for_channel = self.db
            .prepare("
                SELECT title, url, thumbnail_url
                FROM video
                WHERE downloaded != 1 AND channel_id = ?1"
            )
            .expect("SQL statement to fetch not downloaded videos is not valid");

        // This is insane, we basicall take the
        // Result of mapped Rows and then
        // collect it into a Result of
        // Vec<Video> to THEN unwrap it.

        videos_for_channel
            .query_map([channel_id], |r| {
                Ok(Video {
                    title: r.get(0)?,
                    url: r.get(1)?,
                    thumbnail_url: r.get(2)?,
                })
            })?
            .collect::<rusqlite::Result<Vec<Video>>>()
    }

    fn channel_from_row(&self, row: &Row) -> Result<Channel, Error> {
        let video_id: u32 = row.get(0)?;
        let last_fetched_string: String = row.get(4)?;
        let last_fetched: Timestamp = last_fetched_string.parse().unwrap();

        Ok(Channel {
            id: video_id,
            yt_id: row.get(1)?,
            name: row.get(2)?,
            feed_url: row.get(3)?,
            last_fetched,
            undownloaded_videos: self.undownloaded_videos_for_channel(video_id)?,
        })
    }
}
