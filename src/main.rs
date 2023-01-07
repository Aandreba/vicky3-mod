pub mod data;
pub mod home;
pub mod mod_folder;
pub(crate) mod utils;

pub(crate) type Str = Box<str>;
pub type Result<T> = ::core::result::Result<T, jomini::Error>;

use std::mem::MaybeUninit;

use eframe::*;
use home::Home;
use mod_folder::ModFolder;
use tokio::runtime::Runtime;

cfg_if::cfg_if! {
    if #[cfg(debug_assertions)] {
        static mut RUNTIME: Option<Runtime> = None;
    } else {
        static mut RUNTIME: MaybeUninit<Runtime> = MaybeUninit::uninit();
    }
}

pub enum Main {
    Home (Home),
    Mod (ModFolder)
}

impl Default for Main {
    #[inline]
    fn default() -> Self {
        Self::Home(Home::default())
    }
}

impl App for Main {
    #[inline]
    fn update(&mut self, ctx: &egui::Context, frame: &mut Frame) {
        let value = match self {
            Self::Home(home) => home.update(ctx, frame),
            Self::Mod(r#mod) => r#mod.update(ctx, frame)
        };

        if let Some(value) = value {
            *self = value;
            ctx.request_repaint();
        }
    }
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

fn main() -> anyhow::Result<()> {
    let builder = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap();

    unsafe { init_runtime(builder) }

    //unsafe { Game::initialize("D:/SteamLibrary/steamapps/common/Victoria 3/game").await };
    let options = eframe::NativeOptions {
        ..Default::default()
    };

    let _ = eframe::run_native(
        "My egui App",
        options,
        Box::new(|_cc| Box::new(Main::default())),
    );

    return Ok(())
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