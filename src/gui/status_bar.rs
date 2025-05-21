use crate::gui::state::ConnectionStatus;
use eframe::egui::{self, Color32, Label, RichText, TopBottomPanel};
use egui_phosphor::regular as icon;

impl super::App {
    pub(crate) fn status_bar(&self, ctx: &egui::Context) {
        TopBottomPanel::bottom("status bar").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                let funnel_colour: Color32;
                let funnel_tooltip: String;
                if self.gui_state.filter_state.filter_string.is_empty() {
                    funnel_colour = Color32::GRAY;
                    funnel_tooltip = "No filter regex".into();
                } else if self.gui_state.filter_state.regex_error.is_some() {
                    funnel_colour = Color32::RED;
                    funnel_tooltip = self.gui_state.filter_state.regex_error.clone().unwrap();
                } else {
                    funnel_colour = Color32::GREEN;
                    funnel_tooltip = "Filtering with valid regex".into()
                }

                let connection_state_message = match self.gui_state.connection {
                    ConnectionStatus::Disconnected => "Not connected".into(),
                    ConnectionStatus::Connecting => {
                        format!(
                            "Connecting to {}",
                            self.gui_state.connection_parameters.hostname
                        )
                    }
                    ConnectionStatus::Connected => {
                        format!(
                            "Connected to {}",
                            self.gui_state.connection_parameters.hostname
                        )
                    }
                };

                let available_width = ui.available_width() - 24.0; // Leave space for menu icon and padding between elements
                ui.add_sized(
                    [available_width, 24.0],
                    Label::new(connection_state_message),
                );
                ui.label(RichText::new(icon::FUNNEL).color(funnel_colour))
                    .on_hover_text(funnel_tooltip);
            })
        });
    }
}
