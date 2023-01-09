use std::{collections::{BTreeMap}, fmt::Debug, pin::Pin, rc::Rc};
use eframe::{epaint::{Color32}, egui::{SidePanel, ScrollArea, RichText, Ui, Id, Label, Link, Sense}};
use crate::{Str, data::Game};

pub trait ListEntry {
    fn color (&self) -> Option<Color32>;
    fn render_info (&self, ui: &mut Ui);
}

pub struct List<T, F> {
    list_id: Id,
    game: Pin<Rc<Game>>,
    items: F,
    current: Option<(*const str, *const T)>
}

impl<T: Debug + ListEntry, F: for<'a> Fn(&'a Game) -> &'a BTreeMap<Str, T>> List<T, F> {
    #[inline]
    pub fn new (id: &str, game: Pin<Rc<Game>>, f: F) -> Self {
        return Self {
            list_id: format!("{id}_list").into(),
            game,
            items: f,
            current: None
        }
    }
    
    #[inline]
    pub fn update (&mut self, ui: &mut Ui) {
        SidePanel::left(self.list_id).show_inside(ui, |ui| {
            ScrollArea::vertical().show(ui, |ui| {
                for (name, info) in (self.items)(&self.game).iter() {
                    let mut text = RichText::new(name.to_string());
                    if let Some(color) = info.color() {
                        text = text.color(color);
                    }

                    if ui.add(Label::new(text).sense(Sense::click())).clicked() {
                        self.current = Some((name as &str, info))
                    }
                }
            });
        });

        if let Some((name, info)) = self.current {
            let name = unsafe { &*name };
            let info = unsafe { &*info };

            let mut text = RichText::new(name.to_string());
            if let Some(color) = info.color() {
                text = text.color(color);
            }
            
            ui.heading(text);
            info.render_info(ui);
        }
    }
}