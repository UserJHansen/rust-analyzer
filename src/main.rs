use std::fs;

mod calculate_peaks;
mod detector_queuer;
mod manga;
mod schedule;

use chrono::{FixedOffset, LocalResult::Single, TimeZone};
use detector_queuer::Model;
use schedule::Schedule;

fn main() {
    let data = fs::read_to_string("./data.json").expect("Unable to load the data file");

    let v: Vec<manga::Manga> = serde_json::from_str(&data).expect("Unable to decode JSON");

    if false {
        for manga in &v {
            println!("{}", manga)
        }
    }

    // predict the schedule of the manga
    if false {
        let mut sched = Schedule::calculate_schedule(&v);
        sched.sort_by(|(_, a), (_, b)| a.cmp(b));
        sched
            .iter()
            .for_each(|(manga, schedule)| println!("{}: {:?}", manga.name, schedule));
    }

    // Train the queueing model
    if true {
        let start = match TimeZone::with_ymd_and_hms(
            &FixedOffset::east_opt(8 * 3600).unwrap(),
            2023,
            1,
            1,
            0,
            0,
            0,
        ) {
            Single(t) => t,
            _ => panic!("Invalid time"),
        };

        let model = Model::from_weights(
            0.32082537252466403,
            0.9566291030937526,
            0.25397193250459693,
            0.0013125164034918368,
        );
        // let model = Model::new();
        let mut results: Vec<(f64, Model)> = Vec::new();

        let threads = 12;
        let mut handles = Vec::new();

        (0..threads).for_each(|_| {
            let mut queue = detector_queuer::QueuedManga::from_mangas(&v, start);

            let mut model = model.clone();
            let start = start.clone();
            let handle = std::thread::spawn(move || {
                let mut results: Vec<(f64, Model)> = Vec::new();
                let mut best = 0.0;
                for _ in 0..1000 {
                    let mut result = detector_queuer::ScanResult {
                        found: 0,
                        scanned: 0,
                    };
                    for i in 0..20000 {
                        let at_time = start + chrono::Duration::minutes(i);
                        result = result + model.simulate_minute(&mut queue, at_time, 20);
                    }
                    let score = result.found as f64 / result.scanned as f64;
                    results.push((score, model.clone()));
                    if score > best {
                        best = score;
                        println!("New best: {:?} {}", model, score);
                    }
                    println!(
                        "{:?}: {} / {} = {}",
                        model, result.found, result.scanned, score
                    );
                    model.shuffle();
                }
                results
            });
            handles.push(handle);
        });

        for handle in handles {
            results.extend(handle.join().unwrap());
        }

        results.sort_by(|(a, _), (b, _)| a.partial_cmp(b).unwrap());
        println!("Best model: {:?}", results.last().unwrap());
    }
}
