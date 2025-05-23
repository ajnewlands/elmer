use std::{collections::BTreeMap, fmt::Display};

use eframe::egui::{
    self, Align, Button, Color32, Context, FontFamily, FontId, Frame, Grid, Layout, RichText,
    Stroke, TextBuffer, TextEdit, Window,
};
use egui_extras::{Column, Size, StripBuilder, TableBuilder};
use egui_phosphor::regular as icon;
use lapin::{
    options::QueueBindOptions,
    types::{AMQPValue, FieldTable, ShortString},
};
use uuid::Uuid;

use crate::rabbit::Binding;

use super::enums::ModalResult;

#[derive(PartialEq, Clone)]
enum SubscriptionArgumentType {
    Boolean,
    LongString,
    LongLongInt,
    LongInt,
    LongUInt,
    ShortInt,
    ShortUInt,
    ShortShortInt,
    ShortShortUInt,
    Float,
    Double,
    DecimalValue,
}
impl Display for SubscriptionArgumentType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Boolean => "Bool",
                Self::LongInt => "32 bit int",
                Self::LongUInt => "32 bit unsigned int",
                Self::DecimalValue => "Decimal",
                Self::LongString => "String",
                Self::LongLongInt => "64 bit int",
                Self::ShortInt => "16 bit int",
                Self::ShortUInt => "16 bit unsigned int",
                Self::ShortShortInt => "8 bit int",
                Self::ShortShortUInt => "8 bit unsigned int",
                Self::Float => "32 bit float",
                Self::Double => "64 bit float",
            }
        )
    }
}

pub(crate) struct SubscriptionParams {
    pub exchange: String,
    pub routing_key: String,
    pub arguments: Vec<RawSubscriptionArgument>,
}

impl SubscriptionParams {
    // TODO; just use this to generate the dialogue validation error
    pub(crate) fn as_binding(&self) -> Result<Binding, String> {
        if self.exchange.is_empty() {
            return Err("Exchange cannot be empty".into());
        }

        let mut args = BTreeMap::<ShortString, AMQPValue>::new();
        for a in &self.arguments {
            args.insert(ShortString::from(a.name.clone()), a.parse_value()?);
        }

        Ok(Binding {
            id: Uuid::new_v4(),
            exchange: self.exchange.clone(),
            routing_key: self.routing_key.clone(),
            arguments: args.into(),
        })
    }
}

pub struct RawSubscriptionArgument {
    t: SubscriptionArgumentType,
    value: String,
    name: String,
}

impl Default for RawSubscriptionArgument {
    fn default() -> Self {
        Self {
            t: SubscriptionArgumentType::LongString,
            value: "".into(),
            name: "".into(),
        }
    }
}

impl RawSubscriptionArgument {
    pub fn is_valid(&self) -> bool {
        self.is_name_valid() && self.is_value_valid()
    }

    pub fn is_name_valid(&self) -> bool {
        !self.name.is_empty()
    }

    pub fn stroke_for_name(&self) -> Stroke {
        if self.is_name_valid() {
            Stroke::NONE
        } else {
            Stroke::new(1.0, Color32::RED)
        }
    }
    pub fn stroke_for_value(&self) -> Stroke {
        if self.is_value_valid() {
            Stroke::NONE
        } else {
            Stroke::new(1.0, Color32::RED)
        }
    }

