use std::{borrow::Cow, collections::VecDeque, rc::Rc};

use super::state::GuiState;

pub const DEFAULT_DATA_LIMIT: usize = 1000;
pub struct Model {
    data_limit: usize,
    pub data: VecDeque<ModelItem>,
}

impl Default for Model {
    fn default() -> Self {
        let mut data = VecDeque::default();
        for i in 0..20 {
            data.push_back(ModelItem {
                headers: Rc::new(format!("Data headers {}", i)),
                body: Rc::new(format!("Data body {}", i)),
                expanded: false,
                highlights: Vec::default(),
            });
        }

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
        match &gui_state.regex {
            None => {
                self.highlights.clear();
                self.highlights.shrink_to_fit();
            }
            Some(regex) => {
                let mut it = regex.find_iter(&self.headers).peekable();
                if it.peek().is_none() {
                    self.highlights.clear();
                    self.highlights.shrink_to_fit();
                } else {
                    self.highlights = it
                        .map(|m| Highlight {
                            field: HighlightField::Header,
                            start: m.start(),
                            end: m.end(),
                        })
                        .collect();
                }
            }
        }
    }
}
