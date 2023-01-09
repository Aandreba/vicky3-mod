use std::{collections::HashMap, path::Path};
use futures::{Stream, TryStreamExt};
use jomini::JominiDeserialize;
use tokio::task::spawn_blocking;
use crate::Result;
use crate::utils::list::ListEntry;
use crate::utils::{ReadDirStream, FlattenOkIter, attribute, attribute_list};
use super::{Color, read_to_string};

#[derive(Debug, Clone, PartialEq, JominiDeserialize)]
#[non_exhaustive]
pub struct Culture {
    pub color: Color,
    pub religion: String,
    #[jomini(default)]
    pub traits: Box<[String]>,
    #[jomini(default)]
    pub male_common_first_names: Box<[String]>,
    #[jomini(default)]
    pub female_common_first_names: Box<[String]>,
    #[jomini(default)]
    pub noble_last_names: Box<[String]>,
    #[jomini(default)]
    pub common_last_names: Box<[String]>,
    #[jomini(default)]
    pub male_regal_first_names: Box<[String]>,
    #[jomini(default)]
    pub female_regal_first_names: Box<[String]>,
    pub graphics: String,
    pub ethnicities: HashMap<u32, String>
}

impl Culture {
    #[inline]
    pub async fn from_path (path: impl AsRef<Path>) -> Result<HashMap<String, Self>> {
        let data = read_to_string(path).await?;
        return spawn_blocking(move || jomini::text::de::from_utf8_slice(data.as_bytes())).await.unwrap()
    }

    #[inline]
    pub async fn from_common (common: &Path) -> Result<impl Stream<Item = Result<(String, Self)>>> {
        let path = common.join("cultures");
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

impl ListEntry for Culture {
    #[inline]
    fn color (&self) -> Option<eframe::epaint::Color32> {
        Some(self.color.into())
    }

    #[inline]
    fn render_info (&mut self, ui: &mut eframe::egui::Ui) {
        attribute(ui, "Religion", &mut self.religion);
        attribute_list(ui, "Traits", self.traits.iter_mut());
        attribute(ui, "Graphics", &mut self.graphics);
    }
}