use std::{rc::Rc, collections::BTreeMap, pin::Pin};
use eframe::egui::*;
use named_fn::named_fn;
use crate::{Main, data::{Game, religion::Religion, culture::Culture}, utils::list::List, Str};

#[named_fn]
fn get_religions<'a> (game: &'a Game) -> &'a BTreeMap<Str, Religion> {
    return &game.religions
}

#[named_fn]
fn get_cultures<'a> (game: &'a Game) -> &'a BTreeMap<Str, Culture> {
    return &game.cultures
}

pub struct ModFolder {
    game: Pin<Rc<Game>>,
    writeable: bool,
    religions: List<Religion, GetReligions>,
    cultures: List<Culture, GetCultures>,
    show_cultures: bool,
    show_religions: bool,
}

impl ModFolder {
    #[inline]
    pub fn open (game: Game) -> Self {
        let game = Rc::pin(game);
        return Self {
            game: game.clone(),
            religions: List::new("religions", game.clone(), GetReligions::new()),
            cultures: List::new("cultures", game, GetCultures::new()),
            writeable: false,
            show_cultures: false,
            show_religions: false
        }
    }

    #[inline]
    pub fn update (&mut self, ctx: &eframe::egui::Context, _frame: &mut eframe::Frame) -> Option<Main> {
        Window::new("Religions").open(&mut self.show_religions).show(ctx, |ui| {
            self.religions.update(ui);
        });

        Window::new("Cultures").open(&mut self.show_cultures).show(ctx, |ui| {
            self.cultures.update(ui);
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