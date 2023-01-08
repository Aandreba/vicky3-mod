use eframe::egui::*;
use crate::{Main, data::Game};

pub struct ModFolder {
    game: Game
}

impl ModFolder {
    #[inline]
    pub fn open (game: Game) -> Self {
        return Self { game }
    }

    #[inline]
    pub fn update(&mut self, ctx: &eframe::egui::Context, frame: &mut eframe::Frame) -> Option<Main> {
        return CentralPanel::default().show(ctx, |ui| {
            None
            // todo!()
        }).inner
    }
}