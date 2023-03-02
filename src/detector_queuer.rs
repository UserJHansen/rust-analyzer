use std::ops::Add;

use crate::manga;

const SCAN_PER_SECOND: i32 = 10;

#[derive(Debug, PartialEq, Eq)]
pub struct ScanResult {
    pub found: i32,
    pub scanned: i32,
}

impl ScanResult {
    pub fn new(found: i32) -> Self {
        Self { found, scanned: 1 }
    }
}

impl Add for ScanResult {
    type Output = Self;

    fn add(self, other: Self) -> Self::Output {
        Self {
            found: self.found + other.found,
            scanned: self.scanned + other.scanned,
        }
    }
}

pub struct QueuedManga {
    pub manga: manga::Manga,
    pub last_scan: i64,

    /// Number of comments per day
    pub current_growth: i64,
}

impl QueuedManga {
    pub fn new(
        manga: manga::Manga,
        at_time: chrono::DateTime<chrono::offset::FixedOffset>,
    ) -> Self {
        let total_comments_in_last_day = manga
            .comments
            .iter()
            .filter(|c| at_time.signed_duration_since(c.date).num_days().abs() < 1)
            .collect::<Vec<_>>()
            .len();

        let mut total_replys_in_last_day: usize = 0;
        for comment in &manga.comments {
            total_replys_in_last_day += comment
                .replies
                .iter()
                .filter(|r| at_time.signed_duration_since(r.date).num_days().abs() < 1)
                .collect::<Vec<_>>()
                .len();
        }

        let current_growth: i64 =
            total_comments_in_last_day as i64 + total_replys_in_last_day as i64;

        Self {
            manga,
            last_scan: at_time.timestamp(),
            current_growth,
        }
    }

    pub fn from_mangas(
        mangas: &[manga::Manga],
        at_time: chrono::DateTime<chrono::offset::FixedOffset>,
    ) -> Vec<Self> {
        mangas
            .iter()
            .map(|m| Self::new(m.clone(), at_time))
            .collect()
    }

    pub fn scan(&mut self, at_time: chrono::DateTime<chrono::offset::FixedOffset>) -> ScanResult {
        let comments_in_last_day = self
            .manga
            .comments
            .iter()
            .filter(|c| at_time.signed_duration_since(c.date).num_days().abs() < 1)
            .collect::<Vec<_>>();
        let new_comments = self
            .manga
            .comments
            .iter()
            .filter(|c| {
                c.date.timestamp() > self.last_scan && c.date.timestamp() < at_time.timestamp()
            })
            .collect::<Vec<_>>();

        let replies_in_last_day = self
            .manga
            .comments
            .iter()
            .flat_map(|c| {
                c.replies
                    .iter()
                    .filter(|r| at_time.signed_duration_since(r.date).num_days().abs() < 1)
                    .collect::<Vec<_>>()
            })
            .collect::<Vec<_>>();
        let new_replies = self
            .manga
            .comments
            .iter()
            .flat_map(|c| c.replies.iter())
            .filter(|r| {
                r.date.timestamp() > self.last_scan && r.date.timestamp() < at_time.timestamp()
            })
            .collect::<Vec<_>>();

        let current_growth: i64 =
            comments_in_last_day.len() as i64 + replies_in_last_day.len() as i64;

        self.last_scan = at_time.timestamp();
        self.current_growth = current_growth;

        ScanResult::new(new_comments.len() as i32 + new_replies.len() as i32)
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
    pub fn new() -> Self {
        Self {
            subcount_weight: 0.0,
            last_scan_weight: 0.0,
            current_growth_weight: 0.0,
            last_chap_weight: 0.0,
        }
    }

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

    pub fn calculate_score(&self, manga: &QueuedManga) -> f64 {
        let mut score = 0.0;
        score += self.subcount_weight * manga.manga.subs as f64;
        score += self.last_scan_weight * manga.last_scan as f64;
        score += self.current_growth_weight * manga.current_growth as f64;
        if let Some(last_chap) = manga.manga.chapters.last() {
            score += self.last_chap_weight * last_chap.date.timestamp() as f64;
        }
        score
    }

    pub fn simulate_minute(
        &self,
        manga: &mut Vec<QueuedManga>,
        at_time: chrono::DateTime<chrono::offset::FixedOffset>,
        clients: i32, // number of clients
    ) -> ScanResult {
        manga.sort_by(|a, b| self.calculate_score(a).total_cmp(&self.calculate_score(b)));

        let mut i = 0;
        let mut result = ScanResult::new(0);
        for manga in manga.iter_mut() {
            i += 1;
            if i >= clients * SCAN_PER_SECOND {
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
