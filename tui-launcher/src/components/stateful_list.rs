// ===================================================================
// Stateful List Component
// ===================================================================
// A generic struct to manage the state of any selectable list in the UI.

use ratatui::widgets::ListState;

pub struct StatefulList<T> {
    pub state: ListState,
    pub items: Vec<T>,
}

impl<T> StatefulList<T> {
    pub fn with_items(items: Vec<T>) -> Self {
        // FIX: Check if the vector is empty before moving it.
        let is_empty = items.is_empty();
        let mut list = Self { state: ListState::default(), items };
        if !is_empty {
            list.state.select(Some(0));
        }
        list
    }

    pub fn next(&mut self) {
        if self.items.is_empty() { return; }
        let i = self.state.selected().map_or(0, |i| {
            if i >= self.items.len() - 1 { 0 } else { i + 1 }
        });
        self.state.select(Some(i));
    }

    pub fn previous(&mut self) {
        if self.items.is_empty() { return; }
        let i = self.state.selected().map_or(0, |i| {
            if i == 0 { self.items.len() - 1 } else { i - 1 }
        });
        self.state.select(Some(i));
    }

    pub fn selected_item(&self) -> Option<&T> {
        match self.state.selected() {
            Some(i) => self.items.get(i),
            None => None,
        }
    }
}
