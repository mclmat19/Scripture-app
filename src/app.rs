// src/app.rs
use crate::bible::{Book, Testament, load_bible};

#[derive(Debug, Clone, PartialEq)]
pub enum Pane {
    Tree,
    Reader,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Mode {
    Menu,
    Normal,
    Search,
}

pub struct App {
    pub books: Vec<Book>,
    pub active_pane: Pane,
    pub mode: Mode,

    // Tree state
    pub tree_selected: usize,
    pub expanded_books: Vec<bool>,

    // Reader state
    pub selected_book: usize,
    pub selected_chapter: usize,
    pub selected_verse: usize,
    pub scroll_offset: usize,

    // Search
    pub search_query: String,
    pub search_results: Vec<(usize, usize, usize)>, // (book_idx, chapter_idx, verse_idx)
    pub search_cursor: usize,
    pub search_navigating:bool,
    pub should_quit: bool,
    
    pub search_origin: Mode,

}

#[derive(Debug, Clone)]
pub enum TreeItem {
    Testament(String),
    Book { book_idx: usize, expanded: bool },
    Chapter { book_idx: usize, chapter_idx: usize },
}

impl App {
    pub fn new() -> Self {
        let books = load_bible();
        let count = books.len();
        App {
            books,
            active_pane: Pane::Tree,
            mode: Mode::Menu,
            tree_selected: 1, // start on first book
            expanded_books: vec![false; count],
            selected_book: 0,
            selected_chapter: 0,
            selected_verse: 0,
            scroll_offset: 0,
            search_query: String::new(),
            search_navigating: false,
            search_results: vec![],
            search_cursor: 0,
            search_origin: Mode::Menu,
            should_quit: false,
        }
    }

    /// Build a flat list of visible tree items (testament headers + books + expanded chapters)
    pub fn flat_tree(&self) -> Vec<TreeItem> {
        let mut items = vec![];
        let mut last_testament = None;

        for (book_idx, book) in self.books.iter().enumerate() {
            let t = &book.testament;
            if last_testament.as_ref() != Some(t) {
                let label = if *t == Testament::Old {
                    "Old Testament".into()
                } else {
                    "New Testament".into()
                };
                items.push(TreeItem::Testament(label));
                last_testament = Some(t.clone());
            }
            let expanded = self.expanded_books[book_idx];
            items.push(TreeItem::Book { book_idx, expanded });
            if expanded {
                for chapter_idx in 0..book.chapters.len() {
                    items.push(TreeItem::Chapter { book_idx, chapter_idx });
                }
            }
        }
        items
    }

    pub fn tree_down(&mut self) {
        let len = self.flat_tree().len();
        if self.tree_selected + 1 < len {
            self.tree_selected += 1;
        }
    }

    pub fn tree_up(&mut self) {
        if self.tree_selected > 0 {
            self.tree_selected -= 1;
        }
    }

    pub fn tree_enter(&mut self) {
        let items = self.flat_tree();
        if let Some(item) = items.get(self.tree_selected) {
            match item.clone() {
                TreeItem::Book { book_idx, .. } => {
                    self.expanded_books[book_idx] = !self.expanded_books[book_idx];
                }
                TreeItem::Chapter { book_idx, chapter_idx } => {
                    self.selected_book = book_idx;
                    self.selected_chapter = chapter_idx;
                    self.selected_verse = 0;
                    self.scroll_offset = 0;
                    self.active_pane = Pane::Reader;
                }
                TreeItem::Testament(_) => {}
            }
        }
    }

    pub fn reader_down(&mut self) {
        if let Some(book) = self.books.get(self.selected_book) {
            if let Some(chapter) = book.chapters.get(self.selected_chapter) {
                if self.selected_verse + 1 < chapter.verses.len() {
                    self.selected_verse += 1;
                    if self.selected_verse >= self.scroll_offset + 10 {
                        self.scroll_offset += 1;
                    }
                }
            }
        }
    }

    pub fn reader_up(&mut self) {
        if self.selected_verse > 0 {
            self.selected_verse -= 1;
            if self.selected_verse < self.scroll_offset {
                self.scroll_offset = self.selected_verse;
            }
        }
    }

    pub fn next_chapter(&mut self) {
        if let Some(book) = self.books.get(self.selected_book) {
            if self.selected_chapter + 1 < book.chapters.len() {
                self.selected_chapter += 1;
                self.selected_verse = 0;
                self.scroll_offset = 0;
            } else if self.selected_book + 1 < self.books.len() {
                self.selected_book += 1;
                self.selected_chapter = 0;
                self.selected_verse = 0;
                self.scroll_offset = 0;
            }
        }
    }

    pub fn prev_chapter(&mut self) {
        if self.selected_chapter > 0 {
            self.selected_chapter -= 1;
            self.selected_verse = 0;
            self.scroll_offset = 0;
        } else if self.selected_book > 0 {
            self.selected_book -= 1;
            if let Some(book) = self.books.get(self.selected_book) {
                self.selected_chapter = book.chapters.len().saturating_sub(1);
            }
            self.selected_verse = 0;
            self.scroll_offset = 0;
        }
    }

    pub fn search_execute(&mut self) {
        self.search_results.clear();
        self.search_cursor = 0;
        let query = self.search_query.to_lowercase();
        if query.is_empty() {
            return;
        }
        for (bi, book) in self.books.iter().enumerate() {
            for (ci, chapter) in book.chapters.iter().enumerate() {
                for (vi, verse) in chapter.verses.iter().enumerate() {
                    if verse.text.to_lowercase().contains(&query) {
                        self.search_results.push((bi, ci, vi));
                    }
                }
            }
        }
    }

    pub fn jump_to_search_result(&mut self) {
        if let Some(&(bi, ci, vi)) = self.search_results.get(self.search_cursor) {
            self.selected_book = bi;
            self.selected_chapter = ci;
            self.selected_verse = vi;
            self.scroll_offset = vi.saturating_sub(4);
            self.active_pane = Pane::Reader;
            self.mode = Mode::Normal;
        }
    }

    pub fn current_book_name(&self) -> &str {
            self.books
                .get(self.selected_book)
                .map(|b| b.name.as_str())
                .unwrap_or("—")
        }

        pub fn current_chapter_num(&self) -> u32 {
            self.books
                .get(self.selected_book)
            .and_then(|b| b.chapters.get(self.selected_chapter))
            .map(|c| c.number)
            .unwrap_or(0)
    }
}
