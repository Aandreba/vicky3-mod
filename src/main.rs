#![feature(fn_traits, unboxed_closures, new_uninit, local_key_cell_methods)]

pub mod data;
pub mod home;
pub mod mod_folder;
pub(crate) mod utils;

pub type Result<T> = ::core::result::Result<T, jomini::Error>;

// /Users/Aandreba/Library/Application Support/Steam/steamapps/common/Victoria 3/game

use std::{cell::Cell, pin::Pin};
use data::Game;
use eframe::*;
use home::Home;
use tokio::runtime::Runtime;
use mod_folder::*;

thread_local! {
    pub static GAME: Cell<Option<Game>> = Cell::new(None);
}

cfg_if::cfg_if! {
    if #[cfg(debug_assertions)] {
        static mut RUNTIME: Option<Runtime> = None;
    } else {
        static mut RUNTIME: MaybeUninit<Runtime> = MaybeUninit::uninit();
    }
}

fn main() -> anyhow::Result<()> {
    // Initialize Tokio runtime
    let builder = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()?;
    unsafe { init_runtime(builder) }

    //unsafe { Game::initialize("D:/SteamLibrary/steamapps/common/Victoria 3/game").await };
    let options = eframe::NativeOptions {
        ..Default::default()
    };

    // Open folder selector (Home)
    let _ = eframe::run_native(
        "My Vicky3 Mod",
        options.clone(),
        Box::new(|_cc| Box::new(Home::default())),
    );

    // Open mod/game folder (ModFolder)
    if let Some(game) = GAME.take() {
        let app_name = game.path.to_string_lossy().into_owned();
        eframe::run_native(
            &app_name,
            options,
            Box::new(move |_cc| {
                new_mod_folder! {
                    { game, false, false, false, false, false },
                    { ModFolderLists::new },
                    box result
                }
                return unsafe { Pin::into_inner_unchecked(result) as Box<dyn App> }
            })
        );
    }

    return Ok(())
}

#[inline]
pub fn runtime () -> &'static Runtime {
    cfg_if::cfg_if! {
        if #[cfg(debug_assertions)] {
            return unsafe { RUNTIME.as_ref().unwrap() }
        } else {
            return unsafe { RUNTIME.assume_init_ref() }
        }
    }
}

#[inline]
unsafe fn init_runtime (runtime: Runtime) {
    cfg_if::cfg_if! {
        if #[cfg(debug_assertions)] {
            unsafe { RUNTIME = Some(runtime) }
        } else {
            unsafe { RUNTIME.write(runtime); }
        }
    }
}