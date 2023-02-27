use std::cmp::Ordering;

pub trait WeekdayWithNumber: Sized {
    fn from_usize(day: usize) -> Self;
}
pub type NumberedWeekday = chrono::Weekday;
impl WeekdayWithNumber for NumberedWeekday {
    fn from_usize(day: usize) -> chrono::Weekday {
        match day {
            0 => NumberedWeekday::Mon,
            1 => NumberedWeekday::Tue,
            2 => NumberedWeekday::Wed,
            3 => NumberedWeekday::Thu,
            4 => NumberedWeekday::Fri,
            5 => NumberedWeekday::Sat,
            6 => NumberedWeekday::Sun,
            _ => panic!("Invalid day number"),
        }
    }
}

#[derive(Debug)]
pub enum Schedule {
    Day(NumberedWeekday), // Specific day of the week
    Every(u8),            // Every x days
    Monthly(u8),          // Day of the month
    Multiple(Vec<Schedule>),
    Unknown,
}

impl PartialEq for Schedule {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Schedule::Day(a), Schedule::Day(b)) => a == b,
            (Schedule::Every(a), Schedule::Every(b)) => a == b,
            (Schedule::Monthly(a), Schedule::Monthly(b)) => a == b,
            (Schedule::Unknown, Schedule::Unknown) => true,
            _ => false,
        }
    }
}

impl PartialOrd for Schedule {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Eq for Schedule {}

impl Ord for Schedule {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        match (self, other) {
            (Schedule::Unknown, Schedule::Unknown) => Ordering::Equal,
            (Schedule::Unknown, _) => Ordering::Less,
            (_, Schedule::Unknown) => Ordering::Greater,
            (Schedule::Monthly(a), Schedule::Monthly(b)) => a.cmp(b),
            (Schedule::Monthly(_), _) => Ordering::Less,
            (_, Schedule::Monthly(_)) => Ordering::Greater,
            (Schedule::Every(a), Schedule::Every(b)) => a.cmp(b),
            (Schedule::Every(_), _) => Ordering::Less,
            (_, Schedule::Every(_)) => Ordering::Greater,
            (Schedule::Day(a), Schedule::Day(b)) => {
                a.num_days_from_monday().cmp(&b.num_days_from_monday())
            }
            (Schedule::Day(_), _) => Ordering::Less,
            (_, Schedule::Day(_)) => Ordering::Greater,
            (Schedule::Multiple(_), Schedule::Multiple(_)) => Ordering::Equal,
        }
    }
}
