use std::{path::Path};
use futures::{Stream, TryStreamExt};
use jomini::JominiDeserialize;
use serde::{Serialize, Deserialize};
use tokio::task::spawn_blocking;
use crate::{Result, utils::{ReadDirStream, FlattenOkIter}, data::{read_to_string, GamePaths, Ident}};

pub type NamedStateDefinition<'a> = (&'a String, &'a StateDefinition);

#[derive(Debug, Clone, PartialEq, Serialize, JominiDeserialize)]
pub struct StateCreation {
    pub country: Ident,
    pub owned_provinces: Vec<String>, // todo serde as hex nums
    #[jomini(default, duplicated)]
    pub state_type: Vec<Ident> // todo probably state traits
}

#[derive(Debug, Clone, PartialEq, JominiDeserialize)]
pub struct StateDefinition {
    #[jomini(alias = "create_state", duplicated)]
    pub states: Vec<StateCreation>,
    #[jomini(alias = "add_homeland", duplicated)]
    pub homelands: Vec<Ident>
}

impl StateDefinition {
    #[inline]
    pub async fn from_path (path: impl AsRef<Path>) -> Result<impl Iterator<Item = (Ident, Self)>> {
        #[derive(Deserialize)]
        struct States (
            #[serde(deserialize_with = "crate::utils::serde_vec_map::deserialize")]
            Vec<(Ident, StateDefinition)>
        );

        #[derive(JominiDeserialize)]
        struct Inner {
            #[jomini(alias = "STATES", duplicated)]
            states: Vec<States>
        }

        let data = read_to_string(path).await?;
        return spawn_blocking(move ||
            jomini::text::de::from_utf8_slice::<Inner>(data.as_bytes()).map(|x| x.states.into_iter().flat_map(|x| x.0))
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