use std::fs;

mod detector_queuer;
mod manga;

use detector_queuer::Model;

fn main() {
    let data = fs::read_to_string("./data.json").expect("Unable to load the data file");

    let mut mangas: Vec<manga::Manga> = serde_json::from_str(&data).expect("Unable to decode JSON");

    // Print the mangas
    if false {
        for manga in &mangas {
            println!("{}", manga)
        }
    }

    // Train the queueing model
    let start = 27349320;
    let days_to_scan = 365;
    // let days_to_scan = 100;
    let scan_length = days_to_scan * 24 * 60;
    let end = start + scan_length;

    mangas.iter_mut().for_each(|manga| {
        manga.comments.sort_by(|a, b| a.date.cmp(&b.date));
        manga.chapters.sort_by(|a, b| a.date.cmp(&b.date));

        manga
            .comments
            .retain(|comment| comment.date >= start && comment.date < end);
    });

    mangas.sort_unstable_by(|a, b| a.subs.cmp(&b.subs));

    let mut mangas = mangas
        .into_iter()
        .filter(|manga| manga.comments.len() > 0 && manga.subs > 0 && manga.chapters.len() > 0)
        .collect::<Vec<_>>();

    let mut ranges = [0; 5];
    let zone = mangas.len() / 5;

    for i in 0..5 {
        ranges[4 - i] = mangas[zone * (i + 1) - 1].subs;
    }
    ranges[4] = 0;

    mangas.reverse();

    let total = mangas
        .iter()
        .map(|manga| manga.comments.len())
        .sum::<usize>();

    println!("Total comments: {}", total);
    println!("Scanning for {} days", days_to_scan);

    let mut queue = detector_queuer::QueuedManga::from_mangas(&mangas, start);
    let start_time = std::time::Instant::now();
    let mut result = detector_queuer::ScanResult {
        found: 0,
        scanned: 0,
        time_to_scan: 0,
    };
    let model = Model::new();
    for at_time in start..end {
        result = result + model.simulate_minute(&mut queue, at_time, ranges, 6);
    }
    let score = result.found as f64 / total as f64;

    println!("Score: {}", score);
    println!("Time: {:?}", start_time.elapsed());
    println!("Found: {}", result.found);
    println!("Scanned: {}", result.scanned);
    println!("Time to scan: {}", result.time_to_scan);
    println!(
        "Average scans per comment: {}",
        result.scanned / result.found as i32
    );
    println!(
        "Average time to scan per comment: {}",
        result.time_to_scan / result.found as i64
    );
}
