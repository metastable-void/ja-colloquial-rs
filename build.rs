use std::collections::HashSet;
use std::env;
use std::fmt::Write as _;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

const EXPECTED_VERSES: usize = 31_086;
const EXPECTED_CHAPTERS: usize = 1_189;

const BOOKS_SPEC: [(&str, &str); 66] = [
    ("ge", "GENESIS"),
    ("exo", "EXODUS"),
    ("lev", "LEVITICUS"),
    ("num", "NUMBERS"),
    ("deu", "DEUTERONOMY"),
    ("josh", "JOSHUA"),
    ("jdgs", "JUDGES"),
    ("ruth", "RUTH"),
    ("1sm", "FIRST_SAMUEL"),
    ("2sm", "SECOND_SAMUEL"),
    ("1ki", "FIRST_KINGS"),
    ("2ki", "SECOND_KINGS"),
    ("1chr", "FIRST_CHRONICLES"),
    ("2chr", "SECOND_CHRONICLES"),
    ("ezra", "EZRA"),
    ("neh", "NEHEMIAH"),
    ("est", "ESTHER"),
    ("job", "JOB"),
    ("psa", "PSALMS"),
    ("prv", "PROVERBS"),
    ("eccl", "ECCLESIASTES"),
    ("ssol", "SONG_OF_SOLOMON"),
    ("isa", "ISAIAH"),
    ("jer", "JEREMIAH"),
    ("lam", "LAMENTATIONS"),
    ("eze", "EZEKIEL"),
    ("dan", "DANIEL"),
    ("hos", "HOSEA"),
    ("joel", "JOEL"),
    ("amos", "AMOS"),
    ("obad", "OBADIAH"),
    ("jonah", "JONAH"),
    ("mic", "MICAH"),
    ("nahum", "NAHUM"),
    ("hab", "HABAKKUK"),
    ("zep", "ZEPHANIAH"),
    ("hag", "HAGGAI"),
    ("zec", "ZECHARIAH"),
    ("mal", "MALACHI"),
    ("mat", "MATTHEW"),
    ("mark", "MARK"),
    ("luke", "LUKE"),
    ("john", "JOHN"),
    ("acts", "ACTS"),
    ("rom", "ROMANS"),
    ("1cor", "FIRST_CORINTHIANS"),
    ("2cor", "SECOND_CORINTHIANS"),
    ("gal", "GALATIANS"),
    ("eph", "EPHESIANS"),
    ("phi", "PHILIPPIANS"),
    ("col", "COLOSSIANS"),
    ("1th", "FIRST_THESSALONIANS"),
    ("2th", "SECOND_THESSALONIANS"),
    ("1tim", "FIRST_TIMOTHY"),
    ("2tim", "SECOND_TIMOTHY"),
    ("titus", "TITUS"),
    ("phmn", "PHILEMON"),
    ("heb", "HEBREWS"),
    ("jas", "JAMES"),
    ("1pet", "FIRST_PETER"),
    ("2pet", "SECOND_PETER"),
    ("1jn", "FIRST_JOHN"),
    ("2jn", "SECOND_JOHN"),
    ("3jn", "THIRD_JOHN"),
    ("jude", "JUDE"),
    ("rev", "REVELATION"),
];

#[derive(Debug)]
struct RawVerse {
    book_id: String,
    chapter: u64,
    verse: u64,
    book_name: String,
    text: String,
}

struct Parser<'a> {
    input: &'a str,
    bytes: &'a [u8],
    position: usize,
    line: usize,
    column: usize,
}

impl<'a> Parser<'a> {
    fn new(input: &'a str) -> Self {
        Self {
            input,
            bytes: input.as_bytes(),
            position: 0,
            line: 1,
            column: 1,
        }
    }

