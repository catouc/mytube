use crate::channel::Channel;
use crate::video::Video;
use crate::series::Series;
use crate::filter::Filter;
use jiff::Timestamp;
use rusqlite::{Connection, Error, Row};
use regex::Regex;

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
                series_id INTEGER NOT NULL,
                FOREIGN KEY(channel_id) REFERENCES channel(channel_id)
                FOREIGN KEY(series_id) REFERENCES series(series_id)
            )",
            (),
        )?;

        db.execute(
            "CREATE TABLE IF NOT EXISTS filter (
                id INTEGER PRIMARY KEY,
                name TEXT NOT NULL,
                field TEXT NOT NULL,
                pattern TEXT NOT NULL
            )",
            (),
        )?;

        db.execute(
            "CREATE TABLE IF NOT EXISTS series (
                id INTEGER PRIMARY KEY,
                name TEXT NOT NULL,
                destination TEXT NOT NULL,
                channel_id INTEGER NOT NULL,
                filter_id INTEGER NOT NULL
            )",
            (),
        )?;

        Ok(Storage { db })
    }

    pub fn channel(&self, channel_name: &str) -> Result<Channel, Error> {
        self.db.query_one(
            "SELECT
                id,
                yt_id,
                feed_url,
                name,
                last_fetched,
                (SELECT count(id) FROM video WHERE channel_id = channel.id AND downloaded = 1) AS video_count
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
                SELECT
                    id, yt_id, name, feed_url, last_fetched, 
                    (SELECT count(id) FROM video WHERE channel_id = channel.id AND downloaded = 1) AS video_count
                FROM channel WHERE datetime(last_fetched) < datetime(?);",
            )
            .expect("SQL statement to fetch outdated channels is not valid");

        fetch_channels
            .query_map([fetch_from.to_string()], |r| self.channel_from_row(r))?
            .collect()

    }

    pub fn insert_channels(&self, channels: &[Channel]) {
        for c in channels {
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
                                channel_id,
                                series_id
                            )
                            VALUES (
                                (SELECT id FROM video WHERE url = ?1),
                                ?1, ?2, ?3, ?4, ?5, ?6
                            )
                            ON CONFLICT DO NOTHING",
                        (&v.url, &v.title, &v.thumbnail_url, 0, c.id, &v.series_id),
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
        let mut videos_for_channel = self
            .db
            .prepare(
                "
                SELECT title, url, thumbnail_url, series_id
                FROM video
                WHERE downloaded != 1 AND channel_id = ?1",
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
                    series_id: r.get(3)?,
                })
            })?
            .collect::<rusqlite::Result<Vec<Video>>>()
    }

    fn channel_from_row(&self, row: &Row) -> Result<Channel, Error> {
        let video_id: u32 = row.get(0)?;
        let last_fetched_string: String = row.get(4)?;
        let last_fetched: Timestamp = last_fetched_string.parse().unwrap();
        let downloaded_video_count = row.get(5)?;

        Ok(Channel {
            id: video_id,
            yt_id: row.get(1)?,
            name: row.get(2)?,
            feed_url: row.get(3)?,
            last_fetched,
            undownloaded_videos: self.undownloaded_videos_for_channel(video_id)?,
            downloaded_video_count,
        })
    }

    pub fn insert_series(&self, series: &[Series]) -> Result<(), Error> {
        for s in series {
            self.db
                .execute(
                    "
                    REPLACE INTO series (id, name, destination, channel_id, filter_id)
                    VALUES ((SELECT id FROM series WHERE name = ?1), ?1, ?2, ?3, ?4)",
                    (&s.name, &s.destination, &s.channel_id, &s.filter_id)
                )?;
        }
        Ok(())
    }

    pub fn series_from_id(&self, series_id: &u32) -> Result<Series, Error> {
        self.db.query_one(
            "SELECT
                id,
                name,
                destination,
                channel_id,
                filter_id
             FROM series
             WHERE id = ?1
            ",
            [series_id],
            |r| self.series_from_row(r),
        )
    }

    pub fn series_for_channel(&self, channel_id: &u32) -> Result<Vec<Series>, Error> {
        let mut series_for_channel = self.db.prepare(
            "SELECT
                id,
                name,
                destination,
                channel_id,
                filter_id
             FROM series
             WHERE channel_id = ?1
            "
        ).expect("sql query for fetching series is invalid");

        series_for_channel.query_map(
            [channel_id],
            |r| {
                Ok(Series{
                    id: r.get(0)?,
                    name: r.get(1)?,
                    destination: r.get(2)?,
                    channel_id: r.get(3)?,
                    filter_id: r.get(4)?,
                })
            })?
        .collect::<rusqlite::Result<Vec<Series>>>()
    }


    fn series_from_row(&self, row: &Row) -> Result<Series, Error> {
        Ok(Series {
            id: row.get(0)?,
            name: row.get(1)?,
            destination: row.get(2)?,
            channel_id: row.get(3)?,
            filter_id: row.get(4)?,
        })
    }

    fn series_from_channel(&self, channel_name: &str) -> Result<Vec<Series>, Error> {
        let channel = self.channel(channel_name)?;

        let mut fetch_series = self
            .db
            .prepare(
                "
                SELECT
                    id, name, channel_id, filter_id, destination
                FROM series WHERE channel_id = ?",
            )
            .expect("SQL statement to fetch series is not valid");

        fetch_series
            .query_map([channel.id], |r| self.series_from_row(r))?
            .collect()
    }

    pub fn filter_from_id(&self, filter_id: u32) -> Result<Filter, Error> {
        self.db.query_one(
            "SELECT
                id,
                name,
                field,
                pattern
             FROM filter
             WHERE id = ?1
            ",
            [filter_id],
            |r| self.filter_from_row(r),
        )
    }

    pub fn filter_from_name(&self, filter_name: &str) -> Result<Filter, Error> {
        self.db.query_one(
            "SELECT
                id,
                name,
                field,
                pattern
             FROM filter
             WHERE name = ?1
            ",
            [filter_name],
            |r| self.filter_from_row(r),
        )
    }

    fn filter_from_row(&self, row: &Row) -> Result<Filter, Error> {
        Ok(Filter {
            id: row.get(0)?,
            name: row.get(1)?,
            field: row.get(2)?,
            pattern: row.get(3)?,
        })
    }

    pub fn insert_filters(&self, filters: &[Filter]) -> Result<(), Error> {
        for f in filters {
            self.db
                .execute(
                    "
                    REPLACE INTO filter (id, name, field, pattern)
                    VALUES ((SELECT id FROM filter WHERE name = ?1), ?1, ?2, ?3)",
                    (&f.name, &f.field, &f.pattern)
                )?;
        }
        Ok(())
    }
}
