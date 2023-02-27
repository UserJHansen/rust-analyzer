use std::fs;

mod types;
use types::*;
mod calculate_peaks;
mod schedule;

fn main() {
    let data = fs::read_to_string("./data.json").expect("Unable to load the data file");

    let v: Vec<Manga> = serde_json::from_str(&data).expect("Unable to decode JSON");

    // for manga in v {
    //     println!("{}", manga)
    // }

    let mut sched = schedule::calculate_schedule(&v);

    sched.sort_by(|(_, a), (_, b)| a.cmp(b));

    sched
        .iter()
        .for_each(|(manga, schedule)| println!("{}: {:?}", manga.name, schedule));
}