    fn parse(mut self) -> Result<Vec<RawVerse>, String> {
        self.skip_whitespace();
        self.expect_byte(b'[', "expected a top-level JSON array")?;
        self.skip_whitespace();

        let mut verses = Vec::new();
        if self.peek() == Some(b']') {
            self.bump();
        } else {
            loop {
                verses.push(self.parse_verse()?);
                self.skip_whitespace();
                match self.peek() {
                    Some(b',') => {
                        self.bump();
                        self.skip_whitespace();
                    }
                    Some(b']') => {
                        self.bump();
                        break;
                    }
                    _ => return Err(self.error("expected `,` or `]` after array element")),
                }
            }
        }

        self.skip_whitespace();
        if self.peek().is_some() {
            return Err(self.error("unexpected data after the top-level array"));
        }
        Ok(verses)
    }

    fn parse_verse(&mut self) -> Result<RawVerse, String> {
        self.expect_byte(b'{', "expected a verse object")?;
        self.skip_whitespace();

        let mut book_id = None;
        let mut chapter = None;
        let mut verse = None;
        let mut book_name = None;
        let mut text = None;

        if self.peek() == Some(b'}') {
            self.bump();
        } else {
            loop {
                let key = self.parse_string()?;
                self.skip_whitespace();
                self.expect_byte(b':', "expected `:` after object key")?;
                self.skip_whitespace();

                match key.as_str() {
                    "b" => Self::set_once(&mut book_id, self.parse_string()?, "b")?,
                    "c" => Self::set_once(&mut chapter, self.parse_u64()?, "c")?,
                    "v" => Self::set_once(&mut verse, self.parse_u64()?, "v")?,
                    "jb" => Self::set_once(&mut book_name, self.parse_string()?, "jb")?,
                    "t" => Self::set_once(&mut text, self.parse_string()?, "t")?,
                    _ => return Err(self.error(&format!("unknown field {key:?}"))),
                }

                self.skip_whitespace();
                match self.peek() {
                    Some(b',') => {
                        self.bump();
                        self.skip_whitespace();
                    }
                    Some(b'}') => {
                        self.bump();
                        break;
                    }
                    _ => return Err(self.error("expected `,` or `}` after object field")),
                }
            }
        }

        Ok(RawVerse {
            book_id: book_id.ok_or_else(|| self.error("missing field `b`"))?,
            chapter: chapter.ok_or_else(|| self.error("missing field `c`"))?,
            verse: verse.ok_or_else(|| self.error("missing field `v`"))?,
            book_name: book_name.ok_or_else(|| self.error("missing field `jb`"))?,
            text: text.ok_or_else(|| self.error("missing field `t`"))?,
        })
    }

    fn set_once<T>(slot: &mut Option<T>, value: T, name: &str) -> Result<(), String> {
        if slot.replace(value).is_some() {
            return Err(format!("duplicate field `{name}`"));
        }
        Ok(())
    }

    fn parse_string(&mut self) -> Result<String, String> {
        self.expect_byte(b'"', "expected a JSON string")?;
        let mut output = String::new();

        loop {
            let byte = self
                .peek()
                .ok_or_else(|| self.error("unterminated JSON string"))?;
            match byte {
                b'"' => {
                    self.bump();
                    return Ok(output);
                }
                b'\\' => {
                    self.bump();
                    let escape = self
                        .bump()
                        .ok_or_else(|| self.error("unterminated JSON escape"))?;
                    match escape {
                        b'"' => output.push('"'),
                        b'\\' => output.push('\\'),
                        b'/' => output.push('/'),
                        b'b' => output.push('\u{0008}'),
                        b'f' => output.push('\u{000c}'),
                        b'n' => output.push('\n'),
                        b'r' => output.push('\r'),
                        b't' => output.push('\t'),
                        b'u' => output.push(self.parse_unicode_escape()?),
                        _ => return Err(self.error("unsupported JSON escape")),
                    }
                }
                0x00..=0x1f => {
                    return Err(self.error("unescaped control character in JSON string"));
                }
                0x20..=0x7f => {
                    output.push(char::from(byte));
                    self.bump();
                }
                _ => {
                    let rest = &self.input[self.position..];
                    let character = rest
                        .chars()
                        .next()
                        .ok_or_else(|| self.error("invalid UTF-8 in JSON string"))?;
                    output.push(character);
                    for _ in 0..character.len_utf8() {
                        self.bump();
                    }
                }
            }
        }
    }

