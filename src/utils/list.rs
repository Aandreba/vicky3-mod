use std::{collections::{BTreeMap}, fmt::Debug};
use eframe::{epaint::{Color32}, egui::{SidePanel, ScrollArea, RichText, Ui, Id, Label, Sense}};

pub trait ListEntry {
    fn color (&self) -> Option<Color32>;
    fn render_info (&mut self, ui: &mut Ui);
}

pub struct List<'this, T> {
    list_id: Id,
    items: &'this mut BTreeMap<String, T>,
    current: Option<(*const String, *mut T)>
}

impl<'this, T: Debug + ListEntry> List<'this, T> {
    #[inline]
    pub fn new (id: &str, items: &'this mut BTreeMap<String, T>) -> Self {
        return Self {
            list_id: format!("{id}_list").into(),
            items,
            current: None
        }
    }

    #[inline]
    pub fn items (&self) -> &BTreeMap<String, T> {
        return &self.items
    }
    
    #[inline]
    pub fn update (&mut self, ui: &mut Ui) {
        SidePanel::left(self.list_id).show_inside(ui, |ui| {
            ScrollArea::vertical().show(ui, |ui| {
                for (name, info) in self.items.iter_mut() {
                    let mut text = RichText::new(name.to_string());
                    if let Some(color) = info.color() {
                        text = text.color(color);
                    }

                    if ui.add(Label::new(text).sense(Sense::click())).clicked() {
                        self.current = Some((name as &String, info))
                    }
                }
            });
        });

        ui.vertical_centered(|ui| {
            if let Some((name, info)) = self.current {
                let name = unsafe { &*name };
                let info = unsafe { &mut *info };
    
                let mut text = RichText::new(name.to_string());
                if let Some(color) = info.color() {
                    text = text.color(color);
                }
                
                ui.heading(text);
                info.render_info(ui);
            }
        });

    }
}