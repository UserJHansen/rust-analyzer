use chrono::Datelike;

mod types;
use crate::{calculate_peaks, schedule::types::*, types::*};

pub fn calculate_schedule(m: &Vec<Manga>) -> Vec<(&Manga, Schedule)> {
    let mut sched = Vec::new();
    for manga in m {
        if manga.chapters.is_empty() {
            sched.push((manga, Schedule::Unknown));

            continue;
        }

        let mut prob_days = [0; 7];
        let mut prob_every = [0; 31];
        let mut prob_month = [0; 31];

        let mut sorted_chaps = manga.chapters.iter().collect::<Vec<&Chapter>>();
        sorted_chaps.sort_by(|a, b| a.date.partial_cmp(&b.date).unwrap());

        let mut last_chap = sorted_chaps[0].date.num_days_from_ce();

        for chapter in sorted_chaps {
            let date = chapter.date;
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
        if !days_peaks.1.is_empty() && days_peaks.0 > 0.5f64 {
            match days_peaks.1.len() {
                1 => sched.push((
                    manga,
                    Schedule::Day(NumberedWeekday::from_usize(days_peaks.1[0])),
                )),
                _ => {
                    let mut schedules = Vec::new();
                    for day in days_peaks.1 {
                        schedules.push(Schedule::Day(NumberedWeekday::from_usize(day)));
                    }
                    sched.push((manga, Schedule::Multiple(schedules)));
                }
            }

            continue;
        }

        let every_peaks = calculate_peaks::calculate_peaks(prob_every.to_vec());
        if !every_peaks.1.is_empty() && every_peaks.0 > 0.5f64 {
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
        if !month_peaks.1.is_empty() && month_peaks.0 > 0.5f64 {
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
    }

    sched
}
