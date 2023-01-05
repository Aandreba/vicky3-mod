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
pub(crate) mod utils;

flat_mod! { color }

use std::{path::{PathBuf, Path}, pin::Pin};
use country::CountryGame;
use itertools::Itertools;

#[cfg(debug_assertions)]
static mut GAME: Option<Game> = None;
#[cfg(not(debug_assertions))]
static mut GAME: MaybeUninit<Game> = MaybeUninit::uninit();

pin_project_lite::pin_project! {
    #[derive(Debug)]
    pub struct Game {
        _path: &'static Path,
        common: PathBuf,
        #[pin]
        country: CountryGame<'static>
    }
}

impl Game {
    #[inline]
    pub unsafe fn initialize<T: ?Sized + AsRef<Path>> (path: &'static T) {
        let path = path.as_ref();
        let common = path.join("common");
        let country = unsafe { CountryGame::new_uninit(&common).unwrap() };
        let game = Game {
            _path: path,
            common,
            country
        };

        let this;
        unsafe {
            cfg_if::cfg_if! {
                if #[cfg(debug_assertions)] {
                    GAME = Some(game);
                    this = Pin::new_unchecked(GAME.as_mut().unwrap_unchecked()).project();
                } else {
                    GAME.write(game);
                    this = Pin::new_unchecked(GAME.assume_init_mut()).project();
                }
            }
        }

        this.country.initialize_with_common(this.common).unwrap();
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
    pub fn country () -> &'static CountryGame<'static> {
        return &Self::get().country
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