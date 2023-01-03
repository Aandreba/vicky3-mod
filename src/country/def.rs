use std::path::Path;
use itertools::Itertools;
use jomini::JominiDeserialize;
use crate::{Color, Result, Game};

#[derive(Debug, Clone, PartialEq, JominiDeserialize)]
#[non_exhaustive]
pub struct RawDefinition {
    pub color: Color,
    pub country_type: String,
    pub tier: String,
    pub cultures: Vec<String>,
    pub capital: String
}

impl RawDefinition {
    #[inline]
    pub fn import_from_game () -> Result<impl Iterator<Item = Result<Self>>> {
        let defs = Game::common().join("country_definitions");
        let iter = std::fs::read_dir(defs)?
            .map(|x| x.map_err(jomini::Error::from))
            .filter_ok(|x| x.file_type().unwrap().is_file())
            .map_ok(|x| Self::from_path(x.path()))
            .flatten();

        return Ok(iter)
    }

    #[inline]
    pub fn from_path (path: impl AsRef<Path>) -> Result<Self> {
        let contents = std::fs::read_to_string(path)?;
        return jomini::text::de::from_utf8_slice(contents.as_bytes());
    }
}