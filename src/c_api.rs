use core::ffi::{c_char, c_int};

use crate::{BIBLE, Verse};

#[repr(C)]
#[derive(Clone, Copy)]
struct CVerse {
    book: c_int,
    chapter: c_int,
    verse: c_int,
    text: *const c_char,
}

impl CVerse {
    const INVALID: Self = Self {
        book: -1,
        chapter: 0,
        verse: 0,
        text: core::ptr::null(),
    };
}

fn to_c_verse(verse: Verse) -> CVerse {
    CVerse {
        book: verse.book_index() as c_int,
        chapter: verse.chapter() as c_int,
        verse: verse.number() as c_int,
        text: verse.c_text().as_ptr(),
    }
}

fn book_index(book: c_int) -> Option<usize> {
    let index = usize::try_from(book).ok()?;
    (index < BIBLE.book_names().len()).then_some(index)
}

fn positive(value: c_int) -> Option<usize> {
    let value = usize::try_from(value).ok()?;
    (value != 0).then_some(value)
}

// SAFETY: the globally unique project prefix makes the unmangled symbol an
// intentional part of the C ABI and prevents collisions with generic names.
#[unsafe(no_mangle)]
extern "C" fn ja_colloquial_seed(a: u64, b: u64) {
    let mut bytes = [0_u8; 16];
    bytes[..8].copy_from_slice(&a.to_le_bytes());
    bytes[8..].copy_from_slice(&b.to_le_bytes());
    crate::seed(bytes);
}

// SAFETY: the globally unique project prefix makes the unmangled symbol an
// intentional part of the C ABI and prevents collisions with generic names.
#[unsafe(no_mangle)]
extern "C" fn ja_colloquial_book_count() -> c_int {
    BIBLE.book_names().len() as c_int
}

// SAFETY: the globally unique project prefix makes the unmangled symbol an
// intentional part of the C ABI and prevents collisions with generic names.
#[unsafe(no_mangle)]
extern "C" fn ja_colloquial_book_chapters(book: c_int) -> c_int {
    book_index(book)
        .and_then(|index| BIBLE.chapter_count_by_index(index))
        .map_or(-1, |count| count as c_int)
}

// SAFETY: the globally unique project prefix makes the unmangled symbol an
// intentional part of the C ABI and prevents collisions with generic names.
#[unsafe(no_mangle)]
extern "C" fn ja_colloquial_chapter_verses(book: c_int, chapter: c_int) -> c_int {
    let Some((book, chapter)) = book_index(book).zip(positive(chapter)) else {
        return -1;
    };
    BIBLE
        .chapter_by_index(book, chapter)
        .map_or(-1, |verses| verses.len() as c_int)
}

// SAFETY: the globally unique project prefix makes the unmangled symbol an
// intentional part of the C ABI and prevents collisions with generic names.
#[unsafe(no_mangle)]
extern "C" fn ja_colloquial_get_verse(book: c_int, chapter: c_int, verse: c_int) -> CVerse {
    let Some((book, chapter, verse)) = book_index(book)
        .zip(positive(chapter))
        .zip(positive(verse))
        .map(|((book, chapter), verse)| (book, chapter, verse))
    else {
        return CVerse::INVALID;
    };
    BIBLE
        .verse_by_index(book, chapter, verse)
        .map_or(CVerse::INVALID, to_c_verse)
}

// SAFETY: the globally unique project prefix makes the unmangled symbol an
// intentional part of the C ABI and prevents collisions with generic names.
#[unsafe(no_mangle)]
extern "C" fn ja_colloquial_random_verse() -> CVerse {
    to_c_verse(BIBLE.random_verse())
}

// SAFETY: stable precompiled `core` may reference this conventional runtime
// symbol even when the final artifact uses panic=abort. It is not a public API.
#[unsafe(no_mangle)]
extern "C" fn rust_eh_personality() {}

#[cfg(not(test))]
#[panic_handler]
fn panic(_: &core::panic::PanicInfo<'_>) -> ! {
    loop {
        core::hint::spin_loop();
    }
}
