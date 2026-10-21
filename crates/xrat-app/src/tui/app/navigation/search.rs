use super::TuiApp;

pub(crate) fn delete_last_word(text: &mut String) {
    while text.ends_with(' ') {
        text.pop();
    }
    while let Some(ch) = text.chars().next_back() {
        if ch == ' ' {
            break;
        }
        text.pop();
    }
}

impl TuiApp {
    pub(crate) fn push_search_char(&mut self, ch: char) {
        if !self.config_list.editing_search || ch.is_control() {
            return;
        }

        self.config_list.search_query.push(ch);
        self.config_list.focused = 0;
    }

    pub(crate) fn pop_search_char(&mut self) {
        if !self.config_list.editing_search {
            return;
        }

        self.config_list.search_query.pop();
        self.config_list.focused = 0;
    }

    pub(crate) fn delete_search_word(&mut self) {
        if !self.config_list.editing_search {
            return;
        }

        delete_last_word(&mut self.config_list.search_query);
        self.config_list.focused = 0;
    }

    pub(crate) fn close_search(&mut self) {
        self.config_list.editing_search = false;
    }
}
