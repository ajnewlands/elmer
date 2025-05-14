use std::{borrow::Cow, collections::VecDeque};

use super::state::GuiState;

pub const DEFAULT_DATA_LIMIT: usize = 1000;
pub struct Model<'a> {
    data_limit: usize,
    pub data: VecDeque<ModelItem<'a>>,
}

impl Default for Model<'_> {
    fn default() -> Self {
        let mut data = VecDeque::default();
        for i in 0..200 {
            data.push_back(ModelItem {
                headers: Cow::Owned(format!("Data headers {}", i)),
                body: Cow::Owned(format!("Data body {}", i)),
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

impl Model<'_> {
    pub fn filter_all(&mut self, gui_state: &GuiState) {
        for item in &mut self.data {
            item.apply_filter(gui_state);
        }
    }
}

pub struct Highlight {
    pub start: usize,
    pub end: usize,
}

pub struct ModelItem<'a> {
    pub headers: Cow<'a, str>,
    pub body: Cow<'a, str>,
    pub expanded: bool,
    pub highlights: Vec<Highlight>,
}

impl ModelItem<'_> {
    fn apply_filter(&mut self, gui_state: &GuiState) {
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
                            start: m.start(),
                            end: m.end(),
                        })
                        .collect();
                }
            }
        }
    }
}