    pub fn parse_value(&self) -> Result<AMQPValue, String> {
        fn parse_error(arg: &RawSubscriptionArgument) -> String {
            format!("Invalid value '{}' for field '{}'", arg.value, arg.name)
        }
        Ok(match self.t {
            SubscriptionArgumentType::Boolean => {
                AMQPValue::Boolean(self.value.parse().map_err(|_| parse_error(self))?)
            }
            SubscriptionArgumentType::LongInt => {
                AMQPValue::LongInt(self.value.parse().map_err(|_| parse_error(self))?)
            }
            SubscriptionArgumentType::LongUInt => {
                AMQPValue::LongUInt(self.value.parse().map_err(|_| parse_error(self))?)
            }
            SubscriptionArgumentType::DecimalValue => AMQPValue::DecimalValue(
                serde_json::from_str(&self.value).map_err(|_| parse_error(self))?,
            ),
            SubscriptionArgumentType::LongString => {
                AMQPValue::LongString(self.value.clone().into())
            }
            SubscriptionArgumentType::LongLongInt => {
                AMQPValue::LongLongInt(self.value.parse().map_err(|_| parse_error(self))?)
            }
            SubscriptionArgumentType::ShortInt => {
                AMQPValue::ShortInt(self.value.parse().map_err(|_| parse_error(self))?)
            }
            SubscriptionArgumentType::ShortUInt => {
                AMQPValue::ShortUInt(self.value.parse().map_err(|_| parse_error(self))?)
            }
            SubscriptionArgumentType::ShortShortInt => {
                AMQPValue::ShortShortInt(self.value.parse().map_err(|_| parse_error(self))?)
            }
            SubscriptionArgumentType::ShortShortUInt => {
                AMQPValue::ShortShortUInt(self.value.parse().map_err(|_| parse_error(self))?)
            }
            SubscriptionArgumentType::Float => {
                AMQPValue::Float(self.value.parse().map_err(|_| parse_error(self))?)
            }
            SubscriptionArgumentType::Double => {
                AMQPValue::Double(self.value.parse().map_err(|_| parse_error(self))?)
            }
        })
    }

    pub fn is_value_valid(&self) -> bool {
        self.parse_value().is_ok()
    }
}

impl Default for SubscriptionParams {
    fn default() -> Self {
        Self {
            exchange: String::default(),
            routing_key: String::default(),
            arguments: Vec::<RawSubscriptionArgument>::default(),
        }
    }
}

