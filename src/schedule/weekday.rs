#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum NumberedWeekday {
    /// Monday.
    Mon = 0,
    /// Tuesday.
    Tue = 1,
    /// Wednesday.
    Wed = 2,
    /// Thursday.
    Thu = 3,
    /// Friday.
    Fri = 4,
    /// Saturday.
    Sat = 5,
    /// Sunday.
    Sun = 6,
}

impl NumberedWeekday {
    pub fn from_usize(day: usize) -> NumberedWeekday {
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
