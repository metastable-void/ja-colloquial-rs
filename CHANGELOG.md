# Changelog

## 0.2.0

- Reworked the crate as a dependency-free, allocation-free `no_std` library.
- Replaced runtime JSON parsing with validated build-time generation.
- Added the `BIBLE` API with full Japanese book-name lookup and private verse
  representation.
- Added a concurrency-safe, explicitly seeded ChaCha20 random verse selector.
- Added conditionally built C `staticlib` and `cdylib` artifacts with a C99/C++
  header.
- Corrected the Japanese name of Esther and relicensed the project under
  CC0-1.0.

## 0.1.0

- Initial Rust release with runtime JSON loading, abbreviated book lookup, and
  random verse selection through `rand`.
