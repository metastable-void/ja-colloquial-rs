
use std::{collections::HashMap, sync::OnceLock};

use serde::Deserialize;

static JSON: &'static str = include_str!("./books.json");

/// Verse of the Christian Bible (in Japanese)
#[derive(Debug, Clone, Deserialize)]
pub struct Verse {
    /// book short name
    pub b: String,

    /// chapter number
    pub c: u8,

    /// verse number
    pub v: u8,

    /// Japanese book name
    pub jb: String,

    /// text (Japanese)
    pub t: String,
}

#[derive(Debug, Clone)]
pub struct ChapterIndex {
    /// number of verses
    pub verse_count: u8,

    /// pointer (index) to verses
    pub indices: HashMap<u8, usize>,
}

#[derive(Debug, Clone)]
pub struct BookIndex {
    /// number of chapters
    pub chapter_count: u8,

    /// Chapters
    pub indices: HashMap<u8, ChapterIndex>,
}

#[derive(Debug, Clone)]
pub struct Books {
    /// raw verses
    pub verses: Vec<Verse>,

    /// list of book short names
    pub book_names: Vec<String>,

    pub book_indices: HashMap<String, BookIndex>,
}

impl Books {
    pub fn get_verse(&self, book: &str, chapter: u8, verse: u8) -> Option<Verse> {
        self.book_indices.get(book)
            .map(|b| b.indices.get(&chapter)).flatten()
            .map(|c| c.indices.get(&verse)).flatten()
            .map(|v| self.verses.get(*v)).flatten().cloned()
    }
}

static LOCK: OnceLock<Books> = OnceLock::new();

pub fn books() -> &'static Books {
    LOCK.get_or_init(|| {
        let verses = serde_json::from_str::<Vec<Verse>>(JSON).unwrap();
        let mut books = Books {
            verses,
            book_names: vec![],
            book_indices: HashMap::new(),
        };

        for (i, verse) in books.verses.iter().enumerate() {
            if !books.book_names.contains(&verse.b) {
                books.book_names.push(verse.b.clone());
            }

            let entry = books.book_indices.entry(verse.b.clone()).or_insert_with(|| {
                BookIndex { chapter_count: 0, indices: HashMap::new() }
            });

            entry.chapter_count = u8::max(entry.chapter_count, verse.c);

            let chapter = entry.indices.entry(verse.c).or_insert_with(|| {
                ChapterIndex { verse_count: 0, indices: HashMap::new() }
            });

            chapter.verse_count = u8::max(chapter.verse_count, verse.v);
            chapter.indices.insert(verse.v, i);
        }

        books
    })
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn genesis() {
        let books = books();
        assert!(books.book_names.contains(&"ge".to_string()));
        assert_eq!(
            books.get_verse("ge", 4, 13).unwrap().t,
            "カインは主に言った、「わたしの罰は重くて負いきれません。",
        );
    }
}
