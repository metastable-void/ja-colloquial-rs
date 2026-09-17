//! Allocation-free access to the Japanese Colloquial Bible.
//!
//! Book lookup uses the full Japanese names returned by [`Bible::book_names`].
//! Source-data abbreviations are intentionally not part of the public API.
//! Random selection is deterministic until the caller supplies an
//! unpredictable 128-bit key with [`seed`].

#![no_std]
#![deny(missing_docs)]
#![deny(unsafe_code)]
#![deny(unsafe_op_in_unsafe_fn)]
#![allow(long_running_const_eval)]
#![doc = include_str!("../README.md")]

#[cfg(all(ja_colloquial_c_artifact, not(panic = "abort")))]
compile_error!("C artifacts require panic=abort; use `cargo rustc --profile capi ...`");

#[cfg(test)]
extern crate std;

use core::ffi::CStr;

mod random;
mod verses;

#[cfg(ja_colloquial_c_artifact)]
#[allow(unsafe_code)]
mod c_api;

pub use verses::BIBLE;

/// An immutable view of the generated Bible corpus.
#[derive(Clone, Copy, Debug)]
pub struct Bible {
    verses: &'static [Verse],
    books: &'static [Book],
    chapters: &'static [Chapter],
    book_names: &'static [&'static str],
}

#[derive(Clone, Copy, Debug)]
struct Book {
    first_chapter_idx: u16,
    chapter_count: u8,
}

#[derive(Clone, Copy, Debug)]
struct Chapter {
    first_verse_idx: u16,
    verse_count: u8,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
struct Text {
    rust: &'static str,
    c: &'static CStr,
}

impl Text {
    const fn new(c: &'static CStr) -> Self {
        let rust = match c.to_str() {
            Ok(text) => text,
            Err(_) => panic!("generated verse text is not UTF-8"),
        };
        Self { rust, c }
    }
}

/// One verse from the Japanese Colloquial Bible.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Verse {
    text: Text,
    book_idx: u8,
    chapter: u8,
    verse: u8,
}

impl Bible {
    /// Returns every verse record in canonical source order.
    pub fn verses(&self) -> &[Verse] {
        self.verses
    }

    /// Returns full Japanese book names in canonical order.
    ///
    /// Name lookup is exact UTF-8 equality. It performs no normalization and
    /// accepts no abbreviations or aliases.
    pub fn book_names(&self) -> &[&str] {
        self.book_names
    }

    /// Returns the number of chapters in a Japanese-named book.
    pub fn chapter_count(&self, book_name: &str) -> Option<usize> {
        self.book_index(book_name)
            .and_then(|index| self.chapter_count_by_index(index))
    }

    /// Returns every present verse record in a 1-based chapter.
    pub fn chapter(&self, book_name: &str, chapter: usize) -> Option<&[Verse]> {
        let book_idx = self.book_index(book_name)?;
        self.chapter_by_index(book_idx, chapter)
    }

    /// Returns the number of records present in a 1-based chapter.
    ///
    /// This is not necessarily the highest verse label because the source
    /// contains intentional numbering gaps.
    pub fn verse_count(&self, book_name: &str, chapter: usize) -> Option<usize> {
        self.chapter(book_name, chapter).map(<[Verse]>::len)
    }

    /// Looks up an exact 1-based chapter and verse number.
    ///
    /// Returns `None` for an unknown name, zero, an out-of-range value, or a
    /// verse number absent from the source.
    pub fn verse(&self, book_name: &str, chapter: usize, verse: usize) -> Option<Verse> {
        let book_idx = self.book_index(book_name)?;
        self.verse_by_index(book_idx, chapter, verse)
    }

    /// Uniformly selects one verse with the crate-global ChaCha20 generator.
    ///
    /// # Security
    ///
    /// Always call [`seed`] with an independently generated, unpredictable key
    /// whenever the target environment can provide one. The default stream is
    /// deterministic and publicly predictable.
    pub fn random_verse(&self) -> Verse {
        self.verses[random::random_index(self.verses.len())]
    }

    fn book_index(&self, book_name: &str) -> Option<usize> {
        let index = verses::book_index(book_name)?;
        (index < self.books.len()).then_some(index)
    }

    fn book(&self, book_idx: usize) -> Option<&Book> {
        self.books.get(book_idx)
    }