impl super::App {
    pub(crate) fn add_subscription_window(&mut self, ctx: &Context) {
        if let Some(params) = &mut self.gui_state.add_subscription_parameters {
            let mut delete_index = None;

            let mut result: ModalResult = ModalResult::None;

            let binding: Option<Binding>;
            let error: String;

            match params.as_binding() {
                Ok(b) => {
                    binding = Some(b);
                    error = String::default();
                }
                Err(e) => {
                    binding = None;
                    error = e;
                }
            }
            Window::new("Add subscription")
                .movable(true)
                .resizable(true)
                .collapsible(false)
                .show(ctx, |ui| {
                    let style = ui.style_mut();
                    style.override_font_id = Some(FontId {
                        size: 16.0,
                        family: FontFamily::Proportional,
                    });
                    Grid::new("subscription")
                        .num_columns(2)
                        .min_col_width(100.0)
                        .show(ui, |ui| {
                            ui.with_layout(egui::Layout::right_to_left(Align::Center), |ui| {
                                ui.label("Exchange");
                            });
                            let stroke = if params.exchange.is_empty() {
                                Stroke::new(1.0, Color32::RED)
                            } else {
                                Stroke::NONE
                            };
                            Frame::NONE.stroke(stroke).show(ui, |ui| {
                                ui.add_sized([ui.available_width(), 16.0], {
                                    egui::TextEdit::singleline(&mut params.exchange)
                                })
                            });
                            ui.end_row();

                            ui.with_layout(egui::Layout::right_to_left(Align::Center), |ui| {
                                ui.label("Routing key");
                            });
                            ui.add_sized([ui.available_width(), 16.0], {
                                egui::TextEdit::singleline(&mut params.routing_key)
                            });
                            ui.end_row();
                        });
                    ui.add(egui::Separator::default().horizontal());

                    // Table of arguments
                    let available_height = ui.available_height();
                    let table = TableBuilder::new(ui)
                        .striped(false)
                        .resizable(false)
                        .cell_layout(Layout::right_to_left(Align::Center))
                        .column(Column::remainder())
                        .column(Column::remainder()) // remainder() was not working
                        .column(Column::auto())
                        .column(Column::auto().at_least(24.0))
                        .auto_shrink([false, false])
                        .max_scroll_height(available_height - 60.0); // Leave room for window decorations

                    table
                        .header(16.0, |mut header| {
                            header.col(|ui| {
                                ui.label("Argument");
                            });
                            header.col(|ui| {
                                ui.label("Value");
                            });
                            header.col(|ui| {
                                ui.label("Type");
                            });
                            header.col(|ui| {
                                if ui
                                    .add(
                                        Button::new(
                                            RichText::new(icon::PLUS).color(Color32::GREEN),
                                        )
                                        .fill(Color32::TRANSPARENT),
                                    )
                                    .clicked()
                                {
                                    params.arguments.push(Default::default());
                                }
                            });
                        })
                        .body(|mut body| {
                            for index in 0..params.arguments.len() {
                                let arg = &mut params.arguments[index];
                                body.row(16.0, |mut row| {
                                    row.col(|ui| {
                                        Frame::NONE.stroke(arg.stroke_for_name()).show(ui, |ui| {
                                            ui.add(
                                                TextEdit::singleline(&mut arg.name)
                                                    .hint_text("X-my-arg"),
                                            )
                                        });
                                    });

                                    row.col(|ui| {
                                        Frame::NONE.stroke(arg.stroke_for_value()).show(ui, |ui| {
                                            ui.text_edit_singleline(&mut arg.value);
                                        });
                                    });
                                    row.col(|ui| {
                                        egui::ComboBox::from_label("")
                                            .selected_text(format!("{}", arg.t))
                                            .show_ui(ui, |ui| {
                                                ui.selectable_value(
                                                    &mut arg.t,
                                                    SubscriptionArgumentType::Boolean,
                                                    "bool",
                                                );
                                                ui.selectable_value(
                                                    &mut arg.t,
                                                    SubscriptionArgumentType::LongString,
                                                    "string",
                                                );
                                                ui.selectable_value(
                                                    &mut arg.t,
                                                    SubscriptionArgumentType::LongLongInt,
                                                    "64 bit int",
                                                );
                                                ui.selectable_value(
                                                    &mut arg.t,
                                                    SubscriptionArgumentType::LongInt,
                                                    "32 bit int",
                                                );
                                                ui.selectable_value(
                                                    &mut arg.t,
                                                    SubscriptionArgumentType::LongUInt,
                                                    "32 bit unsigned int",
                                                );
                                                ui.selectable_value(
                                                    &mut arg.t,
                                                    SubscriptionArgumentType::ShortInt,
                                                    "16 bit int",
                                                );
                                                ui.selectable_value(
                                                    &mut arg.t,
                                                    SubscriptionArgumentType::ShortUInt,
                                                    "16 bit unsigned int",
                                                );
                                                ui.selectable_value(
                                                    &mut arg.t,
                                                    SubscriptionArgumentType::ShortShortInt,
                                                    "8 bit int",
                                                );
                                                ui.selectable_value(
                                                    &mut arg.t,
                                                    SubscriptionArgumentType::ShortShortUInt,
                                                    "8 bit unsigned int",
                                                );
                                                ui.selectable_value(
                                                    &mut arg.t,
                                                    SubscriptionArgumentType::Float,
                                                    "32 bit float",
                                                );
                                                ui.selectable_value(
                                                    &mut arg.t,
                                                    SubscriptionArgumentType::Double,
                                                    "64 bit float",
                                                );
                                                ui.selectable_value(
                                                    &mut arg.t,
                                                    SubscriptionArgumentType::DecimalValue,
                                                    "Decimal with scale and val",
                                                );
                                            });
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
                                            delete_index = Some(index);
                                        }
                                    });
                                });
                            }
                        });

                    StripBuilder::new(ui)
                        .size(Size::remainder())
                        .size(Size::remainder())
                        .horizontal(|mut strip| {
                            strip.cell(|ui| {
                                ui.add_enabled_ui(error.is_empty(), |ui| {
                                    if ui
                                        .add_sized(
                                            [ui.available_width(), 24.0],
                                            Button::new(RichText::new("OK"))
                                                .fill(Color32::DARK_GREEN),
                                        )
                                        .on_disabled_hover_text(error)
                                        .clicked()
                                    {
                                        let _ = self.connection_manager.bind(binding.unwrap());
                                    }
                                });
                            });
                            strip.cell(|ui| {
                                if ui
                                    .add_sized(
                                        [ui.available_width(), 24.0],
                                        Button::new(RichText::new("Cancel")),
                                    )
                                    .clicked()
                                {
                                    result = ModalResult::Cancel;
                                }
                            });
                        });
                });

            if let Some(index) = delete_index {
                params.arguments.remove(index);
            }
            match result {
                ModalResult::None => (),
                _ => self.gui_state.add_subscription_parameters = None,
            }
        }
    }
}
