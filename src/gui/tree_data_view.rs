use super::model::{Highlight, HighlightField};
use eframe::egui::{self, CentralPanel, Color32, Grid, RichText, ScrollArea, Ui, Vec2};
use egui_phosphor::regular as icon;

impl super::App {
    pub(crate) fn tree_data_view(&mut self, ctx: &egui::Context) {
        CentralPanel::default().show(ctx, |ui| {
            // TODO utilize show_viewport instead to render only visible section
            ScrollArea::vertical().auto_shrink(false).show(ui, |ui| {
                Grid::new("data").min_col_width(0.0).show(ui, |ui| {
                    for item in &mut self.gui_data.data {
                        // Omit rows that should be filtered according to the current regex.
                        if self.gui_state.filter_state.regex.is_some() && item.highlights.is_empty()
                        {
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
                        ui.horizontal(|ui| {
                            Self::highlight_text(
                                ui,
                                &item.headers,
                                &item.highlights,
                                HighlightField::Header,
                            )
                        });
                        //ui.label(item.headers.clone());
                        ui.end_row();

                        if item.expanded {
                            ui.label("");
                            ui.label("");
                            //    ui.label(&*item.body);
                            ui.horizontal(|ui| {
                                Self::highlight_text(
                                    ui,
                                    &item.body,
                                    &item.highlights,
                                    HighlightField::Body,
                                )
                            });
                            ui.end_row();
                        }
                    }
                });
            });
        });
    }
    fn highlight_text(
        ui: &mut Ui,
        text: &str,
        highlights: &[Highlight],
        field_specifier: HighlightField,
    ) {
        let mut index = 0;

        let style = ui.style_mut();
        style.spacing.item_spacing = Vec2::new(0.0, 0.0);

        for highlight in highlights {
            if highlight.field != field_specifier {
                continue;
            }
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
}