    fn parse_unicode_escape(&mut self) -> Result<char, String> {
        let first = self.parse_hex_quad()?;
        let scalar = if (0xd800..=0xdbff).contains(&first) {
            if self.bump() != Some(b'\\') || self.bump() != Some(b'u') {
                return Err(self.error("high surrogate is not followed by a low surrogate"));
            }
            let second = self.parse_hex_quad()?;
            if !(0xdc00..=0xdfff).contains(&second) {
                return Err(self.error("invalid low surrogate"));
            }
            0x1_0000 + ((u32::from(first) - 0xd800) << 10) + (u32::from(second) - 0xdc00)
        } else if (0xdc00..=0xdfff).contains(&first) {
            return Err(self.error("low surrogate without a preceding high surrogate"));
        } else {
            u32::from(first)
        };

        char::from_u32(scalar).ok_or_else(|| self.error("invalid Unicode scalar value"))
    }

    fn parse_hex_quad(&mut self) -> Result<u16, String> {
        let mut value = 0_u16;
        for _ in 0..4 {
            let byte = self
                .bump()
                .ok_or_else(|| self.error("incomplete Unicode escape"))?;
            let digit = match byte {
                b'0'..=b'9' => u16::from(byte - b'0'),
                b'a'..=b'f' => u16::from(byte - b'a' + 10),
                b'A'..=b'F' => u16::from(byte - b'A' + 10),
                _ => return Err(self.error("invalid hexadecimal digit in Unicode escape")),
            };
            value = (value << 4) | digit;
        }
        Ok(value)
    }

    fn parse_u64(&mut self) -> Result<u64, String> {
        let start = self.position;
        match self.peek() {
            Some(b'0') => {
                self.bump();
                if matches!(self.peek(), Some(b'0'..=b'9')) {
                    return Err(self.error("leading zero in JSON number"));
                }
            }
            Some(b'1'..=b'9') => {
                while matches!(self.peek(), Some(b'0'..=b'9')) {
                    self.bump();
                }
            }
            _ => return Err(self.error("expected a non-negative JSON integer")),
        }

        if matches!(self.peek(), Some(b'.' | b'e' | b'E')) {
            return Err(self.error("chapter and verse values must be integers"));
        }
        self.input[start..self.position]
            .parse()
            .map_err(|_| self.error("integer does not fit in u64"))
    }

    fn skip_whitespace(&mut self) {
        while matches!(self.peek(), Some(b' ' | b'\n' | b'\r' | b'\t')) {
            self.bump();
        }
    }

    fn expect_byte(&mut self, expected: u8, message: &str) -> Result<(), String> {
        if self.bump() == Some(expected) {
            Ok(())
        } else {
            Err(self.error(message))
        }
    }

    fn peek(&self) -> Option<u8> {
        self.bytes.get(self.position).copied()
    }

    fn bump(&mut self) -> Option<u8> {
        let byte = self.peek()?;
        self.position += 1;
        if byte == b'\n' {
            self.line += 1;
            self.column = 1;
        } else {
            self.column += 1;
        }
        Some(byte)
    }

    fn error(&self, message: &str) -> String {
        format!(
            "src/books.json:{}:{} (byte {}): {message}",
            self.line, self.column, self.position
        )
    }
}

#[derive(Clone, Copy)]
struct GeneratedBook {
    first_chapter_idx: u16,
    chapter_count: u8,
}

#[derive(Clone, Copy)]
struct GeneratedChapter {
    first_verse_idx: u16,
    verse_count: u8,
}

struct GeneratedData {
    verses: Vec<RawVerse>,
    books: Vec<GeneratedBook>,
    chapters: Vec<GeneratedChapter>,
    book_names: Vec<String>,
}

