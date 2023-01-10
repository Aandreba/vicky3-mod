use std::{path::Path};
use futures::{Stream, TryStreamExt};
use jomini::JominiDeserialize;
use serde::{Serialize};
use tokio::task::spawn_blocking;
use crate::{Result, utils::{ReadDirStream, FlattenOkIter}, data::{read_to_string, GamePaths}};

pub type NamedStateDefinition<'a> = (&'a String, &'a StateDefinition);

#[derive(Debug, Clone, PartialEq, Serialize, JominiDeserialize)]
pub struct StateCreation {
    country: String,
    owned_provinces: Vec<String>, // todo serde as hex nums
    #[jomini(default)]
    state_type: Option<String>
}

#[derive(Debug, Clone, PartialEq, Serialize, JominiDeserialize)]
pub struct StateDefinition {
    #[jomini(alias = "create_state", duplicated)]
    states: Vec<StateCreation>,
    #[jomini(alias = "add_homeland", duplicated)]
    homelands: Vec<String>
}

impl StateDefinition {
    #[inline]
    pub async fn from_path (path: impl AsRef<Path>) -> Result<Vec<(String, Self)>> {
        #[derive(JominiDeserialize)]
        struct Inner {
            #[jomini(alias = "STATES", duplicated)]
            states: Vec<(String, StateDefinition)>
        }

        let data = read_to_string(path).await?;
        return spawn_blocking(move ||
            jomini::text::de::from_utf8_slice::<Inner>(data.as_bytes()).map(|x| x.states)
        ).await.unwrap()
    }

    #[inline]
    pub async fn from_game (game: &GamePaths) -> Result<impl Stream<Item = Result<(String, Self)>>> {
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