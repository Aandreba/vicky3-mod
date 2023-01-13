use std::{pin::Pin};
use eframe::{egui::*, App};
use sis::self_referencing;
use crate::{data::{Game, religion::{Religion}, culture::Culture, country::{CountryRank, CountryType}}, utils::list::List, states::States};

pub struct ModFolderLists<'this> {
    religions: List<'this, Religion>,
    cultures: List<'this, Culture>,
    country_ranks: List<'this, CountryRank>,
    country_types: List<'this, CountryType>,
    states: States<'this>
}

impl<'this> ModFolderLists<'this> {
    #[inline]
    pub fn new (game: &'this Game) -> Self {
        return Self {
            religions: List::new("Religions", &game.religions),
            cultures: List::new("Cultures", &game.cultures),
            country_ranks: List::new("Country Ranks", &game.countries.ranks),
            country_types: List::new("Country Types", &game.countries.tys),
            states: States::new(game)
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
    show_states: bool,
    #[borrows(game)]
    lists: ModFolderLists<'this>
}

impl<'this> App for ModFolder<'this> {
    #[inline]
    fn update(&mut self, ctx: &eframe::egui::Context, _frame: &mut eframe::Frame) {
        let game = unsafe { Pin::new_unchecked(&mut self.game) };
        let ModFolderLists { religions, cultures, country_ranks, country_types, states } = unsafe { self.lists.assume_init_mut() };
        let _pin = unsafe { Pin::new_unchecked(&mut self._pin) };

        // Misc
        Window::new("Religions")
            .open(&mut self.show_religions)
            .vscroll(true)
            .show(ctx, |ui| {
                religions.update(ui, &game);
            });
        Window::new("Cultures")
            .open(&mut self.show_cultures)
            .vscroll(true)
            .show(ctx, |ui| {
                cultures.update(ui, &game);
            });

        // Country
        Window::new("Country Ranks")
            .open(&mut self.show_country_ranks)
            .vscroll(true)
            .show(ctx, |ui| {
                country_ranks.update(ui, &game);
            });
        Window::new("Country Types")
            .open(&mut self.show_country_types)
            .vscroll(true)
            .show(ctx, |ui| {
                country_types.update(ui, &game);
            });

        // States
        Window::new("States")
            .open(&mut self.show_states)
            .vscroll(true)
            .show(ctx, |ui| {
                states.update(ui);
            });
        
        CentralPanel::default().show(ctx, |ui| {
            ui.horizontal(|ui| {
                // Misc
                ui.vertical(|ui| {
                    let cultures = ui.button(
                        format!("Cultures ({})", game.cultures.borrow().len())
                    );
                    
                    let religions = ui.button(
                        format!("Religions ({})", game.religions.borrow().len())
                    );
        
                    self.show_cultures ^= cultures.clicked();
                    self.show_religions ^= religions.clicked();
                });

                // Country info
                ui.vertical(|ui| {
                    let ranks = ui.button(
                        format!("Country Ranks ({})", game.countries.ranks.borrow().len())
                    );
                    
                    let tys = ui.button(
                        format!("Country Types ({})", game.countries.tys.borrow().len())
                    );
        
                    self.show_country_ranks ^= ranks.clicked();
                    self.show_country_types ^= tys.clicked();
                });
            });
        });
    }
}