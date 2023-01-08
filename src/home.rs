use std::{path::{PathBuf, Path}, mem::MaybeUninit};
use eframe::{egui::{CentralPanel, SidePanel, Window}};
use rfd::FileDialog;
use crate::{Main, mod_folder::ModFolder, runtime, data::Game};

#[derive(Debug)]
#[non_exhaustive]
pub struct Home {
    pub game_path: Option<PathBuf>,
    show_error: bool,
    error_message: MaybeUninit<String>
}

impl Default for Home {
    #[inline]
    fn default() -> Self {
        Self {
            game_path: None,
            show_error: false,
            error_message: MaybeUninit::uninit()
        }
    }
}

impl Home {
    #[inline]
    pub fn update(&mut self, ctx: &eframe::egui::Context, _frame: &mut eframe::Frame) -> Option<Main> {
        Window::new("Error").open(&mut self.show_error).show(ctx, |ui| unsafe {
            ui.label(self.error_message.assume_init_ref());
        });

        return CentralPanel::default().show(ctx, |ui| {
            SidePanel::left("my_left_panel").show(ctx, |ui| {
                ui.label("Hello World!");
            });

            let game_path = ui.button("Select game data path");
            let open_game_data = ui.button("Open game data");

            if game_path.clicked() {
                self.select_game_path();
                return None
            }

            match (open_game_data.clicked(), &self.game_path) {
                (true, Some(path)) => {
                    let game = match runtime().block_on(Game::new(path)) {
                        Ok(x) => x,
                        Err(e) => {
                            self.error_message.write(e.to_string());
                            self.show_error = true;
                            ctx.request_repaint();
                            return None
                        }
                    };

                    return Some(Main::Mod(ModFolder::open(game)))
                },

                (true, None) => {
                    self.error_message.write("No game path specified".to_string());
                    self.show_error = true;
                    ctx.request_repaint();
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