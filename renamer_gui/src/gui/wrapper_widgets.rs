use std::{fmt::Display, path::PathBuf};

use egui::{Response, Ui};
use renamer_lib::Action;

pub struct ActionWidget<'a>(&'a mut Action);

impl<'a> From<&'a mut Action> for ActionWidget<'a> {
    fn from(value: &'a mut Action) -> Self {
        Self(value)
    }
}

impl egui::Widget for ActionWidget<'_> {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        ui.label(self.to_string())
    }
}

impl Display for ActionWidget<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.0 {
            Action::Randomize => write!(f, "Randomize"),
            Action::Rename(renaming_pattern) => write!(f, "Rename: {:?}", renaming_pattern),
        }
    }
}

pub struct FileWidget<'a>(&'a mut PathBuf);

impl<'a> From<&'a mut PathBuf> for FileWidget<'a> {
    fn from(value: &'a mut PathBuf) -> Self {
        Self(value)
    }
}

impl egui::Widget for FileWidget<'_> {
    fn ui(self, ui: &mut egui::Ui) -> Response {
        ui.label(self.0.to_string_lossy())
    }
}
