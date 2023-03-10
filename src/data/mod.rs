#![cfg_attr(feature = "nightly", feature(iterator_try_collect))]

macro_rules! flat_mod {
    ($($i:ident),+) => {
        $(
            mod $i;
            pub use $i::*;
        )+
    };
}

pub mod country;
pub mod state;
pub mod culture;
pub mod religion;

flat_mod! { color, ident }

use std::{path::{Path, PathBuf}, collections::{BTreeMap}};
use country::GameCountry;
use culture::Culture;
use futures::{Stream, TryStreamExt, TryFutureExt};
use into_string::IntoPathBuf;
use itertools::Itertools;
use religion::Religion;
use crate::{utils::{FlattenOkIter, refcell::RefCell}, Result};
use self::state::GameState;

#[derive(Debug, Clone, PartialEq)]
pub struct GamePaths {
    game: PathBuf,
    common: once_cell::unsync::OnceCell<PathBuf>,
    history: once_cell::unsync::OnceCell<PathBuf>
}

impl GamePaths {
    pub fn new (game: impl IntoPathBuf) -> Self {
        return Self {
            game: game.into_path_buf(),
            common: once_cell::unsync::OnceCell::new(),
            history: once_cell::unsync::OnceCell::new(),
        }
    }

    #[inline]
    pub fn game (&self) -> &Path {
        return &self.game
    }

    #[inline]
    pub fn common (&self) -> &Path {
        return self.common.get_or_init(|| self.game().join("common"))
    }

    #[inline]
    pub fn history (&self) -> &Path {
        return self.history.get_or_init(|| self.common().join("history"))
    }
}

#[derive(Debug)]
#[non_exhaustive]
pub struct Game {
    pub path: GamePaths,
    pub countries: GameCountry,
    pub states: GameState,
    pub religions: RefCell<BTreeMap<String, Religion>>,
    pub cultures: RefCell<BTreeMap<String, Culture>>
}

impl Game {
    #[inline]
    pub async fn new<P: IntoPathBuf> (path: P) -> Result<Self> {
        let path = GamePaths::new(path);
        let (countries, states, religions, cultures) = futures::try_join! {
            GameCountry::from_game(&path),
            GameState::from_game(&path),
            Religion::from_game(&path).and_then(TryStreamExt::try_collect::<BTreeMap<_, _>>),
            Culture::from_game(&path).and_then(TryStreamExt::try_collect::<BTreeMap<_, _>>)
        }?;

        return Ok(Self {
            path,
            countries,
            states,
            religions: RefCell::new(religions),
            cultures: RefCell::new(cultures)
        })
    }
}

#[inline]
pub(crate) async fn read_to_string (path: impl AsRef<Path>) -> std::io::Result<String> {
    let mut file = tokio::fs::File::open(path).await?;
    let mut string = String::new();
    tokio::io::AsyncReadExt::read_to_string(&mut file, &mut string).await?;
    Ok(string)
}

#[inline]
pub(crate) fn flat_map_ok<T, E, I, F, U> (iter: I, f: F) -> impl Iterator<Item = ::core::result::Result<<U as IntoIterator>::Item, E>> where
    I: IntoIterator<Item = ::core::result::Result<T, E>>,
    F: FnMut(T) -> U,
    U: IntoIterator
{
    return itertools::Itertools::map_ok(iter.into_iter(), f).flatten_ok();
}

#[inline]
pub(crate) fn stream_flat_map_ok<T, E, S, F, U> (stream: S, f: F) -> impl Stream<Item = ::core::result::Result<<U as IntoIterator>::Item, E>> where
    S: Stream<Item = ::core::result::Result<T, E>>,
    F: FnMut(T) -> U,
    U: IntoIterator
{
    return FlattenOkIter::new(stream.map_ok(f))
    //return itertools::Itertools::map_ok(iter.into_iter(), f).flatten_ok();
}