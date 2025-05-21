use std::{collections::VecDeque, rc::Rc};

use super::state::GuiState;

pub const DEFAULT_DATA_LIMIT: usize = 1000;
pub struct Model {
    data_limit: usize,
    pub data: VecDeque<ModelItem>,
}

impl Default for Model {
    fn default() -> Self {
        let data = VecDeque::default();

        Self {
            data,
            data_limit: DEFAULT_DATA_LIMIT,
        }
    }
}

impl Model {
    pub fn filter_all(&mut self, gui_state: &GuiState) {
        for item in &mut self.data {
            item.apply_filter(gui_state);
        }
    }
}

#[derive(PartialEq, Eq)]
pub enum HighlightField {
    Header,
    Body,
}

pub struct Highlight {
    pub field: HighlightField,
    pub start: usize,
    pub end: usize,
}

pub struct ModelItem {
    pub headers: Rc<String>,
    pub body: Rc<String>,
    pub expanded: bool,
    pub highlights: Vec<Highlight>,
}

impl ModelItem {
    pub fn apply_filter(&mut self, gui_state: &GuiState) {
        match &gui_state.filter_state.regex {
            None => {
                self.highlights.clear();
                self.highlights.shrink_to_fit();
            }
            Some(regex) => {
                self.highlights.clear();
                if gui_state.filter_state.filter_headers {
                    let it = regex.find_iter(&self.headers).peekable();
                    self.highlights = it
                        .map(|m| Highlight {
                            field: HighlightField::Header,
                            start: m.start(),
                            end: m.end(),
                        })
                        .collect();
                }
                if gui_state.filter_state.filter_body {
                    let it = regex.find_iter(&self.body).peekable();
                    self.highlights = it
                        .map(|m| Highlight {
                            field: HighlightField::Body,
                            start: m.start(),
                            end: m.end(),
                        })
                        .collect();
                }
                self.highlights.shrink_to_fit();
            }
        }
    }
}
