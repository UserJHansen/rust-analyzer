use std::ops::Add;

use crate::manga;

const SCAN_PER_MINUTE: i32 = 10;

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

pub struct QueuedManga {
    pub chapters: Vec<i64>,
    pub subs: f64,
    pub comments: Vec<i64>,
    pub last_scan: i64,

    /// Number of comments per day
    pub current_growth: usize,

    // Cache the last time and last chapter
    pub last_chapter: i64,
    pub last_chapter_time: i64,
}

impl QueuedManga {
    pub fn new(manga: &manga::Manga, at_time: i64, largest_subs: f64) -> Self {
        let total_comments_in_last_day = manga
            .comments
            .iter()
            .filter(|c| at_time - c.date < 60 * 24)
            .collect::<Vec<_>>()
            .len();

        Self {
            chapters: manga.chapters.iter().map(|c| c.date).collect::<Vec<_>>(),
            subs: manga.subs as f64 / largest_subs,
            comments: manga.comments.iter().map(|c| c.date).collect::<Vec<_>>(),
            last_scan: at_time as i64,
            current_growth: total_comments_in_last_day,
            last_chapter: 0,
            last_chapter_time: 0,
        }
    }

    pub fn from_mangas(mangas: &[manga::Manga], at_time: i64) -> Vec<Self> {
        let largest_subs = mangas
            .iter()
            .map(|manga| manga.subs)
            .max()
            .expect("No mangas in list") as f64;

        mangas
            .iter()
            .map(|m| Self::new(m, at_time, largest_subs))
            .collect()
    }

    pub fn scan(&mut self, at_time: i64) -> ScanResult {
        let mut current_growth = 0;
        let mut new_comments = 0;
        let mut time_to_scan = 0;
        for comment in &self.comments {
            if at_time - *comment < 60 * 24 {
                current_growth += 1;
            }
            if *comment > self.last_scan && *comment < at_time {
                new_comments += 1;

                time_to_scan += at_time - *comment;
            }
        }

        self.last_scan = at_time;
        self.current_growth = current_growth;

        ScanResult::new(new_comments, time_to_scan)
    }

    pub fn last_chapter(&self, at_time: i64) -> i64 {
        if self.last_chapter_time == at_time {
            return self.last_chapter;
        }

        let mut last_chapter = 0;
        for chapter in &self.chapters {
            if *chapter < at_time {
                last_chapter = *chapter;
            } else {
                break;
            }
        }
        last_chapter
    }

    pub fn calculate_score(&self, model: &Model, at_time: i64) -> f64 {
        let mut score = 0.0;
        score += model.subcount_weight * self.subs;
        score += model.last_scan_weight * self.last_scan as f64;
        score += model.current_growth_weight * self.current_growth as f64;
        score += model.last_chap_weight * self.last_chapter(at_time) as f64;
        score
    }
}

#[derive(Debug, Clone)]
pub struct Model {
    pub subcount_weight: f64,
    pub last_scan_weight: f64,
    pub current_growth_weight: f64,
    pub last_chap_weight: f64,
}

impl Model {
    pub fn from_weights(
        subcount_weight: f64,
        last_scan_weight: f64,
        current_growth_weight: f64,
        last_chap_weight: f64,
    ) -> Self {
        Self {
            subcount_weight,
            last_scan_weight,
            current_growth_weight,
            last_chap_weight,
        }
    }

    pub fn simulate_minute(
        &self,
        manga: &mut Vec<QueuedManga>,
        at_time: i64,
        clients: i32, // number of clients
    ) -> ScanResult {
        manga.sort_by(|a, b| {
            a.calculate_score(self, at_time)
                .total_cmp(&b.calculate_score(self, at_time))
        });

        let mut i = 0;
        let mut result = ScanResult::new(0, 0);
        for manga in manga.iter_mut() {
            i += 1;
            if i >= clients * SCAN_PER_MINUTE {
                break;
            }

            result = result + manga.scan(at_time);
        }
        result
    }

    pub fn shuffle(&mut self) {
        self.subcount_weight = rand::random();
        self.last_scan_weight = rand::random();
        self.current_growth_weight = rand::random();
        self.last_chap_weight = rand::random();
    }
}
