pub mod mut_list;
mod popups;
pub mod state;
mod wrapper_widgets;
use crate::Renamer;

use eframe::App;
use egui::Widget;
use mut_list::MutList;
use renamer_lib::{Action, ActionGroup};
use wrapper_widgets::ActionWidget;
use wrapper_widgets::FileWidget;

impl App for Renamer {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.vertical_centered_justified(|ui| {
                ui.heading("Renamer");
                ui.horizontal_centered(|ui| {
                    {
                        let act_group_list =
                            MutList::from(&mut self.action_groups).render::<ActionGroupWidget>(ui);
                        if act_group_list.add_clicked() {}
                        act_group_list.handle_removes(&mut self.action_groups);
                    }
                    ui.button("Go")
                });
            })
        });
    }
}

struct ActionGroupWidget<'a>(&'a mut ActionGroup);
impl<'a> From<&'a mut ActionGroup> for ActionGroupWidget<'a> {
    fn from(value: &'a mut ActionGroup) -> Self {
        Self(value)
    }
}
impl egui::Widget for ActionGroupWidget<'_> {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        ui.horizontal_centered(|ui| {
            {
                let file_list = MutList::from(self.0.files()).render::<FileWidget>(ui);
                if file_list.add_clicked() {}
                file_list.handle_removes(self.0.files());
            }
            {
                let action_list = MutList::from(self.0.actions()).render::<ActionWidget>(ui);
                if action_list.add_clicked() {}
                action_list.handle_removes(self.0.actions());
            }
        })
        .response
    }
}
