# Japanese Colloquial Christian Bible

An allocation-free, dependency-free `no_std` Rust library containing the
Japanese Colloquial Bible.

> [!WARNING]
> Call `seed` as early as possible with an independently generated,
> unpredictable 16-byte key whenever the target environment can provide one.
> Only the first call can install a key. The default random stream is
> deterministic and publicly predictable; it is not suitable for
> security-sensitive use.

## Rust usage

```rust
use ja_colloquial::{BIBLE, seed};

assert_eq!(BIBLE.book_names().first(), Some(&"創世記"));

let verse = BIBLE.verse("創世記", 4, 13).unwrap();
assert_eq!(verse.book_name(), "創世記");
assert_eq!(verse.chapter(), 4);
assert_eq!(verse.number(), 13);
assert_eq!(
    verse.text(),
    "カインは主に言った、「わたしの罰は重くて負いきれません。",
);

// Demonstration only: obtain unpredictable bytes from the target platform.
assert!(seed([42; 16]));
let random = BIBLE.random_verse();
assert!(!random.text().is_empty());
```

Book lookup accepts the exact full Japanese names returned by
`BIBLE.book_names()`. Source-data abbreviations are neither exposed nor
accepted. Verse and chapter numbers are 1-based. Missing numbers in the source
remain missing rather than being synthesized.

The global generator uses the original 128-bit-key ChaCha20 construction. It
becomes a CSPRNG only when the successful caller supplies an unpredictable
`[u8; 16]` key and keeps that key secret. The crate cannot obtain entropy itself
because it is dependency-free and supports environments without an
operating-system random source.

`seed` returns `true` only when that call installs the process-wide key. Once
it succeeds, later calls leave the key and generator position unchanged and
return `false`.

## Command-line example

The `ja-colloquial` example is a `std` executable. Its dev-only `getrandom`
dependency obtains the seed for random selection; it is not linked into the
library. Build it with `cargo build --example ja-colloquial`, then use:

```console
target/debug/examples/ja-colloquial --rand
target/debug/examples/ja-colloquial -r
target/debug/examples/ja-colloquial --list-books
target/debug/examples/ja-colloquial 創世記 4:13
```

## C API

The checked-in C99/C++ header is `include/ja_colloquial.h`. From an unpacked
source package, build both native library forms with Rust 1.98.1 or newer:

```console
cargo rustc --profile capi --lib \
    --crate-type staticlib,cdylib -- \
    --cfg ja_colloquial_c_artifact
```

The resulting platform-specific artifacts are under `target/capi/`. The C ABI
uses numeric book constants from the header and returns static, NUL-terminated
UTF-8 text which must not be freed or modified.

```c
#include <ja_colloquial.h>

if (ja_colloquial_seed(UINT64_C(1), UINT64_C(2)) != 0) {
    /* Another caller already installed the process-wide key. */
}
ja_colloquial_verse_t verse = ja_colloquial_get_verse(
    JA_COLLOQUIAL_BOOK_GENESIS,
    4,
    13
);
```

crates.io distributes the source package rather than prebuilt C libraries or
installed headers. C users can download and unpack the `.crate` archive and run
the command above without a Git checkout.

Release maintainers can run `./build-release.sh` to build packaged libraries
for the supported Linux musl, Darwin, FreeBSD, NetBSD, and illumos targets.
Musl archives contain the static library; other archives contain both static
and dynamic libraries. Every archive also contains `bin/ja-colloquial` and is
named `ja-colloquial-<version>-<target>.tar.gz`. The NetBSD cross build uses a
small in-tree `libexecinfo` stub because the cross-rs image lacks that target
library.

## License and source text

The project-authored implementation and packaging are dedicated under
[CC0 1.0 Universal](https://creativecommons.org/publicdomain/zero/1.0/). The
bundled Japanese Colloquial translation is in the public domain. See
`LICENSE` for the complete CC0 legal code.
