use super::entries::{EntryType, FileEntry};
use std::cmp;

#[derive(Copy, Clone, Default)]
pub struct ListState(Option<usize>);

impl ListState {
    pub fn select(&mut self, index: Option<usize>) {
        self.0 = index;
    }

    pub fn selected(&self) -> Option<usize> {
        self.0
    }
}

#[derive(Default)]
pub struct ResultList {
    entries: Vec<EntryType>,
    state: ListState,
}

impl ResultList {
    pub fn add_entry(&mut self, mut entry: FileEntry) {
        self.entries.append(&mut entry.0);

        if self.state.selected().is_none() {
            self.next_match();
        }
    }

    pub fn iter(&self) -> std::slice::Iter<EntryType> {
        self.entries.iter()
    }

    pub fn clear(&mut self) {
        self.entries.clear();
        self.state.select(None);
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    pub fn next_match(&mut self) {
        if self.is_empty() {
            return;
        }

        let index = match self.state.selected() {
            Some(i) => {
                if i == self.entries.len() - 1 {
                    i
                } else {
                    match self.entries[i + 1] {
                        EntryType::Header(_) => i + 2,
                        EntryType::Match(_, _) => i + 1,
                    }
                }
            }
            None => 1,
        };

        self.state.select(Some(index));
    }

    pub fn previous_match(&mut self) {
        if self.is_empty() {
            return;
        }

        let index = match self.state.selected() {
            Some(i) => {
                if i == 1 {
                    1
                } else {
                    match self.entries[i - 1] {
                        EntryType::Header(_) => i - 2,
                        EntryType::Match(_, _) => i - 1,
                    }
                }
            }
            None => 1,
        };

        self.state.select(Some(index));
    }

    pub fn next_file(&mut self) {
        if self.is_empty() {
            return;
        }

        let index = match self.state.selected() {
            Some(i) => {
                let mut next_index = i;
                loop {
                    if next_index == self.entries.len() - 1 {
                        next_index = i;
                        break;
                    }

                    next_index += 1;
                    match self.entries[next_index] {
                        EntryType::Header(_) => {
                            next_index += 1;
                            break;
                        }
                        EntryType::Match(_, _) => continue,
                    }
                }
                next_index
            }
            None => 1,
        };

        self.state.select(Some(index));
    }

    pub fn previous_file(&mut self) {
        if self.is_empty() {
            return;
        }

        let index = match self.state.selected() {
            Some(i) => {
                let mut next_index = i;
                let mut first_header_visited = false;
                loop {
                    if next_index == 1 {
                        break;
                    }

                    next_index -= 1;
                    match self.entries[next_index] {
                        EntryType::Header(_) => {
                            if !first_header_visited {
                                first_header_visited = true;
                                next_index -= 1;
                            } else {
                                next_index += 1;
                                break;
                            }
                        }
                        EntryType::Match(_, _) => continue,
                    }
                }
                next_index
            }
            None => 1,
        };

        self.state.select(Some(index));
    }

    pub fn top(&mut self) {
        if self.is_empty() {
            return;
        }

        self.state.select(Some(1));
    }

    pub fn bottom(&mut self) {
        if self.is_empty() {
            return;
        }

        self.state.select(Some(self.entries.len() - 1));
    }

    pub fn remove_current_entry(&mut self) {
        if self.is_empty() {
            return;
        }

        if self.is_last_match_in_file() {
            self.remove_current_file();
        } else {
            self.remove_current_entry_and_select_previous();
        }
    }

    pub fn remove_current_file(&mut self) {
        if self.is_empty() {
            return;
        }

        let selected_index = self.state.selected().unwrap();

        let mut current_file_header_index = 0;
        for index in (0..selected_index).rev() {
            if self.is_header(index) {
                current_file_header_index = index;
                break;
            }
        }

        let mut next_file_header_index = self.entries.len();
        for index in selected_index..self.entries.len() {
            if self.is_header(index) {
                next_file_header_index = index;
                break;
            }
        }

        let span = next_file_header_index - current_file_header_index;
        for _ in 0..span {
            self.entries.remove(current_file_header_index);
        }

        if self.entries.is_empty() {
            self.state.select(None);
        } else {
            if selected_index != 1 {
                self.state.select(Some(cmp::max(
                    current_file_header_index.saturating_sub(1),
                    1,
                )));
            }
        }
    }

    fn is_header(&self, index: usize) -> bool {
        matches!(self.entries[index], EntryType::Header(_))
    }

    fn is_last_match_in_file(&self) -> bool {
        let current_index = self.state.selected().unwrap();
        if self.is_header(current_index - 1) {
            if current_index == self.entries.len() - 1 || self.is_header(current_index + 1) {
                true
            } else {
                false
            }
        } else {
            false
        }
    }

    fn remove_current_entry_and_select_previous(&mut self) {
        let selected_index = self.state.selected().unwrap();
        self.entries.remove(selected_index);
        if selected_index >= self.entries.len() || self.is_header(selected_index) {
            self.state.select(Some(selected_index - 1));
        }
    }

    pub fn get_selected_entry(&self) -> Option<(&str, u64)> {
        match self.state.selected() {
            Some(i) => {
                let mut line_number: Option<u64> = None;
                for index in (0..=i).rev() {
                    match &self.entries[index] {
                        EntryType::Header(name) => {
                            return Some((name.as_str(), line_number.unwrap()));
                        }
                        EntryType::Match(number, _) => {
                            if line_number.is_none() {
                                line_number = Some(*number);
                            }
                        }
                    }
                }
                None
            }
            None => None,
        }
    }

    pub fn get_state(&self) -> ListState {
        self.state
    }

    pub fn get_current_match_index(&self) -> usize {
        match self.state.selected() {
            Some(selected) => {
                self.entries
                    .iter()
                    .take(selected)
                    .filter(|&e| matches!(e, EntryType::Match(_, _)))
                    .count()
                    + 1
            }
            None => 0,
        }
    }

    pub fn get_number_of_matches(&self) -> usize {
        self.entries
            .iter()
            .filter(|&e| matches!(e, EntryType::Match(_, _)))
            .count()
    }

    pub fn get_number_of_file_entries(&self) -> usize {
        self.entries
            .iter()
            .filter(|&e| matches!(e, EntryType::Header(_)))
            .count()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ig::entries::Match;

    #[test]
    fn test_empty_list() {
        let mut list = ResultList::default();
        assert_eq!(list.state.selected(), None);
        list.next_match();
        assert_eq!(list.state.selected(), None);
        list.previous_match();
        assert_eq!(list.state.selected(), None);
    }

    #[test]
    fn test_add_entry() {
        let mut list = ResultList::default();
        list.add_entry(FileEntry::new("entry1", vec![Match::new(0, "e1m1")]));
        assert_eq!(list.entries.len(), 2);
        assert_eq!(list.state.selected(), Some(1));

        list.add_entry(FileEntry::new(
            "entry2",
            vec![Match::new(0, "e1m2"), Match::new(0, "e2m2")],
        ));
        assert_eq!(list.entries.len(), 5);
        assert_eq!(list.state.selected(), Some(1));
    }
}
