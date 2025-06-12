use std::rc::Rc;

use eframe::egui::{self, Color32, CornerRadius};
use lapin::{options::QueueBindOptions, types::FieldTable};
use model::ModelItem;
use state::ConnectionStatus;

use crate::rabbit::{Binding, ConnectionManager, ConnectionUpdate};
mod add_subscription_window;
pub mod connection_modal;
mod menu_bar;
mod model;
mod state;
mod status_bar;
mod subscriptions_window;
mod tree_data_view;

mod enums;
mod prelude;

pub struct App {
    gui_state: state::GuiState,
    gui_data: model::Model,
    connection_manager: ConnectionManager,
    queue_bindings: Vec<Binding>,
}

impl App {
    pub fn new(_cc: &eframe::CreationContext<'_>, connection_manager: ConnectionManager) -> Self {
        Self {
            gui_state: state::GuiState::default(),
            gui_data: model::Model::default(),
            connection_manager,
            queue_bindings: Vec::default(),
        }
    }
}

impl App {
    fn change_connection_state(&mut self, new_state: ConnectionStatus) {
        self.gui_state.connection = new_state;
        if self.gui_state.connection == ConnectionStatus::Connected
            && self.gui_state.connection_parameters.wildcard
        {
            // TODO
            let _ = self
                .connection_manager
                .tx
                .send(crate::rabbit::ConnectionCommand::Bind {
                    exchange: self.gui_state.connection_parameters.exchange.clone().into(),
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
                self.change_connection_state(ConnectionStatus::Connecting)
            }
            ConnectionUpdate::Bound(binding) => {
                self.queue_bindings.push(binding);
            }
            ConnectionUpdate::Unbound(binding) => {
                self.queue_bindings.retain(|b| b.id != binding.id);
            }
            ConnectionUpdate::TextDelivery {
                headers,
                content,
                content_type,
            } => {
                let mut item = ModelItem {
                    timestamp: chrono::Local::now()
                        .format("%m/%d %H:%M:%S%.3f")
                        .to_string(),
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
                    timestamp: chrono::Local::now()
                        .format("%m/%d %H:%M:%S%.3f")
                        .to_string(),
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
            .show(ctx, |_ui| {
                ctx.set_visuals(egui::Visuals {
                    menu_corner_radius: CornerRadius::ZERO,
                    override_text_color: Some(Color32::WHITE),
                    ..egui::Visuals::dark()
                });
                self.add_subscription_window(ctx);
                self.subscriptions_window(ctx);
                self.show_connection_modal(ctx);

                self.menu_bar(ctx);
                self.status_bar(ctx);

                // Tree data grid body
                self.tree_data_view(ctx);
            });
    }
}
