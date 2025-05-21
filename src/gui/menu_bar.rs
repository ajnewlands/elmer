use crate::gui::state::ConnectionStatus;
use eframe::egui::{self, Button, Color32, FontFamily, FontId, RichText};
use egui::Ui;
use egui_phosphor::regular as icon;

impl super::App {
    /// Open the connection settings window e.g. to connect to a different
    /// bus
    fn connection_settings_button(&mut self, ui: &mut Ui) {
        let icon = icon::NETWORK;
        if self.gui_state.connection == ConnectionStatus::Disconnected {
            if ui
                .button(RichText::new(icon).size(24.0).color(Color32::LIGHT_BLUE))
                .on_hover_text("Connection settings")
                .clicked()
            {
                self.gui_state.connection_modal_parameters =
                    Some(self.gui_state.connection_parameters.clone());
            }
        } else {
            ui.add_enabled(false, Button::new(RichText::new(icon).size(24.0)));
        }
    }

    /// The connect/disconnect button (depending on current status)
    fn connect_button(&mut self, ui: &mut Ui) {
        // The CONNECT/DISCONNECT button
        let (icon, colour, hover_text) = match self.gui_state.connection {
            ConnectionStatus::Disconnected => (icon::PLAY, Color32::LIGHT_GREEN, "Connect"),
            _ => (icon::STOP, Color32::RED, "Disconnect"),
        };
        if ui
            .button(RichText::new(icon).size(24.0).color(colour))
            .on_hover_text(hover_text)
            .clicked()
        {
            let command = if self.gui_state.connection == ConnectionStatus::Disconnected {
                crate::rabbit::ConnectionCommand::Connect(
                    self.gui_state.connection_parameters.build_url(),
                    ui.ctx().to_owned(),
                )
            } else {
                crate::rabbit::ConnectionCommand::Disconnect
            };
            // TODO error surfacing
            self.connection_manager
                .tx
                .send(command)
                .expect("Internal channel closed")
        }
    }

    /// Construct the clear data button, which when clicked will purge any currently accumulated
    /// data from the tree view.
    fn clear_button(&mut self, ui: &mut Ui) {
        if ui
            .button(RichText::new(icon::EMPTY).size(24.0).color(Color32::WHITE))
            .on_hover_text("Clear data")
            .clicked()
        {
            self.gui_data.data.clear();
        }
    }

    fn regex_entry(&mut self, ui: &mut Ui) {
        // Add a text box that fills the remaining space
        let available_width = ui.available_width() - 32.0; // Leave space for menu icon and padding between elements
        if ui
            .add_sized(
                [available_width, 24.0],
                egui::TextEdit::singleline(&mut self.gui_state.filter_state.filter_string)
                    .hint_text("filter regex")
                    .vertical_align(egui::Align::Center)
                    .font(FontId {
                        size: 16.0,
                        family: FontFamily::Proportional,
                    }),
            )
            .changed()
        {
            self.gui_state.update_regex();
            self.gui_data.filter_all(&self.gui_state);
        }
    }

    fn menu_button(&mut self, ui: &mut Ui) {
        ui.menu_button(RichText::new(icon::LIST).size(24.0), |ui| {
            ui.checkbox(
                &mut self.gui_state.filter_state.filter_headers,
                RichText::new("Filter headers").size(16.0),
            );
            ui.checkbox(
                &mut self.gui_state.filter_state.filter_body,
                RichText::new("Filter body").size(16.0),
            );
        });
    }

    /// Construct the toolbar across the top of the main window.
    pub(crate) fn menu_bar(&mut self, ctx: &egui::Context) {
        egui::TopBottomPanel::top("menubar_container").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                self.connection_settings_button(ui);
                self.connect_button(ui);
                self.clear_button(ui);
                self.regex_entry(ui);
                self.menu_button(ui);
            });
        });
    }
}
