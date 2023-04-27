use std::ops::Add;

use crate::manga;

const SCAN_PER_MINUTE: i32 = 30;

#[derive(Debug, PartialEq, Eq)]
pub struct ScanResult {
    pub found: i32,
    pub scanned: i32,
    pub time_to_scan: i64,
}

impl ScanResult {
    pub fn new(found: i32, time_to_scan: i64) -> Self {
        Self {
            found,
            time_to_scan,
            scanned: 1,
        }
    }
}

impl Add for ScanResult {
    type Output = Self;

    fn add(self, other: Self) -> Self::Output {
        Self {
            found: self.found + other.found,
            scanned: self.scanned + other.scanned,
            time_to_scan: self.time_to_scan + other.time_to_scan,
        }
    }
}

#[derive(Clone)]
pub struct QueuedManga {
    pub chapters: Vec<i64>,
    pub subs: usize,
    pub comments: Vec<i64>,
    pub last_scan: i64,

    // Cache the last time and last chapter
    last_chapter: usize,
    last_chapter_time: i64,
}

impl QueuedManga {
    pub fn new(manga: &manga::Manga, at_time: i64) -> Self {
        Self {
            chapters: manga.chapters.iter().map(|c| c.date).collect::<Vec<_>>(),
            subs: manga.subs,
            comments: manga.comments.iter().map(|c| c.date).collect::<Vec<_>>(),
            last_scan: at_time as i64,
            last_chapter: 0,
            last_chapter_time: 0,
        }
    }

    pub fn from_mangas(mangas: &[manga::Manga], at_time: i64) -> Vec<Self> {
        mangas.iter().map(|m| Self::new(m, at_time)).collect()
    }

    pub fn scan(&mut self, at_time: i64) -> ScanResult {
        let mut new_comments = 0;
        let mut time_to_scan = 0;
        for comment in &self.comments {
            if *comment > self.last_scan && *comment <= at_time {
                new_comments += 1;

                if at_time - *comment > 90 {
                    println!(
                        "Comment {} is too old, it was {} minutes old",
                        comment,
                        (at_time - *comment)
                    )
                }

                time_to_scan += at_time - *comment;
            }
        }

        self.last_scan = at_time;

        ScanResult::new(new_comments, time_to_scan)
    }

    pub fn update_last_chapter(&mut self, at_time: i64) {
        let mut last_chapter = self.last_chapter;

        // Take advantage of the fact that the chapters are sorted and we only increment it forward
        while (last_chapter + 1) < self.chapters.len() && self.chapters[last_chapter + 1] < at_time
        {
            last_chapter += 1;
        }

        self.last_chapter_time = at_time;
        self.last_chapter = last_chapter;
    }

    pub fn should_scan(&self, at_time: i64, ranges: [usize; 5]) -> bool {
        // Index of portion of the manga we are in
        let mut index = 0;
        while index < ranges.len() && ranges[index] > self.subs {
            index += 1;
        }

        // Boost: parabola with vertex at (24h, 2) and passing through (0, 1) and (48h, 1)
        // Desmos graph: y=\frac{-x\left(x-48\cdot60\right)}{48\cdot60\cdot60\cdot6}\left\{y>1\right\}
        let mut boost = -(at_time - self.last_scan) * ((at_time - self.last_scan) - (48 * 60))
            / (48 * 60 * 60 * 6);

        if boost < 1 {
            boost = 1;
        }

        // Lookup for how long to wait between scans in minutes
        let wait_time = match index {
            0 => 5,
            1 => 10,
            2 => 20,
            3 => 40,
            4 => 60,
            _ => panic!("Invalid index"),
        } / boost;
        // };

        at_time - self.last_scan > wait_time
    }
}

#[derive(Debug, Clone)]
pub struct Model {}

impl Model {
    pub fn new() -> Self {
        Self {}
    }

    pub fn simulate_minute(
        &self,
        manga: &mut Vec<QueuedManga>,
        at_time: i64,
        ranges: [usize; 5], // number of subs in each range
        clients: i32,       // number of clients
    ) -> ScanResult {
        let mut allowance = clients * SCAN_PER_MINUTE;
        let mut result = ScanResult::new(0, 0);

        for manga in manga.iter_mut() {
            manga.update_last_chapter(at_time);

            if at_time - manga.last_scan > 90 {
                println!(
                    "Warning: manga {} has not been scanned in {} minutes",
                    manga.subs,
                    at_time - manga.last_scan
                );

                allowance -= 1;
                if allowance < 0 {
                    break;
                }

                result = result + manga.scan(at_time);
            }
        }

        if allowance < 0 {
            println!(
                "Warning: clients * scan per minute is negative: {}",
                allowance
            );
        }

        for manga in manga.iter_mut() {
            manga.update_last_chapter(at_time);
            if !manga.should_scan(at_time, ranges) {
                continue;
            }

            allowance -= 1;
            if allowance < 0 {
                break;
            }

            result = result + manga.scan(at_time);
        }
        result
    }
}
