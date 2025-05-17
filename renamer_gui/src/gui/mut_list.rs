use std::{ops::Deref, path::PathBuf};

use egui::{Response, Ui, Widget};

use std::fmt::Display;

use renamer_lib::Action;
pub struct MutList<'a, T>(&'a mut Vec<T>);

impl<'a, T> From<&'a mut Vec<T>> for MutList<'a, T> {
    fn from(value: &'a mut Vec<T>) -> Self {
        Self(value)
    }
}

pub struct MutListResponse {
    add: bool,
    inner: egui::Response,
    to_remove: Vec<usize>,
}

impl MutListResponse {
    pub fn add_clicked(&self) -> bool {
        self.add
    }
    pub fn remove_clicked(&self) -> Option<&Vec<usize>> {
        if self.to_remove.is_empty() {
            None
        } else {
            Some(&self.to_remove)
        }
    }
    pub fn handle_removes<T>(&self, xs: &mut Vec<T>) {
        for idx in self.to_remove.iter() {
            xs.remove(*idx);
        }
    }
}
impl Deref for MutListResponse {
    type Target = egui::Response;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<'a, T> MutList<'a, T> {
    pub fn render<U>(self, ui: &'a mut egui::Ui) -> MutListResponse
    where
        U: Widget + From<&'a mut T>,
    {
        let mut add = false;
        let mut to_remove = Vec::new();
        let inner = ui.vertical_centered_justified(|ui| {
            for (i, item) in self.0.iter_mut().enumerate() {
                ui.horizontal(|ui| {
                    U::from(item).ui(ui);
                    if ui.button("Remove").clicked() {
                        to_remove.push(i);
                    }
                });
            }
            if ui.button("Add").clicked() {
                add = true;
            };
        });

        MutListResponse {
            add,
            to_remove,
            inner: inner.response,
        }
    }
}