fn validate(verses: Vec<RawVerse>) -> Result<GeneratedData, String> {
    if verses.is_empty() {
        return Err("src/books.json contains no verses".to_owned());
    }
    if verses.len() != EXPECTED_VERSES {
        return Err(format!(
            "expected {EXPECTED_VERSES} verses, found {}",
            verses.len()
        ));
    }

    let mut books = Vec::<GeneratedBook>::new();
    let mut chapters = Vec::<GeneratedChapter>::new();
    let mut book_names = Vec::<String>::new();
    let mut seen_ids = HashSet::<String>::new();
    let mut seen_names = HashSet::<String>::new();
    let mut current_id = String::new();
    let mut current_name = String::new();
    let mut current_chapter = 0_u8;
    let mut previous_verse = 0_u8;

    for (index, raw) in verses.iter().enumerate() {
        if raw.book_id.is_empty() {
            return Err(format!("verse record {index} has an empty book ID"));
        }
        if raw.book_name.is_empty() {
            return Err(format!(
                "verse record {index} has an empty Japanese book name"
            ));
        }
        if raw.text.is_empty() {
            return Err(format!("verse record {index} has empty text"));
        }
        if raw.book_name.contains('\0') || raw.text.contains('\0') {
            return Err(format!("verse record {index} contains an embedded NUL"));
        }

        let chapter = u8::try_from(raw.chapter)
            .map_err(|_| format!("chapter {} does not fit in u8", raw.chapter))?;
        let verse = u8::try_from(raw.verse)
            .map_err(|_| format!("verse {} does not fit in u8", raw.verse))?;
        if chapter == 0 || verse == 0 {
            return Err(format!("verse record {index} uses chapter or verse zero"));
        }

        if raw.book_id != current_id {
            if !seen_ids.insert(raw.book_id.clone()) {
                return Err(format!(
                    "book ID {:?} reappears after its contiguous group",
                    raw.book_id
                ));
            }
            if books.len() >= BOOKS_SPEC.len() {
                return Err(format!("unexpected extra book ID {:?}", raw.book_id));
            }
            let expected_id = BOOKS_SPEC[books.len()].0;
            if raw.book_id != expected_id {
                return Err(format!(
                    "book {} must be ID {expected_id:?}, found {:?}",
                    books.len(),
                    raw.book_id
                ));
            }
            if !seen_names.insert(raw.book_name.clone()) {
                return Err(format!(
                    "Japanese book name {:?} is not unique",
                    raw.book_name
                ));
            }

            if let Some(previous) = books.last_mut() {
                let count = chapters.len() - usize::from(previous.first_chapter_idx);
                previous.chapter_count = u8::try_from(count)
                    .map_err(|_| "chapter count does not fit in u8".to_owned())?;
            }

            books.push(GeneratedBook {
                first_chapter_idx: u16::try_from(chapters.len())
                    .map_err(|_| "chapter index does not fit in u16".to_owned())?,
                chapter_count: 0,
            });
            book_names.push(raw.book_name.clone());
            current_id.clone_from(&raw.book_id);
            current_name.clone_from(&raw.book_name);
            current_chapter = 0;
            previous_verse = 0;
        } else if raw.book_name != current_name {
            return Err(format!(
                "book ID {:?} uses inconsistent Japanese names {:?} and {:?}",
                raw.book_id, current_name, raw.book_name
            ));
        }

        if chapter != current_chapter {
            let expected = current_chapter
                .checked_add(1)
                .ok_or_else(|| "chapter number overflow".to_owned())?;
            if chapter != expected {
                return Err(format!(
                    "book {:?} expected chapter {expected}, found {chapter}",
                    raw.book_id
                ));
            }
            chapters.push(GeneratedChapter {
                first_verse_idx: u16::try_from(index)
                    .map_err(|_| "verse index does not fit in u16".to_owned())?,
                verse_count: 0,
            });
            current_chapter = chapter;
            previous_verse = 0;
        }

        if verse <= previous_verse {
            return Err(format!(
                "book {:?} chapter {chapter} has non-increasing verse {verse}",
                raw.book_id
            ));
        }
        previous_verse = verse;
        let chapter_meta = chapters
            .last_mut()
            .ok_or_else(|| "internal error: missing chapter".to_owned())?;
        chapter_meta.verse_count = chapter_meta
            .verse_count
            .checked_add(1)
            .ok_or_else(|| "chapter record count does not fit in u8".to_owned())?;
    }

    let last_book = books
        .last_mut()
        .ok_or_else(|| "internal error: missing book".to_owned())?;
    let count = chapters.len() - usize::from(last_book.first_chapter_idx);
    last_book.chapter_count =
        u8::try_from(count).map_err(|_| "chapter count does not fit in u8".to_owned())?;

    if books.len() != BOOKS_SPEC.len() {
        return Err(format!(
            "expected {} books, found {}",
            BOOKS_SPEC.len(),
            books.len()
        ));
    }
    if chapters.len() != EXPECTED_CHAPTERS {
        return Err(format!(
            "expected {EXPECTED_CHAPTERS} chapters, found {}",
            chapters.len()
        ));
    }

    Ok(GeneratedData {
        verses,
        books,
        chapters,
        book_names,
    })
}

