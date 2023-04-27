use std::{collections::HashSet, fs};

mod detector_queuer;
mod manga;

use detector_queuer::Model;
use rayon::prelude::*;

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
    // let days_to_scan = 365;
    let days_to_scan = 100;
    let scan_length = days_to_scan * 24 * 60;
    let end = start + scan_length;

    // let num_scans = 1_000_000;
    let num_scans = 10_000;
    // let num_scans = 20;

    mangas.iter_mut().for_each(|manga| {
        manga.comments.sort_by(|a, b| a.date.cmp(&b.date));
        manga.chapters.sort_by(|a, b| a.date.cmp(&b.date));

        manga
            .comments
            .retain(|comment| comment.date >= start && comment.date < end);
    });

    let mut mangas = mangas
        .into_iter()
        .filter(|manga| manga.comments.len() > 0 && manga.subs > 0 && manga.chapters.len() > 0)
        .collect::<Vec<_>>();

    let total = mangas
        .iter()
        .map(|manga| manga.comments.len())
        .sum::<usize>();

    mangas.sort_unstable_by(|b, a| a.comments.len().partial_cmp(&b.comments.len()).unwrap());

    let delta_info = mangas.iter().map(|manga| {
        let mut previous_chap_date = 0;

        manga
            .comments
            .iter()
            .map(|comment| {
                let delta = comment.date - previous_chap_date;
                previous_chap_date = comment.date;
                delta / 60 // Bin by hour
            })
            .collect::<Vec<_>>()
    });

    // Make it an array of dictionaries where the key is the delta and the value is the number of times it occurs
    let delta_info = delta_info
        .map(|delta_info| {
            let delta_info =
                delta_info
                    .into_iter()
                    .fold(std::collections::HashMap::new(), |mut map, delta| {
                        *map.entry(delta).or_insert(0) += 1;
                        map
                    });

            // Convert the hashmap to a vector of tuples
            let mut delta_info = delta_info
                .into_iter()
                .map(|(delta, count)| (delta, count))
                .collect::<Vec<_>>();

            // Sort the vector by the delta
            delta_info.sort_by(|a, b| a.0.cmp(&b.0));

            delta_info
        })
        .collect::<Vec<_>>();

    let all_y = delta_info
        .iter()
        .flat_map(|delta_info| delta_info.iter().map(|(y, _)| *y).collect::<Vec<_>>())
        .collect::<HashSet<_>>();

    let mut csv_data = String::new() + ",";
    for y in all_y.iter() {
        csv_data += &y.to_string();
        csv_data += ",";
    }
    for (i, delta) in delta_info.iter().enumerate() {
        csv_data += "\n";
        csv_data += &i.to_string();
        csv_data += ",";
        for y in all_y.iter() {
            if let Some((_, count)) = delta.iter().find(|(delta, _)| delta == y) {
                csv_data += &count.to_string();
                csv_data += ",";
            } else {
                csv_data += "0,";
            }
        }
    }

    fs::write("data.csv", csv_data).expect("Unable to write to file");

    return;

    println!("Total comments: {}", total);
    println!("Scanning for {} days", days_to_scan);

    fn get_point(iteration: usize) -> detector_queuer::Model {
        // Compute the corresponding index for each dimension
        let dim1_idx = iteration % 10;
        let dim2_idx = (iteration / 10) % 10;
        let dim3_idx = (iteration / 100) % 10;
        let dim4_idx = (iteration / 1000) % 10;

        // Get the model
        detector_queuer::Model::from_weights(
            0.1 * dim1_idx as f64,
            0.1 * dim2_idx as f64,
            0.1 * dim3_idx as f64,
            0.1 * dim4_idx as f64,
        )
    }

    let queue = detector_queuer::QueuedManga::from_mangas(&mangas, start);
    let start_time = std::time::Instant::now();
    let mut results: Vec<(f64, Model)> = (0..num_scans)
        .into_par_iter()
        .map(|scan| {
            if scan % 100 == 0 {
                println!(
                    "Number {} - {} scans/min",
                    scan,
                    (scan) as f64 / (start_time.elapsed().as_secs() as f64 / 60.0)
                );
            }

            let mut queue = queue.clone();

            let model = get_point(scan);
            let mut result = detector_queuer::ScanResult {
                found: 0,
                scanned: 0,
                time_to_scan: 0,
            };
            for at_time in start..end {
                result = result + model.simulate_minute(&mut queue, at_time, 20);
            }
            let score = result.found as f64 / total as f64;
            (score, model)
        })
        .collect::<Vec<_>>();

    results.sort_unstable_by(|a, b| b.0.partial_cmp(&a.0).unwrap());

    let best = &results[0];
    println!("Best score#: {}", best.0);
    println!("Best model: {:?}", best.1);

    #[derive(serde::Serialize)]
    struct Result {
        score: f64,
        subcount_weight: f64,
        last_scan_weight: f64,
        current_growth_weight: f64,
        last_chap_weight: f64,
    }

    // Output the results to a csv file
    let mut csv = csv::Writer::from_path("./results.csv").unwrap();
    for (score, model) in results {
        csv.serialize(Result {
            score,
            subcount_weight: model.subcount_weight,
            last_scan_weight: model.last_scan_weight,
            current_growth_weight: model.current_growth_weight,
            last_chap_weight: model.last_chap_weight,
        })
        .unwrap();
    }
}
