use std::{collections::BTreeMap};
use crate::{utils::refcell::RefCell, Result};
use super::{GamePaths, Ident};
use futures::stream::TryStreamExt;
use futures::TryFutureExt;
flat_mod! { def, pops }

#[derive(Debug, PartialEq)]
pub struct StateRegion<'a> {
    def: &'a RegionDefinition,
    pops: &'a [CreatePop]
}

#[derive(Debug, PartialEq)]
pub struct State<'a> {
    def: &'a StateDefinition,
    pops: &'a RegionPops
}


impl<'a> State<'a> {
    #[inline]
    pub fn get<'b> (&'b self, region: &str) -> Option<StateRegion<'b>> where 'a: 'b {
        let def = self.def.regions.iter().find(|x| &x.country == region)?;
        let pops = self.pops.get(region)?;
        return Some(StateRegion { def, pops })
    }
}

#[derive(Debug)]
pub struct GameState {
    pub defs: RefCell<BTreeMap<Ident, StateDefinition>>,
    pub pops: RefCell<BTreeMap<Ident, RegionPops>>
}

impl GameState {
    pub fn get<'a> (&'a self, state: &str) -> Option<State<'a>> {
        todo!()
    }
}

impl GameState {
    #[inline]
    pub async fn from_game (game: &GamePaths) -> Result<Self> {
        let (defs, pops) = futures::try_join! {
            StateDefinition::from_game(game).and_then(TryStreamExt::try_collect::<BTreeMap<_, _>>),
            RegionPops::from_game(game).and_then(TryStreamExt::try_collect::<BTreeMap<_, _>>)
        }?;

        return Ok(Self {
            defs: RefCell::new(defs),
            pops: RefCell::new(pops)
        })
    }
}