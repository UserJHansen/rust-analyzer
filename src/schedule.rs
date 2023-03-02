use chrono::{Datelike, Utc};

mod weekday;
use crate::{calculate_peaks, manga};

use weekday::NumberedWeekday;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Schedule {
    Day(NumberedWeekday), // Specific day of the week
    Every(u8),            // Every x days
    Monthly(u8),          // Day of the month
    Multiple(Vec<Schedule>),
    Unknown,
}

impl Schedule {
    pub fn calculate_schedule(m: &Vec<manga::Manga>) -> Vec<(&manga::Manga, Schedule)> {
        let mut sched = Vec::new();
        for manga in m {
            if manga.chapters.is_empty() {
                sched.push((manga, Schedule::Unknown));

                continue;
            }

            let mut prob_days = [0; 7];
            let mut prob_every = [0; 31];
            let mut prob_month = [0; 31];

            let mut sorted_chaps = manga.chapters.iter().collect::<Vec<&manga::Chapter>>();
            sorted_chaps.sort_by(|a, b| a.date.partial_cmp(&b.date).unwrap());

            let sorted_chaps = sorted_chaps.iter().map(|c| c.date);

            // Remove duplicates
            let mut last = 0;
            let sorted_chaps = sorted_chaps
                .filter(|c| {
                    if last == c.num_days_from_ce() {
                        false
                    } else {
                        last = c.num_days_from_ce();
                        true
                    }
                })
                .collect::<Vec<_>>();

            let last_date = sorted_chaps.last().unwrap().num_days_from_ce();
            // Filter manga which have less than 30 days between the first and last chapter
            // or have not been updated in the last 60 days
            if sorted_chaps.last().unwrap().num_days_from_ce() - sorted_chaps[0].num_days_from_ce()
                < 30
                || Utc::now().num_days_from_ce() - last_date > 30
            {
                sched.push((manga, Schedule::Unknown));
                continue;
            }

            let mut last_chap = sorted_chaps[0].num_days_from_ce();

            for date in sorted_chaps {
                let day = date.weekday().num_days_from_monday();

                prob_days[day as usize] += 1;

                let day = date.day0();
                prob_month[day as usize] += 1;

                let diff = date.num_days_from_ce() - last_chap;
                if diff > 30 || diff == 0 {
                    continue;
                }
                prob_every[diff as usize] += 1;
                last_chap = date.num_days_from_ce();
            }

            let days_peaks = calculate_peaks::calculate_peaks(prob_days.to_vec());
            if !days_peaks.1.is_empty() && days_peaks.0 > 0.6f64 {
                match days_peaks.1.len() {
                    1 => sched.push((
                        manga,
                        Schedule::Day(NumberedWeekday::from_usize(days_peaks.1[0])),
                    )),
                    _ => {
                        let mut schedules = Vec::new();
                        for day in days_peaks.1 {
                            schedules
                                .push(Schedule::Day(weekday::NumberedWeekday::from_usize(day)));
                        }
                        sched.push((manga, Schedule::Multiple(schedules)));
                    }
                }

                continue;
            }

            let every_peaks = calculate_peaks::calculate_peaks(prob_every.to_vec());
            if !every_peaks.1.is_empty() && every_peaks.0 > 0.6f64 {
                match every_peaks.1.len() {
                    1 => sched.push((manga, Schedule::Every(every_peaks.1[0] as u8))),
                    _ => {
                        let mut schedules = Vec::new();
                        for day in every_peaks.1 {
                            schedules.push(Schedule::Every(day as u8));
                        }
                        sched.push((manga, Schedule::Multiple(schedules)));
                    }
                }

                continue;
            }

            let month_peaks = calculate_peaks::calculate_peaks(prob_month.to_vec());
            if !month_peaks.1.is_empty() && month_peaks.0 > 0.6f64 {
                match month_peaks.1.len() {
                    1 => sched.push((manga, Schedule::Monthly(month_peaks.1[0] as u8))),
                    _ => {
                        let mut schedules = Vec::new();
                        for day in month_peaks.1 {
                            schedules.push(Schedule::Monthly(day as u8));
                        }
                        sched.push((manga, Schedule::Multiple(schedules)));
                    }
                }

                continue;
            }

            sched.push((manga, Schedule::Unknown));
        }

        sched
    }
}
