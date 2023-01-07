use std::{path::{PathBuf, Path}, sync::atomic::AtomicBool};
use eframe::{egui::{CentralPanel, Window}};
use rfd::FileDialog;
use serde::{Serialize, Deserialize};
use tokio::runtime::Runtime;
use crate::{Main, mod_folder::ModFolder, runtime, data::Game};

#[derive(Debug, Serialize, Deserialize)]
#[non_exhaustive]
pub struct Home {
    pub game_path: Option<PathBuf>
}

impl Default for Home {
    #[inline]
    fn default() -> Self {
        Self {
            game_path: None
        }
    }
}

impl Home {
    #[inline]
    pub fn update(&mut self, ctx: &eframe::egui::Context, frame: &mut eframe::Frame) -> Option<Main> {
        return CentralPanel::default().show(ctx, |ui| {
            let game_path = ui.button("Select game data path");
            let open_game_data = ui.button("Open game data");

            if game_path.clicked() {
                self.select_game_path();
                return None
            }

            match (open_game_data.clicked(), &self.game_path) {
                (true, Some(path)) => unsafe {
                    runtime().block_on(Game::initialize(path));
                    return Some(Main::Mod(ModFolder::open(path.clone())))
                },

                (true, None) => {
                    // todo alert
                },
                
                _ => {}
            }

            return None
        }).inner;
    }

    #[inline]
    pub fn select_game_path (&mut self) {
        let mut builder = FileDialog::new();
        if let Some(path) = self.game_path.as_deref().and_then(Path::parent) {
            builder = builder.set_directory(path);
        }
        
        self.game_path = builder.pick_folder();
        
    }
}