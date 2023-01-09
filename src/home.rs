use std::{path::{Path}, mem::MaybeUninit};
use eframe::{egui::{CentralPanel, SidePanel, Window, RichText, Color32}};
use rfd::FileDialog;
use crate::{*, data::Game};

#[derive(Debug)]
#[non_exhaustive]
pub struct Home {
    init_game_path: bool,
    pub game_path: String,
    show_error: bool,
    error_message: MaybeUninit<String>
}

impl Default for Home {
    #[inline]
    fn default() -> Self {
        Self {
            init_game_path: true,
            game_path: String::new(),
            show_error: false,
            error_message: MaybeUninit::uninit()
        }
    }
}

impl App for Home {
    fn update(&mut self, ctx: &eframe::egui::Context, frame: &mut eframe::Frame) {    
        self.init_game_path(ctx, frame);
        Window::new(RichText::new("Error").background_color(Color32::DARK_RED)).open(&mut self.show_error).show(ctx, |ui| unsafe {
            ui.label(self.error_message.assume_init_ref());
        });

        CentralPanel::default().show(ctx, |ui| {
            SidePanel::left("my_left_panel").show(ctx, |ui| {
                ui.label("Hello World!");
            });

            let game_path = ui.button("Select game data path");
            ui.text_edit_singleline(&mut self.game_path);
            let open_game_data = ui.button("Open game data");

            if game_path.clicked() {
                self.select_game_path();
                return
            }

            match (open_game_data.clicked(), &self.game_path) {
                (true, x) if x.is_empty() => {
                    self.error_message.write("No game path specified".to_string());
                    self.show_error = true;
                    ctx.request_repaint();
                },

                (true, path) => {
                    let game = match runtime().block_on(Game::new(path as &String)) {
                        Ok(game) => {
                            if let Some(storage) = frame.storage_mut() {
                                storage.set_string("game_path", path.clone());
                                storage.flush();
                            }
                            game
                        },

                        Err(e) => {
                            self.error_message.write(e.to_string());
                            self.show_error = true;
                            ctx.request_repaint();
                            return
                        }
                    };

                    GAME.set(Some(game));
                    frame.close();
                    return
                },
                
                _ => {}
            }
        });
    }
}

impl Home {
    #[inline]
    fn init_game_path (&mut self, ctx: &eframe::egui::Context, frame: &eframe::Frame) {
        if self.init_game_path {
            self.game_path = frame.storage()
                .and_then(|stg| stg.get_string("game_path"))
                .unwrap_or_default();
            self.init_game_path = false;
            ctx.request_repaint()
        }
    }

    #[inline]
    fn select_game_path (&mut self) {
        let mut builder = FileDialog::new();
        if !self.game_path.is_empty() {
            let path = <String as AsRef<Path>>::as_ref(&self.game_path);
            if path.is_dir() {
                builder = builder.set_directory(path);
            }
        }

        match builder.pick_folder().map(|x| x.into_os_string().into_string()) {
            Some(Ok(x)) => self.game_path = x,
            _ => {}
        }
    }
}