fn generate_rust(data: &GeneratedData) -> Result<String, std::fmt::Error> {
    let mut output = String::with_capacity(10_000_000);
    writeln!(
        output,
        "pub(crate) static VERSES: [Verse; {}] = [",
        data.verses.len()
    )?;
    for (index, verse) in data.verses.iter().enumerate() {
        let book_idx = data.books[..].partition_point(|book| {
            usize::from(book.first_chapter_idx) <= chapter_index(data, index)
        });
        let book_idx = book_idx.saturating_sub(1);
        writeln!(
            output,
            "    Verse {{ text: Text::new(c{:?}), book_idx: {book_idx}, chapter: {}, verse: {} }},",
            verse.text, verse.chapter, verse.verse
        )?;
    }
    writeln!(output, "];")?;

    writeln!(
        output,
        "pub(crate) static BOOKS: [Book; {}] = [",
        data.books.len()
    )?;
    for book in &data.books {
        writeln!(
            output,
            "    Book {{ first_chapter_idx: {}, chapter_count: {} }},",
            book.first_chapter_idx, book.chapter_count
        )?;
    }
    writeln!(output, "];")?;

    writeln!(
        output,
        "pub(crate) static CHAPTERS: [Chapter; {}] = [",
        data.chapters.len()
    )?;
    for chapter in &data.chapters {
        writeln!(
            output,
            "    Chapter {{ first_verse_idx: {}, verse_count: {} }},",
            chapter.first_verse_idx, chapter.verse_count
        )?;
    }
    writeln!(output, "];")?;

    writeln!(
        output,
        "pub(crate) static BOOK_NAMES: [&str; {}] = [",
        data.book_names.len()
    )?;
    for name in &data.book_names {
        writeln!(output, "    {name:?},")?;
    }
    writeln!(output, "];")?;

    writeln!(
        output,
        "/// The complete Japanese Colloquial Bible corpus.\npub const BIBLE: Bible = Bible {{"
    )?;
    writeln!(output, "    verses: &VERSES,")?;
    writeln!(output, "    books: &BOOKS,")?;
    writeln!(output, "    chapters: &CHAPTERS,")?;
    writeln!(output, "    book_names: &BOOK_NAMES,")?;
    writeln!(output, "}};")?;

    writeln!(
        output,
        "pub(crate) fn book_index(name: &str) -> Option<usize> {{"
    )?;
    writeln!(output, "    match name {{")?;
    for (index, name) in data.book_names.iter().enumerate() {
        writeln!(output, "        {name:?} => Some({index}),")?;
    }
    writeln!(output, "        _ => None,")?;
    writeln!(output, "    }}")?;
    writeln!(output, "}}")?;

    Ok(output)
}

fn chapter_index(data: &GeneratedData, verse_index: usize) -> usize {
    data.chapters
        .partition_point(|chapter| usize::from(chapter.first_verse_idx) <= verse_index)
        .saturating_sub(1)
}

