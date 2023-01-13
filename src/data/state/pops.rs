use std::{path::Path, ptr::addr_of};
use futures::{Stream, TryStreamExt};
use jomini::JominiDeserialize;
use serde::{Deserialize, Serialize, de::{Visitor, Unexpected}, ser::SerializeMap};
use tokio::task::spawn_blocking;
use crate::{Result, data::{Ident, GamePaths, read_to_string}, utils::{ReadDirStream, FlattenOkIter}};

#[derive(Debug, Clone, PartialEq)]
pub struct RegionPops {
    // only a few pop types will exist per region, so a map is not worth it
    pub regions: Vec<(Ident, Vec<CreatePop>)> // (region, pops)
}

impl RegionPops {
    #[inline]
    pub fn get<'a> (&'a self, region: &str) -> Option<&'a [CreatePop]> {
        for (region, pops) in self.regions.iter() {
            if region == region {
                return Some(pops)
            }
        }
        return None
    }
}

impl Serialize for RegionPops {
    #[inline]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error> where S: serde::Serializer {
        let this = unsafe { &*(addr_of!(self.regions) as *const Vec<(Ident, PopsList)>) };
        return crate::utils::serde_vec_map::serialize(this, serializer)
    }
}

impl<'de> Deserialize<'de> for RegionPops {
    #[inline]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error> where D: serde::Deserializer<'de> {
        let (ptr, len, cap) = crate::utils::serde_vec_map::deserialize::<'de, Ident, PopsList, _>(deserializer)?.into_raw_parts();
        let regions = unsafe { Vec::from_raw_parts(ptr.cast(), len, cap) };
        return Ok(Self { regions })
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct CreatePop {
    pub culture: Ident,
    #[serde(default)]
    pub religion: Option<Ident>,
    pub size: u64
}

impl RegionPops {
    #[inline]
    pub async fn from_path (path: impl AsRef<Path>) -> Result<impl Iterator<Item = (Ident, Self)>> {
        #[derive(Deserialize)]
        pub struct StateRegionPops {
            // only a few ammount of entries will exist, so a map is not worth it
            #[serde(with = "crate::utils::serde_vec_map")]
            pub states: Vec<(Ident, RegionPops)> // (state, regions)
        }

        #[derive(JominiDeserialize)]
        struct Inner {
            #[jomini(alias = "POPS", duplicated)]
            pops: Vec<StateRegionPops>
        }

        let data = read_to_string(path).await?;
        return spawn_blocking(move ||
            jomini::text::de::from_utf8_slice::<Inner>(data.as_bytes()).map(|x| x.pops.into_iter().flat_map(|x| x.states))
        ).await.unwrap()
    }

    #[inline]
    pub async fn from_game (game: &GamePaths) -> Result<impl Stream<Item = Result<(Ident, Self)>>> {
        let path = game.history().join("states");
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

#[repr(transparent)]
struct PopsList (Vec<CreatePop>);

impl Serialize for PopsList {
    #[inline]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error> where S: serde::Serializer {
        let mut ser = serializer.serialize_map(Some(self.0.len()))?;
        for pop in self.0.iter() {
            ser.serialize_entry("create_pop", pop)?;
        }
        return ser.end()
    }
}

impl<'de> Deserialize<'de> for PopsList {
    #[inline]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error> where D: serde::Deserializer<'de> {
        struct LocalVisitor;
        impl<'de> Visitor<'de> for LocalVisitor {
            type Value = Vec<CreatePop>;

            #[inline]
            fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                write!(f, "a list of 'create_pop's")
            }

            fn visit_map<A>(self, mut map: A) -> std::result::Result<Self::Value, A::Error> where A: serde::de::MapAccess<'de>, {
                let mut result = Vec::with_capacity(map.size_hint().unwrap_or_default());
                loop {
                    match map.next_key::<&str>()? {
                        Some("create_pop") => result.push(map.next_value()?),
                        Some(other) => return Err(serde::de::Error::invalid_value(Unexpected::Str(other), &"create_pop")),
                        None => break
                    }
                }
                return Ok(result)
            }
        }

        return deserializer.deserialize_map(LocalVisitor).map(Self)
    }
}

impl From<Vec<CreatePop>> for PopsList {
    #[inline]
    fn from(value: Vec<CreatePop>) -> Self {
        Self(value)
    }
}

impl Into<Vec<CreatePop>> for PopsList {
    #[inline]
    fn into(self) -> Vec<CreatePop> {
        self.0
    }
}