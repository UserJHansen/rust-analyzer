use chrono::{DateTime, NaiveDateTime};
use serde::Deserialize;
use std::fmt;

#[derive(Deserialize, Clone)]
pub struct Reply {
    pub id: u32,
    #[serde(deserialize_with = "parse_date")]
    pub date: DateTime<chrono::offset::FixedOffset>,
}
#[derive(Deserialize, Clone)]
pub struct Comment {
    pub id: u32,
    #[serde(deserialize_with = "parse_date")]
    pub date: DateTime<chrono::offset::FixedOffset>,
    pub replies: Vec<Reply>,
}

#[derive(PartialEq, Eq, Clone)]
pub struct ChapterNo(u32, u32);

impl fmt::Display for ChapterNo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.1 == 0 {
            return write!(f, "{}", self.0);
        }

        write!(f, "{}.{}", self.0, self.1)
    }
}

impl ChapterNo {
    fn from_int(chapter: u32) -> Self {
        // Remove first number, assuming it is 6 digits
        let chapter = chapter - (chapter / 1000);

        let odd = chapter % 10;

        ChapterNo(chapter / 10, odd)
    }

    fn parse<'de, D>(deserialiser: D) -> Result<ChapterNo, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s: u32 = Deserialize::deserialize(deserialiser)?;
        Ok(ChapterNo::from_int(s))
    }
}

impl PartialOrd for ChapterNo {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        if self.0 == other.0 {
            return self.1.partial_cmp(&other.1);
        }

        self.0.partial_cmp(&other.0)
    }
}

#[derive(Deserialize, Clone)]
pub struct Chapter {
    #[serde(deserialize_with = "ChapterNo::parse")]
    pub chap_no: ChapterNo,
    #[serde(deserialize_with = "parse_date")]
    pub date: DateTime<chrono::offset::FixedOffset>,
}

fn parse_date<'de, D>(deserialiser: D) -> Result<DateTime<chrono::offset::FixedOffset>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let s: &str = Deserialize::deserialize(deserialiser)?;
    let datetime = NaiveDateTime::parse_from_str(s, "%Y-%m-%d %H:%M:%S").unwrap();
    Ok(DateTime::from_utc(
        datetime,
        chrono::FixedOffset::east_opt(8 * 3600).unwrap(),
    ))
}

impl fmt::Display for Chapter {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "  {} ({})", self.chap_no, self.date)
    }
}

#[derive(Deserialize, Clone)]
pub struct Manga {
    pub name: String,
    pub chapters: Vec<Chapter>,
    pub subs: usize,
    pub comments: Vec<Comment>,
}

impl fmt::Display for Manga {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.chapters.is_empty() {
            write!(f, "{} - {} (0)", self.name, self.subs)?;

            return Ok(());
        }

        write!(
            f,
            "{} ({}) {} Comments - {} Chaps",
            self.name,
            self.subs,
            self.chapters.len(),
            self.chapters.len()
        )?;
        Ok(())
    }
}
