use std::{collections::HashMap, path::Path};
use itertools::Itertools;
use jomini::JominiDeserialize;
use crate::{Color, Result, Str};

#[derive(Debug, Clone, PartialEq, JominiDeserialize)]
#[non_exhaustive]
pub struct RawCulture {
    pub color: Color,
    pub religion: Str,
    #[jomini(default)]
    pub traits: Vec<Str>,
    #[jomini(default)]
    pub male_common_first_names: Vec<Str>,
    #[jomini(default)]
    pub female_common_first_names: Vec<Str>,
    #[jomini(default)]
    pub noble_last_names: Vec<Str>,
    #[jomini(default)]
    pub common_last_names: Vec<Str>,
    #[jomini(default)]
    pub male_regal_first_names: Vec<Str>,
    #[jomini(default)]
    pub female_regal_first_names: Vec<Str>,
    pub graphics: Str,
    pub ethnicities: HashMap<u32, Str>
}

impl RawCulture {
    #[inline]
    pub fn from_path (path: impl AsRef<Path>) -> Result<HashMap<Str, Self>> {
        let data = std::fs::read_to_string(path)?;
        return jomini::text::de::from_utf8_slice(data.as_bytes())
    }

    #[inline]
    pub fn from_common (common: &Path) -> Result<impl Iterator<Item = Result<(Str, Self)>>> {
        let path = common.join("cultures");
        let iter = std::fs::read_dir(path)?
            .filter_ok(|x| x.metadata().unwrap().is_file())
            .map_ok(|x| Self::from_path(x.path()))
            .flatten()
            .flatten_ok();

        return Ok(iter)
    }
}