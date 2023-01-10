use std::{collections::BTreeMap};
use crate::{utils::refcell::RefCell, Result};
use super::GamePaths;
use futures::stream::TryStreamExt;
use futures::TryFutureExt;

flat_mod! { def }

pub struct GameState {
    pub defs: RefCell<BTreeMap<String, StateDefinition>>
}

impl GameState {
    #[inline]
    pub async fn from_game (game: &GamePaths) -> Result<Self> {
        let (defs, _) = futures::try_join! {
            StateDefinition::from_game(game).and_then(TryStreamExt::try_collect::<BTreeMap<_, _>>),
            futures::future::ready(Ok(()))
        }?;

        return Ok(Self {
            defs: RefCell::new(defs),
        })
    }
}