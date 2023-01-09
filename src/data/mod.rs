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
pub mod culture;
pub mod religion;

flat_mod! { color }

use std::{path::{Path, PathBuf}, collections::{HashMap, BTreeMap}};
use country::GameCountry;
use culture::Culture;
use futures::{Stream, TryStreamExt, TryFutureExt};
use into_string::IntoPathBuf;
use itertools::Itertools;
use religion::Religion;
use crate::{utils::FlattenOkIter, Result};
use crate::Str;

#[derive(Debug)]
#[non_exhaustive]
pub struct Game {
    pub path: PathBuf,
    pub common: PathBuf,
    pub countries: GameCountry,
    pub religions: BTreeMap<Str, Religion>,
    pub cultures: BTreeMap<Str, Culture>
}

impl Game {
    #[inline]
    pub async fn new<P: IntoPathBuf> (path: P) -> Result<Self> {
        let path = path.into_path_buf();
        let common = path.join("common");

        let (countries, religions, cultures) = futures::try_join! {
            GameCountry::from_common(&common),
            Religion::from_common(&common).and_then(TryStreamExt::try_collect::<BTreeMap<_, _>>),
            Culture::from_common(&common).and_then(TryStreamExt::try_collect::<BTreeMap<_, _>>)
        }?;

        return Ok(Self { path, common, countries, religions, cultures })
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