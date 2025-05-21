use super::model::{Highlight, HighlightField};
use eframe::egui::{
    self, text::LayoutJob, CentralPanel, Color32, Grid, RichText, ScrollArea, TextFormat, Ui,
};
use egui_phosphor::regular as icon;

impl super::App {
    pub(crate) fn tree_data_view(&mut self, ctx: &egui::Context) {
        CentralPanel::default().show(ctx, |ui| {
            // TODO utilize show_viewport instead to render only visible section
            ScrollArea::vertical().auto_shrink(false).show(ui, |ui| {
                Grid::new("data")
                    .num_columns(3)
                    .min_col_width(0.0)
                    .show(ui, |ui| {
                        for item in &mut self.gui_data.data {
                            // Omit rows that should be filtered according to the current regex.
                            if self.gui_state.filter_state.regex.is_some()
                                && item.highlights.is_empty()
                            {
                                continue;
                            }
                            let caret = if item.expanded {
                                icon::CARET_DOWN
                            } else {
                                icon::CARET_RIGHT
                            };
                            if ui
                                .add(
                                    egui::Button::new(RichText::new(caret))
                                        .fill(Color32::TRANSPARENT),
                                )
                                .clicked()
                            {
                                item.expanded = !item.expanded;
                            }
                            ui.label("timestamp");
                            Self::highlight_text(
                                ui,
                                &item.headers,
                                &item.highlights,
                                HighlightField::Header,
                            );
                            ui.end_row();

                            if item.expanded {
                                ui.label("");
                                ui.label("");
                                Self::highlight_text(
                                    ui,
                                    &item.body,
                                    &item.highlights,
                                    HighlightField::Body,
                                );
                                ui.end_row();
                            }
                        }
                    })
            });
        });
    }

    fn highlight_text(
        ui: &mut Ui,
        text: &str,
        highlights: &[Highlight],
        field_specifier: HighlightField,
    ) {
        let mut job = LayoutJob::default();
        let mut index = 0;

        let text_format_regular = TextFormat {
            color: Color32::WHITE,
            ..Default::default()
        };
        let text_format_highlighted = TextFormat {
            color: Color32::BLACK,
            background: Color32::LIGHT_BLUE,
            ..Default::default()
        };

        for highlight in highlights {
            if highlight.field != field_specifier {
                continue;
            }
            // Add normal text before the highlight
            if index < highlight.start {
                job.append(
                    &text[index..highlight.start],
                    0.0,
                    text_format_regular.clone(),
                );
            }

            // Add highlighted text
            job.append(
                &text[highlight.start..highlight.end],
                0.0,
                text_format_highlighted.clone(),
            );
            index = highlight.end;
        }

        // Add remaining text after the last highlight
        if index < text.len() {
            job.append(&text[index..], 0.0, text_format_regular.clone());
        }
        ui.add(egui::Label::new(job).wrap());
    }
}
