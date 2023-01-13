use eframe::egui::Ui;
use crate::data::{Game};

pub struct States<'a> {
    game: &'a Game
}

impl<'a> States<'a> {
    #[inline]
    pub fn new (game: &'a Game) -> Self {
        return Self { game }
    }

    #[inline]
    pub fn update (&mut self, ui: &mut Ui) {

    }
}