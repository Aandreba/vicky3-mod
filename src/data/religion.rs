use std::{path::{Path}, collections::HashMap};
use eframe::egui::{Ui};
use futures::{Stream, TryStreamExt};
use jomini::JominiDeserialize;
use tokio::task::spawn_blocking;
use crate::{Result, utils::{list::ListEntry, attribute_list}};
use crate::utils::{ReadDirStream, FlattenOkIter};
use super::{Color, read_to_string, Game, GamePaths};

#[derive(Debug, Clone, PartialEq, JominiDeserialize)]
#[non_exhaustive]
pub struct Religion {
    pub texture: Box<Path>,
    // religion traits, different from other kinds of traits
    pub traits: Box<[String]>,
    pub color: Color,
    #[jomini(default)]
    pub taboos: Box<[String]>
}

impl Religion {
    #[inline]
    pub async fn from_path (path: impl AsRef<Path>) -> Result<HashMap<String, Self>> {
        let data = read_to_string(path).await?;
        return spawn_blocking(move || jomini::text::de::from_utf8_slice(data.as_bytes())).await.unwrap()
    }

    #[inline]
    pub async fn from_game (game: &GamePaths) -> Result<impl Stream<Item = Result<(String, Self)>>> {
        let path = game.common().join("religions");
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

impl ListEntry for Religion {
    #[inline]
    fn color (&self) -> Option<eframe::epaint::Color32> {
        Some(self.color.into())
    }

    #[inline]
    fn render_info (&mut self, ui: &mut Ui, game: &Game) {
        attribute_list(ui, "Traits", self.traits.iter_mut());
        attribute_list(ui, "Taboos", self.taboos.iter_mut());
        // todo texture
    }
}