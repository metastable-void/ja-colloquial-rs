use std::env;
use std::process::ExitCode;

use ja_colloquial::{BIBLE, Verse, seed};

const USAGE: &str = "usage:\n  ja-colloquial --rand|-r\n  ja-colloquial --list-books\n  ja-colloquial <BOOK> <CHAPTER>:<VERSE>";

fn print_verse(verse: Verse) {
    println!(
        "{} {}:{} {}",
        verse.book_name(),
        verse.chapter(),
        verse.number(),
        verse.text()
    );
}

fn parse_reference(reference: &str) -> Result<(usize, usize), String> {
    let (chapter, verse) = reference
        .split_once(':')
        .ok_or_else(|| format!("invalid reference {reference:?}; expected CHAPTER:VERSE"))?;
    let chapter = chapter
        .parse()
        .map_err(|_| format!("invalid chapter number {chapter:?}"))?;
    let verse = verse
        .parse()
        .map_err(|_| format!("invalid verse number {verse:?}"))?;
    Ok((chapter, verse))
}

fn run() -> Result<(), String> {
    let arguments: Vec<_> = env::args().skip(1).collect();
    match arguments.as_slice() {
        [option] if option == "--rand" || option == "-r" => {
            let mut key = [0_u8; 16];
            getrandom::fill(&mut key)
                .map_err(|error| format!("could not obtain random seed: {error}"))?;
            if !seed(key) {
                return Err("the process-wide random generator was already seeded".to_owned());
            }
            print_verse(BIBLE.random_verse());
            Ok(())
        }
        [option] if option == "--list-books" => {
            for name in BIBLE.book_names() {
                println!("{name}");
            }
            Ok(())
        }
        [option] if option == "--help" || option == "-h" => {
            println!("{USAGE}");
            Ok(())
        }
        [book, reference] => {
            let (chapter, verse) = parse_reference(reference)?;
            let verse = BIBLE
                .verse(book, chapter, verse)
                .ok_or_else(|| format!("verse not found: {book} {chapter}:{verse}"))?;
            print_verse(verse);
            Ok(())
        }
        _ => Err(USAGE.to_owned()),
    }
}

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("ja-colloquial: {error}");
            ExitCode::FAILURE
        }
    }
}
