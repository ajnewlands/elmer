use std::default;

use super::{enums::ModalResult, prelude::*};

fn modal_label(ui: &mut Ui, label: &str, binding: &mut String, password: bool) -> Response {
    let label1 = ui.label(RichText::new(label).size(16.0));
    ui.add_sized(
        [200.0, 24.0],
        TextEdit::singleline(binding)
            .password(password)
            .vertical_align(egui::Align::Center)
            .font(FontId {
                size: 16.0,
                family: FontFamily::Proportional,
            }),
    )
    .labelled_by(label1.id)
}

impl App {
    pub(crate) fn show_connection_modal(&mut self, _ui: &mut Ui, ctx: &egui::Context) {
        let modal = egui::containers::Modal::new(Id::new("connection"));

        let mut modal_result = ModalResult::None;
        // Mutate temporary state; only committed on 'connect' (and not on 'cancel')
        if let Some(con) = &mut self.gui_state.connection_modal_parameters {
            modal.show(ctx, |ui| {
                let mut changed = Vec::<bool>::default();
                Grid::new("data").min_col_width(80.0).show(ui, |ui| {
                    changed.push(modal_label(ui, "Hostname", &mut con.hostname, false).changed());
                    ui.end_row();
                    changed.push(modal_label(ui, "Port", &mut con.port, false).changed());
                    ui.end_row();

                    changed.push(modal_label(ui, "Virtual Host", &mut con.vhost, false).changed());
                    ui.end_row();

                    changed.push(modal_label(ui, "Exchange", &mut con.exchange, false).changed());
                    ui.end_row();

                    changed.push(modal_label(ui, "Username", &mut con.username, false).changed());
                    ui.end_row();

                    changed.push(modal_label(ui, "Password", &mut con.password, true).changed());
                    ui.end_row();

                    ui.label("");
                    changed.push(
                        ui.checkbox(&mut con.tls, RichText::new("Enable TLS").size(16.0))
                            .changed(),
                    );
                    ui.end_row();

                    ui.label("");
                    changed.push(
                        ui.checkbox(
                            &mut con.wildcard,
                            RichText::new("Wildcard subscription").size(16.0),
                        )
                        .changed(),
                    );
                    ui.end_row();

                    if changed.iter().find(|b| **b).is_some() {
                        con.validate();
                    }
                });

                // Filthy hack to make layout work here.
                ui.horizontal(|ui| {
                    ui.add_enabled_ui(con.validation_error.is_none(), |ui| {
                        if ui
                            .add_sized(
                                [140.0, 24.0],
                                Button::new(RichText::new("Connect")).fill(Color32::DARK_GREEN),
                            )
                            .on_disabled_hover_text(
                                con.validation_error.as_deref().unwrap_or_default(),
                            )
                            .clicked()
                        {
                            modal_result = ModalResult::Ok;
                        }
                    });

                    if ui
                        .add_sized([140.0, 24.0], Button::new(RichText::new("Cancel")))
                        .clicked()
                    {
                        modal_result = ModalResult::Cancel;
                    }
                });
            });

            match modal_result {
                ModalResult::Ok => {
                    self.gui_state.connection_parameters = con.clone();
                    self.gui_state.connection_parameters.validate();
                    self.close_connection_modal();
                    self.connection_manager
                        .tx
                        .send(crate::rabbit::ConnectionCommand::Connect(
                            self.gui_state.connection_parameters.build_url(),
                            ctx.clone(),
                        ))
                        .expect("Internal channel closed");
                }
                ModalResult::Cancel => {
                    self.close_connection_modal();
                }
                ModalResult::None => (),
            };
        }
    }

    fn close_connection_modal(&mut self) {
        self.gui_state.connection_modal_parameters = None;
    }
}
