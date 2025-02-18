use rusqlite::Connection;
use rustube::blocking::download_best_quality;

fn main() {
    let db = Connection::open("mytube.db").unwrap();
    let mut statement = db
        .prepare("SELECT url FROM videos WHERE downloaded == 0;")
        .expect("SQL statement to fetch undownloaded videos is not valid");

    let video_iter = statement.query_map([], |row| {
        let url: String = row.get(0).unwrap();
        Ok(url)
    }).unwrap();

    video_iter.for_each(|v| {
        let path = download_best_quality(&v.unwrap()).unwrap();
        println!("downloaded {:#?}", path);
    });
}
