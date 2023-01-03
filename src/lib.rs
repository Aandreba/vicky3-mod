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

flat_mod! { country, color }

use std::{path::{PathBuf, Path}};

use itertools::Itertools;
pub type Result<T> = ::core::result::Result<T, jomini::Error>;

#[cfg(debug_assertions)]
static mut GAME: Option<Game> = None;
#[cfg(not(debug_assertions))]
static mut GAME: MaybeUninit<Game> = MaybeUninit::uninit();

#[derive(Debug)]
pub struct Game {
    _path: &'static Path,
    common: PathBuf,
    country: CountryGame
}

impl Game {
    #[inline]
    pub unsafe fn initialize<T: ?Sized + AsRef<Path>> (path: &'static T) -> Result<&'static Game> {
        let path = path.as_ref();
        let common = path.join("common");
        let country = CountryGame::init(&common)?;

        let game = Game {
            _path: path,
            common,
            country
        };

        unsafe {
            cfg_if::cfg_if! {
                if #[cfg(debug_assertions)] {
                    GAME = Some(game);
                } else {
                    GAME.write(game);
                }
            }
        }

        return Ok(Self::get())
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
        &Self::get().common
    }

    #[inline]
    pub fn country () -> &'static CountryGame {
        return &Self::get().country
    }
}

#[inline]
pub(crate) fn flat_map_ok<T, E, I, F, U> (iter: I, f: F) -> impl Iterator<Item = ::core::result::Result<<U as IntoIterator>::Item, E>> where
    I: IntoIterator<Item = ::core::result::Result<T, E>>,
    F: FnMut(T) -> U,
    U: IntoIterator
{
    return itertools::Itertools::map_ok(iter.into_iter(), f).flatten_ok();
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