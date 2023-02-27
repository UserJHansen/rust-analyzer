use chrono::{DateTime, NaiveDateTime};
use serde::Deserialize;
use std::{fmt, str::FromStr};

fn parse_mangasee_time<'de, D>(
    deserialiser: D,
) -> Result<DateTime<chrono::offset::FixedOffset>, D::Error>
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
    fn from_str(s: &str) -> Result<Self, <u32 as FromStr>::Err> {
        let chapter = s[1..s.len() - 1].parse::<u32>()?;
        let odd = s
            .chars()
            .last()
            .expect("Invalid chapter number")
            .to_digit(10)
            .expect("Invalid chapter number");

        Ok(ChapterNo(chapter, odd))
    }
}

impl PartialEq for ChapterNo {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0 && self.1 == other.1
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

fn parse_chapter_no<'de, D>(deserialiser: D) -> Result<ChapterNo, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let s: &str = Deserialize::deserialize(deserialiser)?;
    let res = ChapterNo::from_str(s);
    res.map_err(|e| serde::de::Error::custom(e))
}

#[derive(Deserialize)]
pub struct Chapter {
    #[serde(rename(deserialize = "Chapter"), deserialize_with = "parse_chapter_no")]
    pub chapter: ChapterNo,
    #[serde(rename(deserialize = "Type"))]
    pub chapter_type: String,
    #[serde(rename(deserialize = "Date"), deserialize_with = "parse_mangasee_time")]
    pub date: DateTime<chrono::offset::FixedOffset>,
    #[serde(rename(deserialize = "ChapterName"))]
    pub name: Option<String>,
}

impl fmt::Display for Chapter {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "  {} {}{} ({})",
            self.chapter_type,
            self.chapter,
            if let Some(name) = &self.name {
                format!(" {}", name)
            } else {
                "".to_string()
            },
            self.date
        )
    }
}

#[derive(Deserialize)]
pub struct Manga {
    pub name: String,
    pub chapters: Vec<Chapter>,
    pub subs: usize,
}

impl fmt::Display for Manga {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.chapters.is_empty() {
            write!(f, "{} - {} (0)", self.name, self.subs)?;

            return Ok(());
        }

        write!(
            f,
            "{} - {} ({}): \n",
            self.name,
            self.subs,
            self.chapters.len()
        )?;

        let chaps = &self.chapters;
        let mut pointers: Vec<&Chapter> = chaps.iter().collect();

        pointers.sort_by(|a, b| a.chapter.partial_cmp(&b.chapter).unwrap());

        write!(
            f,
            "{}",
            pointers
                .iter()
                .map(|c| format!("{}", c))
                .collect::<Vec<String>>()
                .join("\n")
        )?;

        Ok(())
    }
}
