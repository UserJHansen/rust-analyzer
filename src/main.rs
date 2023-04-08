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
    let scan_length = days_to_scan * 24 * 60;
    let end = start + scan_length;

    let model = Model::from_weights(
        0.7745435606397895,
        0.8173917173600213,
        0.8296325741706723,
        0.000846435401496537,
    );
    // let model = Model::new();

    let mut results: Vec<(f64, Model)> = Vec::new();

    let threads = 6;
    let num_scans = 1_000_000;
    let mut handles = Vec::new();

    mangas.iter_mut().for_each(|manga| {
        manga
            .comments
            .retain(|comment| comment.date >= start && comment.date < end);
    });

    let mangas = mangas
        .into_iter()
        .filter(|manga| manga.comments.len() > 0 && manga.subs > 0 && manga.chapters.len() > 0)
        .collect::<Vec<_>>();

    let total = mangas
        .iter()
        .map(|manga| manga.comments.len())
        .sum::<usize>();

    println!("Total comments: {}", total);
    println!("Scanning for {} days", days_to_scan);

    (0..threads).for_each(|thread| {
        let mut queue = detector_queuer::QueuedManga::from_mangas(&mangas, start);

        let mut model = model.clone();
        let handle = std::thread::spawn(move || {
            let mut results: Vec<(f64, Model)> = Vec::new();
            let mut best = 0.0;
            let mut last_time = std::time::Instant::now();
            println!("Starting thread {}", thread);
            for scan in 0..num_scans {
                if scan != 0 {
                    println!(
                        "Time: {}s, {} scans/min",
                        last_time.elapsed().as_secs(),
                        (scan * scan_length) as f64 / (last_time.elapsed().as_secs() as f64 / 60.0)
                    );
                    println!(
                        "Estimated time remaining: {}hrs",
                        (num_scans - scan) as f64
                            * (last_time.elapsed().as_secs() as f64 / scan as f64)
                            / 3600.0
                    );
                    last_time = std::time::Instant::now();
                }

                let mut result = detector_queuer::ScanResult {
                    found: 0,
                    scanned: 0,
                    time_to_scan: 0,
                };
                for at_time in start..end {
                    result = result + model.simulate_minute(&mut queue, at_time, 20);
                }
                let score = result.found as f64 / total as f64;
                results.push((score, model.clone()));
                if score > best {
                    best = score;
                    println!("New best: {:?} {}", model, score);
                }
                if score > 0.01 {
                    println!("{:?}: {} / {} = {}", model, result.found, total, score);
                }

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