    pub(crate) fn chapter_count_by_index(&self, book_idx: usize) -> Option<usize> {
        self.book(book_idx)
            .map(|book| usize::from(book.chapter_count))
    }

    fn chapter_metadata(&self, book_idx: usize, chapter: usize) -> Option<&Chapter> {
        let book = self.book(book_idx)?;
        let offset = chapter.checked_sub(1)?;
        if offset >= usize::from(book.chapter_count) {
            return None;
        }
        self.chapters
            .get(usize::from(book.first_chapter_idx) + offset)
    }

    pub(crate) fn chapter_by_index(&self, book_idx: usize, chapter: usize) -> Option<&[Verse]> {
        let metadata = self.chapter_metadata(book_idx, chapter)?;
        let first = usize::from(metadata.first_verse_idx);
        let end = first + usize::from(metadata.verse_count);
        self.verses.get(first..end)
    }

    pub(crate) fn verse_by_index(
        &self,
        book_idx: usize,
        chapter: usize,
        verse: usize,
    ) -> Option<Verse> {
        if verse == 0 || verse > usize::from(u8::MAX) {
            return None;
        }
        let verses = self.chapter_by_index(book_idx, chapter)?;
        let number = verse as u8;
        let index = verses
            .binary_search_by_key(&number, |candidate| candidate.verse)
            .ok()?;
        verses.get(index).copied()
    }
}

impl Verse {
    /// Returns the full Japanese book name.
    pub fn book_name(&self) -> &'static str {
        verses::BOOK_NAMES[usize::from(self.book_idx)]
    }

    /// Returns the 1-based chapter number.
    pub fn chapter(&self) -> usize {
        usize::from(self.chapter)
    }

    /// Returns the 1-based verse number recorded by the source.
    pub fn number(&self) -> usize {
        usize::from(self.verse)
    }

    /// Returns the Japanese verse text.
    pub fn text(&self) -> &'static str {
        self.text.rust
    }

    #[cfg(ja_colloquial_c_artifact)]
    pub(crate) fn book_index(&self) -> usize {
        usize::from(self.book_idx)
    }

    #[cfg(any(test, ja_colloquial_c_artifact))]
    pub(crate) fn c_text(&self) -> &'static CStr {
        self.text.c
    }
}

/// Installs the crate-global 128-bit random key if it is still unset.
///
/// Returns `true` when this call installed the key. Once any call succeeds,
/// every later call is a no-op and returns `false`. The key bytes are
/// interpreted as four little-endian words. Call this as early as possible so
/// another caller cannot install a weaker key first.
#[must_use = "check whether this call installed the one-time random seed"]
pub fn seed(seed: [u8; 16]) -> bool {
    random::seed(seed)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn corpus_and_lookup() {
        assert_eq!(BIBLE.book_names().len(), 66);
        assert_eq!(BIBLE.verses().len(), 31_086);
        assert_eq!(BIBLE.book_names().first(), Some(&"創世記"));
        assert_eq!(BIBLE.book_names().get(16), Some(&"エステル記"));
        assert_eq!(BIBLE.chapter_count("詩篇"), Some(150));

        let verse = BIBLE.verse("創世記", 4, 13).expect("Genesis 4:13");
        assert_eq!(verse.book_name(), "創世記");
        assert_eq!(verse.chapter(), 4);
        assert_eq!(verse.number(), 13);
        assert_eq!(
            verse.text(),
            "カインは主に言った、「わたしの罰は重くて負いきれません。"
        );
        assert_eq!(verse.c_text().to_bytes(), verse.text().as_bytes());

        assert_eq!(BIBLE.verse("民数記", 15, 4), None);
        assert_eq!(
            BIBLE.verse("民数記", 15, 5).map(|item| item.number()),
            Some(5)
        );
        assert_eq!(BIBLE.verse_count("民数記", 15), Some(40));
        assert_eq!(BIBLE.verse("ge", 1, 1), None);
        assert_eq!(BIBLE.verse("創世記", 0, 1), None);
        assert_eq!(BIBLE.verse("創世記", 1, 0), None);
        assert_eq!(BIBLE.verse("創世記", usize::MAX, 1), None);
        assert_eq!(BIBLE.verse("創世記", 1, usize::MAX), None);
    }
}
