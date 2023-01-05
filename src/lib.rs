#![feature(fs_try_exists)]
#![cfg_attr(feature = "nightly", feature(iterator_try_collect))]

macro_rules! flat_mod {
    ($($i:ident),+) => {
        $(
            mod $i;
            pub use $i::*;
        )+
    };
}

pub(crate) type Str = Box<str>;
pub type Result<T> = ::core::result::Result<T, jomini::Error>;

pub mod country;
pub mod culture;
pub mod religion;
pub(crate) mod utils;

flat_mod! { color }

use std::{path::{Path}, pin::Pin, collections::HashMap, future::ready, marker::PhantomData};
use country::CountryGame;
use culture::Culture;
use futures::{Stream, TryStreamExt, TryFutureExt};
use itertools::Itertools;
use religion::Religion;
use sis::self_referencing;
use utils::FlattenOkIter;

#[cfg(debug_assertions)]
static mut GAME: Option<Game> = None;
#[cfg(not(debug_assertions))]
static mut GAME: MaybeUninit<Game> = MaybeUninit::uninit();

pin_project_lite::pin_project! {
    #[derive(Debug)]
    #[repr(transparent)]
    pub struct Game {
        #[pin]
        inner: GameInner<'static>
    }
}

impl Game {
    #[inline]
    pub async unsafe fn initialize<T: ?Sized + AsRef<Path>> (path: &'static T) {
        let path = path.as_ref();
        let common = Box::leak(path.join("common").into_boxed_path()) as &'static Path;

        let (countries, religions) = futures::try_join! {
            CountryGame::new_uninit(&common),
            TryFutureExt::and_then(Religion::from_common(&common), TryStreamExt::try_collect::<HashMap<_, _>>)
        }.unwrap();

        let game = GameInner::_new_uninit(
            path,
            common,
            religions
        );

        let this;
        unsafe {
            cfg_if::cfg_if! {
                if #[cfg(debug_assertions)] {
                    GAME = Some(Game { inner: game });
                    this = Pin::new_unchecked(GAME.as_mut().unwrap_unchecked()).project();
                } else {
                    GAME.write(Game { inner: game });
                    this = Pin::new_unchecked(GAME.assume_init_mut()).project();
                }
            }
        }

        this.inner._try_initialize_async(
            |religions| async move {
                return Culture::from_common(common, religions)
                    .await?
                    .try_collect()
                    .await
            },

            |_| ready(Ok(countries))
        ).await.unwrap();

        //this.0.country.initialize_with_common(this.common).await.unwrap();
    }

    #[inline]
    fn get () -> &'static Self {
        unsafe {
            cfg_if::cfg_if! {
                if #[cfg(debug_assertions)] {
                    return GAME.as_ref().unwrap()
                } else {
                    return GAME.assume_init_ref()
                }
            }
        }
    }

    #[inline]
    pub fn common () -> &'static Path {
        &Self::get().inner.common
    }

    #[inline]
    pub fn country () -> &'static CountryGame<'static> {
        return Self::get().inner.country()
    }

    #[inline]
    pub fn religions () -> &'static HashMap<Str, Religion> {
        return &Self::get().inner.religions
    }
}

#[self_referencing]
#[derive(Debug)]
struct GameInner {
    _path: &'this Path,
    common: &'this Path,
    religions: HashMap<Str, Religion>,
    countries: CountryGame<'this>,
    #[borrows(religions)]
    cultures: HashMap<Str, Culture<'this>>,
    #[borrows(mut countries, cultures)]
    _country_init: PhantomData<&'this mut ()>
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

#[cfg(feature = "nightly")]
pub(crate) fn try_collect<T, E, C, I> (iter: I) -> ::core::result::Result<C, E> where
    I: IntoIterator<Item = core::result::Result<T, E>>,
    C: FromIterator<T>
{
    return iter.try_collect::<C>();
}

#[cfg(not(feature = "nightly"))]
pub(crate) fn try_collect<T, E, C, I> (iter: I) -> ::core::result::Result<C, E> where
    I: IntoIterator<Item = core::result::Result<T, E>>,
    C: FromIterator<T>
{
    let mut iter = iter.into_iter();
    let chunk = (&mut iter)
        .take_while(|x| x.is_ok())
        .map(|x| unsafe { core::result::Result::<T, E>::unwrap_unchecked(x) })
        .collect::<C>();

    return match iter.next() {
        Some(Err(e)) => Err(e),
        #[cfg(debug_assertions)]
        Some(Ok(_)) => unreachable!(),
        None => Ok(chunk)
    }
}