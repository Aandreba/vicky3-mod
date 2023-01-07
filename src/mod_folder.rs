use std::path::PathBuf;
use eframe::egui::*;
use crate::Main;

pub struct ModFolder {
    path: PathBuf,
    write: bool    
}

impl ModFolder {
    #[inline]
    pub fn open (path: PathBuf) -> Self {
        return Self {
            path,
            write: false,
        }
    }

    #[inline]
    pub fn update(&mut self, ctx: &eframe::egui::Context, frame: &mut eframe::Frame) -> Option<Main> {
        return CentralPanel::default().show(ctx, |ui| {
            None
            // todo!()
        }).inner
    }
}