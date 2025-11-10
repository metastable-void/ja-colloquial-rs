# Japanese Colloquial Christian Bible Library (Rust)

## Usage
```rust
use ja_colloquial::books;

let books = books();

assert!(books.book_names.contains(&"ge".to_string()));
assert_eq!(
    books.get_verse("ge", 4, 13).unwrap().t,
    "カインは主に言った、「わたしの罰は重くて負いきれません。",
);

println!("{:?}", books.random_verse());
```
