use crate::app::SwitchToComic;
use color_eyre::{eyre::Ok, Result};
use dirs::state_dir;
use image::DynamicImage;
use isahc::ReadResponseExt;
use rand::{rngs::ThreadRng, thread_rng, Rng};
use serde_json::{json, Value};
use std::{fs, path::PathBuf};
pub struct ComicDownloader {
    last_seen_comic: u64,
    bookmarked_comic: Option<u64>,
    rng: ThreadRng,
}

pub struct Comic {
    pub number: u64,
    pub name: String,
    pub alt_text: String,
    pub date_uploaded: String,
    pub image: DynamicImage,
}

impl Comic {
    fn new(json: Value) -> Result<Comic> {
        let alt_text = json["alt"].as_str().unwrap().to_string();
        let date_uploaded = format!(
            "{}-{:02}-{:02}",
            json["year"].as_str().unwrap(),
            json["month"].as_str().unwrap().parse::<u16>().unwrap(),
            json["day"].as_str().unwrap().parse::<u16>().unwrap(),
        );

        let image = image::load_from_memory(&isahc::get(json["img"].as_str().unwrap())?.bytes()?)?;
        Ok(Comic {
            alt_text,
            date_uploaded,
            image,
            number: json["num"].as_u64().unwrap(),
            name: json["title"].as_str().unwrap().to_owned(),
        })
    }
}

impl ComicDownloader {
    pub fn new(switch_to_comic: SwitchToComic) -> Result<(Self, Comic)> {
        let file = fs::read_to_string(Self::get_path_to_state_file()).unwrap_or_default();
        let json: Value = serde_json::from_str(&file).unwrap_or_default();
        let mut comic_downloader = Self {
            last_seen_comic: json["last_seen_comic"].as_u64().unwrap_or(1),
            bookmarked_comic: json["bookmarked_comic"].as_u64(),
            rng: thread_rng(),
        };
        let comic = comic_downloader.switch(switch_to_comic)?;
        Ok((comic_downloader, comic))
    }
    pub fn switch(&mut self, switch_to_comic: SwitchToComic) -> Result<Comic> {
        self.switch_to_comic(switch_to_comic)?;
        let comic = Self::download(self.last_seen_comic)?;
        Ok(comic)
    }

    pub fn bookmark_comic(&mut self) {
        self.bookmarked_comic = Some(self.last_seen_comic);
    }

    pub fn save(&self) -> Result<()> {
        let json = json!({"last_seen_comic": self.last_seen_comic, "bookmarked_comic": self.bookmarked_comic}).to_string();
        let path = Self::get_path_to_state_file();
        fs::create_dir_all(path.parent().unwrap())?;
        Ok(fs::write(path, json)?)
    }

    fn get_path_to_state_file() -> PathBuf {
        let mut path = state_dir().unwrap_or_default();
        path.push("oxikcde");
        path.push("comic_downloader.json");
        path
    }

    fn download(number: u64) -> Result<Comic> {
        let json = Self::download_json(Some(number))?;
        Ok(Comic::new(json)?)
    }

    fn download_json(number: Option<u64>) -> Result<Value> {
        let text = isahc::get(match number {
            Some(number) => format!("https://xkcd.com/{}/info.0.json", number),
            _ => String::from("https://xkcd.com/info.0.json"),
        })?
        .text()
        .expect("XKCD should always give valid json");
        Ok(serde_json::from_str(&text).expect("XKCD should always give valid json"))
    }

    fn switch_to_comic(&mut self, switch_to_comic: SwitchToComic) -> Result<()> {
        self.last_seen_comic = match switch_to_comic {
            SwitchToComic::Next => self.last_seen_comic + 1,
            SwitchToComic::Previous => self.last_seen_comic - 1,
            SwitchToComic::Latest => Self::get_latest_comic()?,
            SwitchToComic::First => 1,
            SwitchToComic::Random => self.rng.gen_range(1..Self::get_latest_comic()?),
            SwitchToComic::Bookmarked => self.bookmarked_comic.unwrap_or(self.last_seen_comic),
            SwitchToComic::Specific(num) => num,
            SwitchToComic::LastSeen => self.last_seen_comic,
        };
        Ok(())
    }

    fn get_latest_comic() -> Result<u64> {
        let json = Self::download_json(None)?;
        Ok(json["num"].as_u64().unwrap())
    }
}
