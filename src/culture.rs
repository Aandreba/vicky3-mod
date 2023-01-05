use std::{collections::HashMap, path::Path};
use futures::{Stream, TryStreamExt};
use jomini::JominiDeserialize;
use tokio::task::spawn_blocking;
use crate::{Color, Result, Str, read_to_string, utils::{ReadDirStream, FlattenOkIter, GetStr, stream_and_then}, religion::{Religion, NamedReligion}};

pub type CultureRef<'a> = &'a Culture<'a>;
pub type NamedCulture<'a> = (&'a str, CultureRef<'a>);

#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub struct Culture<'a> {
    pub color: Color,
    pub religion: NamedReligion<'a>,
    // pub traits: Vec<Str>,
    pub male_common_first_names: Vec<Str>,
    pub female_common_first_names: Vec<Str>,
    pub noble_last_names: Vec<Str>,
    pub common_last_names: Vec<Str>,
    pub male_regal_first_names: Vec<Str>,
    pub female_regal_first_names: Vec<Str>,
    // pub graphics: Str,
    // pub ethnicities: HashMap<u32, Str>
}

impl<'a> Culture<'a> {
    #[inline]
    pub fn from_raw (raw: RawCulture, religions: &'a HashMap<Str, Religion>) -> Result<Self> {
        return Ok(Self {
            color: raw.color,
            religion: religions.try_get_str_value(&raw.religion)?,
            male_common_first_names: raw.male_common_first_names,
            female_common_first_names: raw.female_common_first_names,
            noble_last_names: raw.noble_last_names,
            common_last_names: raw.common_last_names,
            male_regal_first_names: raw.male_regal_first_names,
            female_regal_first_names: raw.female_regal_first_names,
        })
    }

    #[inline]
    pub async fn from_common (common: &Path, religions: &'a HashMap<Str, Religion>) -> Result<impl Stream<Item = Result<(Str, Culture<'a>)>>> {
        let iter = RawCulture::from_common(common).await?;
        return Ok(stream_and_then(
            iter,
            |(name, raw)| Ok((name, Self::from_raw(raw, religions)?)))
        );
    }
}

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
    pub async fn from_path (path: impl AsRef<Path>) -> Result<HashMap<Str, Self>> {
        let data = read_to_string(path).await?;
        return spawn_blocking(move || jomini::text::de::from_utf8_slice(data.as_bytes())).await.unwrap()
    }

    #[inline]
    pub async fn from_common (common: &Path) -> Result<impl Stream<Item = Result<(Str, Self)>>> {
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