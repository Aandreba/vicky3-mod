use std::{collections::{BTreeMap}, fmt::Debug};
use eframe::{epaint::{Color32}, egui::{SidePanel, ScrollArea, RichText, Label, Ui, Id, Button}};
use crate::{Str};

pub trait ListEntry {
    fn color (&self) -> Option<Color32>;
    fn render_info (&self, ui: &mut Ui);
}

pub struct List<'this, 'a: 'this, T> {
    list_id: Id,
    items: &'a BTreeMap<Str, T>,
    current: Option<(&'this str, &'this T)>
}

impl<'this, 'a, T: Debug + ListEntry> List<'this, 'a, T> {
    #[inline]
    pub fn new (id: &str, items: &'a BTreeMap<Str, T>) -> Self {
        return Self {
            list_id: format!("{id}_list").into(),
            items,
            current: None
        }
    }
    
    #[inline]
    pub fn update (&'this mut self, ui: &mut Ui) {
        SidePanel::left(self.list_id).show_inside(ui, |ui| {
            ScrollArea::vertical().show(ui, |ui| {
                for (name, info) in self.items.iter() {
                    let mut text = RichText::new(name.to_string());
                    if let Some(color) = info.color() {
                        text = text.color(color);
                    }

                    if ui.add(Button::new(text)).clicked() {
                        self.current = Some((name, info))
                    }
                }
            });
        });

        if let Some((name, info)) = self.current {
            let mut text = RichText::new(name.to_string());
            if let Some(color) = info.color() {
                text = text.color(color);
            }
            
            ui.heading(text);
            info.render_info(ui);
        }
    }
}