fn generate_header() -> Result<String, std::fmt::Error> {
    let mut output = String::new();
    writeln!(output, "#ifndef JA_COLLOQUIAL_H")?;
    writeln!(output, "#define JA_COLLOQUIAL_H")?;
    writeln!(output)?;
    writeln!(output, "#include <stdint.h>")?;
    writeln!(output)?;
    writeln!(output, "/* A zero-based canonical book index. */")?;
    writeln!(output, "typedef int ja_colloquial_book_t;")?;
    writeln!(output)?;
    for (index, (_, suffix)) in BOOKS_SPEC.iter().enumerate() {
        writeln!(
            output,
            "#define JA_COLLOQUIAL_BOOK_{suffix} ((ja_colloquial_book_t){index})"
        )?;
    }
    writeln!(output, "#define JA_COLLOQUIAL_BOOK_COUNT 66")?;
    writeln!(output)?;
    writeln!(
        output,
        "/* A verse returned by value. text is static UTF-8, is NUL-terminated,"
    )?;
    writeln!(output, " * and must not be freed or modified. */")?;
    writeln!(output, "typedef struct ja_colloquial_verse {{")?;
    writeln!(output, "    ja_colloquial_book_t book;")?;
    writeln!(output, "    int chapter;")?;
    writeln!(output, "    int verse;")?;
    writeln!(output, "    const char *text;")?;
    writeln!(output, "}} ja_colloquial_verse_t;")?;
    writeln!(output)?;
    writeln!(output, "#ifdef __cplusplus")?;
    writeln!(output, "extern \"C\" {{")?;
    writeln!(output, "#endif")?;
    writeln!(output)?;
    writeln!(
        output,
        "/* Installs the process-wide 128-bit key only if it is still unset."
    )?;
    writeln!(
        output,
        " * The key is little-endian a followed by little-endian b."
    )?;
    writeln!(
        output,
        " * Call this as early as possible with unpredictable values."
    )?;
    writeln!(
        output,
        " * Returns 0 when this call installs the key, or -1 if already seeded. */"
    )?;
    writeln!(output, "int ja_colloquial_seed(uint64_t a, uint64_t b);")?;
    writeln!(output)?;
    writeln!(output, "/* Returns JA_COLLOQUIAL_BOOK_COUNT. */")?;
    writeln!(output, "int ja_colloquial_book_count(void);")?;
    writeln!(output)?;
    writeln!(
        output,
        "/* Returns the chapter count, or -1 for an invalid book. */"
    )?;
    writeln!(
        output,
        "int ja_colloquial_book_chapters(ja_colloquial_book_t book);"
    )?;
    writeln!(output)?;
    writeln!(
        output,
        "/* Returns the count of present verse records, or -1 for invalid input."
    )?;
    writeln!(
        output,
        " * The count can differ from the highest label where the source has gaps. */"
    )?;
    writeln!(
        output,
        "int ja_colloquial_chapter_verses(ja_colloquial_book_t book, int chapter);"
    )?;
    writeln!(output)?;
    writeln!(
        output,
        "/* Returns the requested verse, or {{-1, 0, 0, NULL}} for invalid input. */"
    )?;
    writeln!(output, "ja_colloquial_verse_t ja_colloquial_get_verse(")?;
    writeln!(output, "    ja_colloquial_book_t book,")?;
    writeln!(output, "    int chapter,")?;
    writeln!(output, "    int verse")?;
    writeln!(output, ");")?;
    writeln!(output)?;
    writeln!(
        output,
        "/* Returns a uniformly selected verse. The unseeded stream is deterministic"
    )?;
    writeln!(
        output,
        " * and publicly predictable; call ja_colloquial_seed as early as possible. */"
    )?;
    writeln!(
        output,
        "ja_colloquial_verse_t ja_colloquial_random_verse(void);"
    )?;
    writeln!(output)?;
    writeln!(output, "#ifdef __cplusplus")?;
    writeln!(output, "}}")?;
    writeln!(output, "#endif")?;
    writeln!(output)?;
    writeln!(output, "#endif")?;
    Ok(output)
}

