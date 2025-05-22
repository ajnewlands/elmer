use eframe::egui::{
    Align, Button, Color32, Context, FontFamily, FontId, Label, Layout, RichText, Window,
};
use egui_extras::{Column, TableBuilder};
use egui_phosphor::regular as icon;

impl super::App {
    pub(crate) fn subscriptions_window(&mut self, ctx: &Context) {
        Window::new("Subscriptions")
            .movable(true)
            .resizable(true)
            .collapsible(false)
            .open(&mut self.gui_state.show_subscriptions)
            .show(ctx, |ui| {
                let style = ui.style_mut();
                style.override_font_id = Some(FontId {
                    size: 16.0,
                    family: FontFamily::Proportional,
                });
                let available_height = ui.available_height();
                let table = TableBuilder::new(ui)
                    .striped(false)
                    .resizable(false)
                    .cell_layout(Layout::right_to_left(Align::Center))
                    .column(Column::auto())
                    .column(Column::auto())
                    .column(Column::remainder())
                    .column(Column::auto().at_least(32.0))
                    .auto_shrink(false)
                    .max_scroll_height(available_height - 120.0); // leave room for window decorations etc

                table
                    .header(16.0, |mut header| {
                        header.col(|ui| {
                            ui.label("Exchange");
                        });
                        header.col(|ui| {
                            ui.label("Routing Key");
                        });
                        header.col(|ui| {
                            ui.label("Arguments");
                        });
                        header.col(|ui| {
                            ui.label("");
                        });
                    })
                    .body(|mut body| {
                        for subscription in &self.queue_bindings {
                            body.row(16.0, |mut row| {
                                row.col(|ui| {
                                    ui.label(&subscription.exchange);
                                });
                                row.col(|ui| {
                                    ui.label(&subscription.routing_key);
                                });
                                row.col(|ui| {
                                    ui.add(
                                        Label::new(
                                            crate::rabbit::field_table_to_json(
                                                &subscription.arguments,
                                            )
                                            .to_string(),
                                        )
                                        .wrap(),
                                    );
                                });
                                row.col(|ui| {
                                    if ui
                                        .add(
                                            Button::new(
                                                RichText::new(icon::TRASH).color(Color32::RED),
                                            )
                                            .fill(Color32::TRANSPARENT),
                                        )
                                        .clicked()
                                    {
                                        self.connection_manager.unbind((*subscription).clone());
                                    }
                                });
                            })
                        }
                    });
            });
    }
}
