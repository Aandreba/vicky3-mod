use std::{collections::{BTreeMap}, fmt::Debug};
use eframe::{epaint::{Color32}, egui::{SidePanel, ScrollArea, RichText, Ui, Id, Label, Sense, TextStyle}};
use crate::data::Game;
use super::refcell::RefCell;

pub trait ListEntry {
    fn color (&self) -> Option<Color32>;
    fn render_info (&mut self, ui: &mut Ui, game: &Game);
}

pub struct List<'this, T> {
    list_id: Id,
    items: &'this RefCell<BTreeMap<String, T>>,
    current: Option<String>
}

impl<'this, T: Debug + ListEntry> List<'this, T> {
    #[inline]
    pub fn new (id: &str, items: &'this RefCell<BTreeMap<String, T>>) -> Self {
        return Self {
            list_id: format!("{id}_list").into(),
            items,
            current: None
        }
    }

    #[inline]
    pub fn update (&mut self, ui: &mut Ui, game: &Game) {
        let mut items = self.items.borrow_mut();

        SidePanel::left(self.list_id).show_inside(ui, |ui| {
            let height = ui.text_style_height(&TextStyle::Body);
            ScrollArea::vertical().show_rows(ui, height, items.len(), |ui, _range| {
                for (name, info) in items.iter_mut() {
                    let mut text = RichText::new(name.to_string());
                    if let Some(color) = info.color() {
                        text = text.color(color);
                    }

                    if ui.add(Label::new(text).sense(Sense::click())).clicked() {
                        self.current = Some(name.clone())
                    }
                }
            });
        });

        ui.vertical_centered(|ui| {
            if let Some((info, name)) = self.current.as_ref().and_then(|key| Some((items.get_mut(key)?, key))) {
                let name = &*name;
                let info = &mut *info;
    
                let mut text = RichText::new(name.to_string());
                if let Some(color) = info.color() {
                    text = text.color(color);
                }
                
                ui.heading(text);
                info.render_info(ui, game);
            }
        });

    }
}