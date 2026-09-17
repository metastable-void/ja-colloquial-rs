# Changelog

## 0.2.0

- Reworked the crate as a dependency-free, allocation-free `no_std` library.
- Replaced runtime JSON parsing with validated build-time generation.
- Added the `BIBLE` API with full Japanese book-name lookup and private verse
  representation.
- Added a concurrency-safe ChaCha20 random verse selector with one-time
  explicit seeding.
- Added a `std` command-line example using the dev-only `getrandom` dependency.
- Added an in-tree `libexecinfo` fallback for cross-built NetBSD executables.
- Added conditionally built C `staticlib` and `cdylib` artifacts with a C99/C++
  header.
- Corrected the Japanese name of Esther and relicensed the project under
  CC0-1.0.

## 0.1.0

- Initial Rust release with runtime JSON loading, abbreviated book lookup, and
  random verse selection through `rand`.
