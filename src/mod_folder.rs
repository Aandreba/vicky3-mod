use eframe::egui::*;
use crate::{Main, data::{Game}, utils::list::List};

pub struct ModFolder {
    game: Game,
    show_cultures: bool,
    show_religions: bool,
}

impl ModFolder {
    #[inline]
    pub fn open (game: Game) -> Self {
        return Self {
            game,
            show_cultures: false,
            show_religions: false,
        }
    }

    #[inline]
    pub fn update(&mut self, ctx: &eframe::egui::Context, _frame: &mut eframe::Frame) -> Option<Main> {
        let mut religions = List::new("religions", &self.game.religions);
        Window::new("religions").open(&mut self.show_religions).show(ctx, |ui| {
            religions.update(ui)
        });

        let mut cultures = List::new("cultures", &self.game.cultures);
        Window::new("cultures").open(&mut self.show_cultures).show(ctx, |ui| {
            cultures.update(ui)
        });
        
        return CentralPanel::default().show(ctx, |ui| {
            let cultures = ui.button(
                format!("Cultures ({})", self.game.cultures.len())
            );
            
            let religions = ui.button(
                format!("Religions ({})", self.game.religions.len())
            );

            self.show_cultures ^= cultures.clicked();
            self.show_religions ^= religions.clicked();
            
            None
            // todo!()
        }).inner
    }
}