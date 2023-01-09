use std::{collections::BTreeMap, pin::Pin, borrow};
use eframe::{egui::*, App};
use named_fn::named_fn;
use sis::self_referencing;
use crate::{data::{Game, religion::{Religion}, culture::Culture, country::{CountryRank, CountryType}}, utils::list::List};

#[named_fn]
fn get_religions<'a> (game: &'a Game) -> &'a BTreeMap<String, Religion> {
    return &game.religions
}

#[named_fn]
fn get_cultures<'a> (game: &'a Game) -> &'a BTreeMap<String, Culture> {
    return &game.cultures
}

pub struct ModFolderLists<'this> {
    religions: List<'this, Religion>,
    cultures: List<'this, Culture>,
    country_ranks: List<'this, CountryRank>,
    country_types: List<'this, CountryType>
}

impl<'this> ModFolderLists<'this> {
    #[inline]
    pub fn new (game: Pin<&'this mut Game>) -> Self {
        let game = Pin::into_inner(game);
        return Self {
            religions: List::new("Religions", &mut game.religions),
            cultures: List::new("Cultures", &mut game.cultures),
            country_ranks: List::new("Country Ranks", &mut game.countries.ranks),
            country_types: List::new("Country Types", &mut game.countries.tys),
        }
    }
}

#[self_referencing(extern)]
pub struct ModFolder {
    game: Game,
    writeable: bool,
    show_cultures: bool,
    show_religions: bool,
    show_country_ranks: bool,
    show_country_types: bool,
    #[borrows(mut game)]
    lists: ModFolderLists<'this>
}

impl<'this> App for ModFolder<'this> {
    #[inline]
    fn update(&mut self, ctx: &eframe::egui::Context, frame: &mut eframe::Frame) {
        let game = unsafe { Pin::new_unchecked(&mut self.game) };
        let ModFolderLists { religions, cultures, country_ranks, country_types } = unsafe { self.lists.assume_init_mut() };
        let _pin = unsafe { Pin::new_unchecked(&mut self._pin) };

        // Misc
        Window::new("Religions").open(&mut self.show_religions).show(ctx, |ui| {
            religions.update(ui);
        });
        Window::new("Cultures").open(&mut self.show_cultures).show(ctx, |ui| {
            cultures.update(ui);
        });

        // Country
        Window::new("Country Ranks").open(&mut self.show_country_ranks).show(ctx, |ui| {
            country_ranks.update(ui);
        });
        Window::new("Country Types").open(&mut self.show_country_types).show(ctx, |ui| {
            country_types.update(ui);
        });
        
        CentralPanel::default().show(ctx, |ui| {
            ui.horizontal(|ui| {
                // Misc
                ui.vertical(|ui| {
                    let cultures = ui.button(
                        format!("Cultures ({})", game.cultures.len())
                    );
                    
                    let religions = ui.button(
                        format!("Religions ({})", game.religions.len())
                    );
        
                    self.show_cultures ^= cultures.clicked();
                    self.show_religions ^= religions.clicked();
                });

                // Country info
                ui.vertical(|ui| {
                    let ranks = ui.button(
                        format!("Country Ranks ({})", game.countries.ranks.len())
                    );
                    
                    let tys = ui.button(
                        format!("Country Types ({})", game.countries.tys.len())
                    );
        
                    self.show_country_ranks ^= ranks.clicked();
                    self.show_country_types ^= tys.clicked();
                });
            });
        });
    }
}