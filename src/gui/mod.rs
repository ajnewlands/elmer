use std::rc::Rc;

use eframe::egui::{
    self, Button, CentralPanel, Color32, CornerRadius, FontFamily, FontId, Frame, Grid, Id, Label,
    Layout, Response, RichText, ScrollArea, Sense, Spacing, TextEdit, TextStyle, TopBottomPanel,
    Ui, Vec2, WidgetText,
};
use egui_phosphor::regular as icon;
use lapin::{options::QueueBindOptions, types::FieldTable};
use model::ModelItem;
use state::ConnectionStatus;

use crate::rabbit::{ConnectionManager, ConnectionUpdate};
pub mod connection_modal;
mod model;
mod state;

mod enums;
mod prelude;

pub struct App {
    gui_state: state::GuiState,
    gui_data: model::Model,
    connection_manager: ConnectionManager,
}

impl App {
    pub fn new(_cc: &eframe::CreationContext<'_>, connection_manager: ConnectionManager) -> Self {
        Self {
            gui_state: state::GuiState::default(),
            gui_data: model::Model::default(),
            connection_manager,
        }
    }

    fn menu_bar(&mut self, ctx: &egui::Context) {
        egui::TopBottomPanel::top("menubar_container").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                if ui
                    .button(
                        RichText::new(icon::NETWORK)
                            .size(24.0)
                            .color(Color32::LIGHT_BLUE),
                    )
                    .on_hover_text("Connection settings")
                    .clicked()
                {
                    self.gui_state.connection_modal_state =
                        Some(self.gui_state.connection_state.clone());
                }
                ui.button(
                    RichText::new(icon::PLAY)
                        .size(24.0)
                        .color(Color32::LIGHT_GREEN),
                )
                .on_hover_text("Connect");
                ui.button(RichText::new(icon::EMPTY).size(24.0).color(Color32::WHITE))
                    .on_hover_text("Clear data");

                // Add a text box that fills the remaining space
                let available_width = ui.available_width() - 32.0; // Leave space for menu icon and padding between elements
                if ui
                    .add_sized(
                        [available_width, 24.0],
                        egui::TextEdit::singleline(&mut self.gui_state.filter_string)
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

                ui.menu_button(RichText::new(icon::LIST).size(24.0), |ui| {
                    ui.checkbox(
                        &mut self.gui_state.filter_headers,
                        RichText::new("Filter headers").size(16.0),
                    );
                    ui.checkbox(
                        &mut self.gui_state.filter_body,
                        RichText::new("Filter body").size(16.0),
                    );
                });
            });
        });
    }

    fn tree_data_view(&mut self, ctx: &egui::Context) {
        CentralPanel::default().show(ctx, |ui| {
            // TODO utilize show_viewport instead to render only visible section
            ScrollArea::vertical().auto_shrink(false).show(ui, |ui| {
                Grid::new("data").min_col_width(0.0).show(ui, |ui| {
                    for item in &mut self.gui_data.data {
                        // Omit rows that should be filtered according to the current regex.
                        if self.gui_state.regex.is_some() && item.highlights.is_empty() {
                            continue;
                        }
                        if ui
                            .add(
                                egui::Button::new(RichText::new(icon::CARET_RIGHT))
                                    .fill(Color32::TRANSPARENT),
                            )
                            .clicked()
                        {
                            item.expanded = !item.expanded;
                        }
                        ui.label("timestamp");
                        ui.horizontal(|ui| highlight_text(ui, &item.headers, &item.highlights));
                        //ui.label(item.headers.clone());
                        ui.end_row();

                        if item.expanded {
                            ui.label("");
                            ui.label("");
                            ui.label(&*item.body);
                            ui.end_row();
                        }
                    }
                });
            });
        });
    }

    fn status_bar(&self, ctx: &egui::Context) {
        TopBottomPanel::bottom("status bar").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                let funnel_colour: Color32;
                let funnel_tooltip: String;
                if self.gui_state.filter_string.is_empty() {
                    funnel_colour = Color32::GRAY;
                    funnel_tooltip = "No filter regex".into();
                } else if self.gui_state.regex_error.is_some() {
                    funnel_colour = Color32::RED;
                    funnel_tooltip = self.gui_state.regex_error.clone().unwrap();
                } else {
                    funnel_colour = Color32::GREEN;
                    funnel_tooltip = "Filtering with valid regex".into()
                }

                let connection_state_message = match self.gui_state.connection {
                    ConnectionStatus::Disconnected => "Not connected".into(),
                    ConnectionStatus::Connecting => {
                        format!("Connecting to {}", self.gui_state.connection_state.hostname)
                    }
                    ConnectionStatus::Connected => {
                        format!("Connected to {}", self.gui_state.connection_state.hostname)
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

fn highlight_text(ui: &mut Ui, text: &str, highlights: &[model::Highlight]) {
    let mut index = 0;

    let style = ui.style_mut();
    style.spacing.item_spacing = Vec2::new(0.0, 0.0);

    for highlight in highlights {
        // Add normal text before the highlight
        if index < highlight.start {
            ui.label(&text[index..highlight.start]);
        }

        // Add highlighted text
        ui.label(
            RichText::new(&text[highlight.start..highlight.end])
                .background_color(Color32::LIGHT_BLUE)
                .color(Color32::BLACK),
        );

        index = highlight.end;
    }

    // Add remaining text after the last highlight
    if index < text.len() {
        ui.label(&text[index..]);
    }
}

impl App {
    fn change_connection_state(&mut self, new_state: ConnectionStatus) {
        self.gui_state.connection = new_state;
        if self.gui_state.connection == ConnectionStatus::Connected
            && self.gui_state.connection_state.wildcard
        {
            // TODO
            let _ = self
                .connection_manager
                .tx
                .send(crate::rabbit::ConnectionCommand::Bind {
                    exchange: "2steps".into(),
                    routing_key: "".into(),
                    options: QueueBindOptions::default(),
                    arguments: FieldTable::default(),
                });
        }
    }
    fn process_connection_update(&mut self, update: ConnectionUpdate) {
        match update {
            ConnectionUpdate::Connected => {
                self.change_connection_state(ConnectionStatus::Connected)
            }
            ConnectionUpdate::Disconnected => {
                self.change_connection_state(ConnectionStatus::Disconnected)
            }
            ConnectionUpdate::Connecting => {
                self.change_connection_state(ConnectionStatus::Connected)
            }
            ConnectionUpdate::TextDelivery {
                headers,
                content,
                content_type,
            } => {
                let mut item = ModelItem {
                    headers: Rc::new(headers),
                    body: Rc::new(content),
                    expanded: false,
                    highlights: Vec::default(),
                };
                item.apply_filter(&self.gui_state);
                // TODO check if max length is hit, in which case we must pop from the front first.
                self.gui_data.data.push_back(item);
            }
            ConnectionUpdate::BinaryDelivery {
                headers,
                content_type,
            } => {
                let mut item = ModelItem {
                    headers: Rc::new(headers),
                    body: Rc::new("-Binary data-".into()),
                    expanded: false,
                    highlights: Vec::default(),
                };
                item.apply_filter(&self.gui_state);
                // TODO check if max length is hit, in which case we must pop from the front first.
                self.gui_data.data.push_back(item);
            }
        }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        while let Some(update) = self.connection_manager.rx.try_recv().ok() {
            self.process_connection_update(update);
        }

        egui::CentralPanel::default()
            .frame(egui::Frame::NONE)
            .show(ctx, |ui| {
                ctx.set_visuals(egui::Visuals {
                    menu_corner_radius: CornerRadius::ZERO,
                    override_text_color: Some(Color32::WHITE),
                    ..egui::Visuals::dark()
                });
                self.show_connection_modal(ui, ctx);

                self.menu_bar(ctx);
                self.status_bar(ctx);

                // Tree data grid body
                self.tree_data_view(ctx);
            });
    }
}
