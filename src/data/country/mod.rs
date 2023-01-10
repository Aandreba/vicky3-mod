use std::{collections::{BTreeMap}};
use futures::{TryFutureExt, TryStreamExt};
use crate::{Result, utils::refcell::RefCell};
use super::{GamePaths, Ident};

flat_mod! { def, ty, rank, tier }

#[derive(Debug)]
pub struct GameCountry {
    pub ranks: RefCell<BTreeMap<String, CountryRank>>,
    pub tys: RefCell<BTreeMap<String, CountryType>>,
    pub definitions: RefCell<BTreeMap<Ident, CountryDefinition>>,
    // todo history
}

impl GameCountry {
    #[inline]
    pub async fn from_game (game: &GamePaths) -> Result<Self> {
        let (ranks, tys, definitions) = futures::try_join! {
            CountryRank::from_game(game).and_then(TryStreamExt::try_collect::<BTreeMap<_, _>>),
            CountryType::from_game(game).and_then(TryStreamExt::try_collect::<BTreeMap<_, _>>),
            CountryDefinition::from_game(game).and_then(TryStreamExt::try_collect::<BTreeMap<_, _>>)
        }?;

        return Ok(Self {
            ranks: RefCell::new(ranks),
            tys: RefCell::new(tys),
            definitions: RefCell::new(definitions)
        })
    }
}