fn compare_header(path: &Path, generated: &str, generated_path: &Path) -> Result<(), String> {
    let checked_in = fs::read_to_string(path).map_err(|error| {
        format!(
            "cannot read {}: {error}; generated header is at {}",
            path.display(),
            generated_path.display()
        )
    })?;
    if checked_in != generated {
        return Err(format!(
            "{} is stale; replace it with {}",
            path.display(),
            generated_path.display()
        ));
    }
    Ok(())
}

fn target_tool(variable: &str, target: &str, default_suffix: &str) -> std::ffi::OsString {
    let target_variable = format!("{variable}_{target}");
    let underscored_variable = target_variable.replace('-', "_");
    let cargo_variable = format!("TARGET_{variable}");
    env::var_os(&target_variable)
        .or_else(|| env::var_os(underscored_variable))
        .or_else(|| env::var_os(cargo_variable))
        .or_else(|| env::var_os(variable))
        .unwrap_or_else(|| format!("{target}-{default_suffix}").into())
}

fn run_tool(command: &mut Command, description: &str) -> Result<(), String> {
    let status = command
        .status()
        .map_err(|error| format!("cannot run {description}: {error}"))?;
    if !status.success() {
        return Err(format!("{description} failed with {status}"));
    }
    Ok(())
}

fn build_netbsd_execinfo_stub(out_dir: &Path) -> Result<(), String> {
    let target =
        env::var("TARGET").map_err(|error| format!("Cargo did not set TARGET: {error}"))?;
    if !target.ends_with("-unknown-netbsd") {
        return Ok(());
    }

    let source = Path::new("src/netbsd_execinfo_stub.c");
    let object = out_dir.join("netbsd_execinfo_stub.o");
    let archive = out_dir.join("libexecinfo.a");
    let compiler = target_tool("CC", &target, "gcc");
    let archiver = target_tool("AR", &target, "ar");

    run_tool(
        Command::new(compiler)
            .arg("-c")
            .arg("-fPIC")
            .arg(source)
            .arg("-o")
            .arg(&object),
        "NetBSD C compiler for the libexecinfo stub",
    )?;
    run_tool(
        Command::new(archiver).arg("crs").arg(&archive).arg(&object),
        "NetBSD archiver for the libexecinfo stub",
    )?;

    println!("cargo::rustc-link-search=native={}", out_dir.display());
    println!("cargo::rustc-link-lib=static=execinfo");
    Ok(())
}

fn run() -> Result<(), String> {
    println!("cargo::rerun-if-changed=src/books.json");
    println!("cargo::rerun-if-changed=src/netbsd_execinfo_stub.c");
    println!("cargo::rerun-if-changed=include/ja_colloquial.h");
    println!("cargo::rustc-check-cfg=cfg(ja_colloquial_c_artifact)");

    let json = fs::read_to_string("src/books.json")
        .map_err(|error| format!("cannot read src/books.json: {error}"))?;
    let parsed = Parser::new(&json).parse()?;
    let data = validate(parsed)?;

    let out_dir = PathBuf::from(
        env::var_os("OUT_DIR").ok_or_else(|| "Cargo did not set OUT_DIR".to_owned())?,
    );
    build_netbsd_execinfo_stub(&out_dir)?;
    let rust = generate_rust(&data).map_err(|error| error.to_string())?;
    fs::write(out_dir.join("verses.rs"), rust)
        .map_err(|error| format!("cannot write generated verses: {error}"))?;

    let header = generate_header().map_err(|error| error.to_string())?;
    let generated_header = out_dir.join("ja_colloquial.h");
    fs::write(&generated_header, &header)
        .map_err(|error| format!("cannot write generated C header: {error}"))?;
    compare_header(
        Path::new("include/ja_colloquial.h"),
        &header,
        &generated_header,
    )
}

fn main() {
    if let Err(error) = run() {
        panic!("{error}");
    }
}
