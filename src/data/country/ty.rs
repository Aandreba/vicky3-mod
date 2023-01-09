use std::{path::Path, collections::{HashMap}, ops::Deref};
use futures::{TryStreamExt, Stream};
use serde::{Serialize, Deserialize};
use tokio::task::spawn_blocking;
use crate::{Result, utils::{ReadDirStream, FlattenOkIter, list::ListEntry, attribute_bool, attribute_combo}, data::{read_to_string, Game}};

#[derive(Debug, Serialize, Deserialize)]
#[non_exhaustive]
pub struct CountryType {
    pub is_colonizable: bool,
    #[serde(rename = "is_unrecognized", with = "recognized_serde")]
    pub is_recognized: bool,
    pub uses_prestige: bool,
    pub has_events: bool,
    pub has_military: bool,
    pub has_economy: bool,
    pub has_politics: bool,
    pub can_research: bool,
    pub default_rank: String
}

impl ListEntry for CountryType {
    #[inline]
    fn color (&self) -> Option<eframe::epaint::Color32> {
        None
    }

    fn render_info (&mut self, ui: &mut eframe::egui::Ui, game: &Game) {
        let ranks = game.countries.ranks.borrow();
        attribute_bool(ui, "Recognized", &mut !self.is_recognized);
        attribute_bool(ui, "Prestige", &mut self.uses_prestige);
        attribute_bool(ui, "Events", &mut self.has_events);
        attribute_bool(ui, "Military", &mut self.has_military);
        attribute_bool(ui, "Economy", &mut self.has_economy);
        attribute_bool(ui, "Politics", &mut self.has_politics);
        attribute_bool(ui, "Research", &mut self.can_research);
        attribute_combo(ui, "Default Rank", &mut self.default_rank.deref(), ranks.keys().map(Deref::deref));
    }
}

impl CountryType {
    #[inline]
    pub async fn from_path (path: impl AsRef<Path>) -> Result<HashMap<String, Self>> {
        let data = read_to_string(path).await?;
        return spawn_blocking(move || jomini::text::de::from_utf8_slice(data.as_bytes())).await.unwrap()
    }

    #[inline]
    pub async fn from_common (common: &Path) -> Result<impl Stream<Item = Result<(String, Self)>>> {
        let path = common.join("country_types");
        let iter = ReadDirStream::new(tokio::fs::read_dir(path).await?)
            .map_err(<jomini::Error as From<std::io::Error>>::from)
            .try_filter_map(|x: tokio::fs::DirEntry| async move {
                if x.metadata().await.map_err(jomini::Error::from)?.is_file() {
                    return Ok(Some(Self::from_path(x.path()).await?))
                } else {
                    return Ok(None)
                }
            });

        return Ok(FlattenOkIter::new(iter))
    }
}

mod recognized_serde {
    use std::ops::{Not};
    use serde::*;

    #[inline]
    pub fn serialize<S> (this: &bool, serializer: S) -> Result<S::Ok, S::Error> where S: Serializer {
        (!this).serialize(serializer)
    }

    #[inline]
    pub fn deserialize<'de, D> (deserializer: D) -> Result<bool, D::Error> where D: Deserializer<'de> {
        bool::deserialize(deserializer).map(Not::not)
    }
}