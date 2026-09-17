#ifndef JA_COLLOQUIAL_H
#define JA_COLLOQUIAL_H

#include <stdint.h>

/* A zero-based canonical book index. */
typedef int ja_colloquial_book_t;

#define JA_COLLOQUIAL_BOOK_GENESIS ((ja_colloquial_book_t)0)
#define JA_COLLOQUIAL_BOOK_EXODUS ((ja_colloquial_book_t)1)
#define JA_COLLOQUIAL_BOOK_LEVITICUS ((ja_colloquial_book_t)2)
#define JA_COLLOQUIAL_BOOK_NUMBERS ((ja_colloquial_book_t)3)
#define JA_COLLOQUIAL_BOOK_DEUTERONOMY ((ja_colloquial_book_t)4)
#define JA_COLLOQUIAL_BOOK_JOSHUA ((ja_colloquial_book_t)5)
#define JA_COLLOQUIAL_BOOK_JUDGES ((ja_colloquial_book_t)6)
#define JA_COLLOQUIAL_BOOK_RUTH ((ja_colloquial_book_t)7)
#define JA_COLLOQUIAL_BOOK_FIRST_SAMUEL ((ja_colloquial_book_t)8)
#define JA_COLLOQUIAL_BOOK_SECOND_SAMUEL ((ja_colloquial_book_t)9)
#define JA_COLLOQUIAL_BOOK_FIRST_KINGS ((ja_colloquial_book_t)10)
#define JA_COLLOQUIAL_BOOK_SECOND_KINGS ((ja_colloquial_book_t)11)
#define JA_COLLOQUIAL_BOOK_FIRST_CHRONICLES ((ja_colloquial_book_t)12)
#define JA_COLLOQUIAL_BOOK_SECOND_CHRONICLES ((ja_colloquial_book_t)13)
#define JA_COLLOQUIAL_BOOK_EZRA ((ja_colloquial_book_t)14)
#define JA_COLLOQUIAL_BOOK_NEHEMIAH ((ja_colloquial_book_t)15)
#define JA_COLLOQUIAL_BOOK_ESTHER ((ja_colloquial_book_t)16)
#define JA_COLLOQUIAL_BOOK_JOB ((ja_colloquial_book_t)17)
#define JA_COLLOQUIAL_BOOK_PSALMS ((ja_colloquial_book_t)18)
#define JA_COLLOQUIAL_BOOK_PROVERBS ((ja_colloquial_book_t)19)
#define JA_COLLOQUIAL_BOOK_ECCLESIASTES ((ja_colloquial_book_t)20)
#define JA_COLLOQUIAL_BOOK_SONG_OF_SOLOMON ((ja_colloquial_book_t)21)
#define JA_COLLOQUIAL_BOOK_ISAIAH ((ja_colloquial_book_t)22)
#define JA_COLLOQUIAL_BOOK_JEREMIAH ((ja_colloquial_book_t)23)
#define JA_COLLOQUIAL_BOOK_LAMENTATIONS ((ja_colloquial_book_t)24)
#define JA_COLLOQUIAL_BOOK_EZEKIEL ((ja_colloquial_book_t)25)
#define JA_COLLOQUIAL_BOOK_DANIEL ((ja_colloquial_book_t)26)
#define JA_COLLOQUIAL_BOOK_HOSEA ((ja_colloquial_book_t)27)
#define JA_COLLOQUIAL_BOOK_JOEL ((ja_colloquial_book_t)28)
#define JA_COLLOQUIAL_BOOK_AMOS ((ja_colloquial_book_t)29)
#define JA_COLLOQUIAL_BOOK_OBADIAH ((ja_colloquial_book_t)30)
#define JA_COLLOQUIAL_BOOK_JONAH ((ja_colloquial_book_t)31)
#define JA_COLLOQUIAL_BOOK_MICAH ((ja_colloquial_book_t)32)
#define JA_COLLOQUIAL_BOOK_NAHUM ((ja_colloquial_book_t)33)
#define JA_COLLOQUIAL_BOOK_HABAKKUK ((ja_colloquial_book_t)34)
#define JA_COLLOQUIAL_BOOK_ZEPHANIAH ((ja_colloquial_book_t)35)
#define JA_COLLOQUIAL_BOOK_HAGGAI ((ja_colloquial_book_t)36)
#define JA_COLLOQUIAL_BOOK_ZECHARIAH ((ja_colloquial_book_t)37)
#define JA_COLLOQUIAL_BOOK_MALACHI ((ja_colloquial_book_t)38)
#define JA_COLLOQUIAL_BOOK_MATTHEW ((ja_colloquial_book_t)39)
#define JA_COLLOQUIAL_BOOK_MARK ((ja_colloquial_book_t)40)
#define JA_COLLOQUIAL_BOOK_LUKE ((ja_colloquial_book_t)41)
#define JA_COLLOQUIAL_BOOK_JOHN ((ja_colloquial_book_t)42)
#define JA_COLLOQUIAL_BOOK_ACTS ((ja_colloquial_book_t)43)
#define JA_COLLOQUIAL_BOOK_ROMANS ((ja_colloquial_book_t)44)
#define JA_COLLOQUIAL_BOOK_FIRST_CORINTHIANS ((ja_colloquial_book_t)45)
#define JA_COLLOQUIAL_BOOK_SECOND_CORINTHIANS ((ja_colloquial_book_t)46)
#define JA_COLLOQUIAL_BOOK_GALATIANS ((ja_colloquial_book_t)47)
#define JA_COLLOQUIAL_BOOK_EPHESIANS ((ja_colloquial_book_t)48)
#define JA_COLLOQUIAL_BOOK_PHILIPPIANS ((ja_colloquial_book_t)49)
#define JA_COLLOQUIAL_BOOK_COLOSSIANS ((ja_colloquial_book_t)50)
#define JA_COLLOQUIAL_BOOK_FIRST_THESSALONIANS ((ja_colloquial_book_t)51)
#define JA_COLLOQUIAL_BOOK_SECOND_THESSALONIANS ((ja_colloquial_book_t)52)
#define JA_COLLOQUIAL_BOOK_FIRST_TIMOTHY ((ja_colloquial_book_t)53)
#define JA_COLLOQUIAL_BOOK_SECOND_TIMOTHY ((ja_colloquial_book_t)54)
#define JA_COLLOQUIAL_BOOK_TITUS ((ja_colloquial_book_t)55)
#define JA_COLLOQUIAL_BOOK_PHILEMON ((ja_colloquial_book_t)56)
#define JA_COLLOQUIAL_BOOK_HEBREWS ((ja_colloquial_book_t)57)
#define JA_COLLOQUIAL_BOOK_JAMES ((ja_colloquial_book_t)58)
#define JA_COLLOQUIAL_BOOK_FIRST_PETER ((ja_colloquial_book_t)59)
#define JA_COLLOQUIAL_BOOK_SECOND_PETER ((ja_colloquial_book_t)60)
#define JA_COLLOQUIAL_BOOK_FIRST_JOHN ((ja_colloquial_book_t)61)
#define JA_COLLOQUIAL_BOOK_SECOND_JOHN ((ja_colloquial_book_t)62)
#define JA_COLLOQUIAL_BOOK_THIRD_JOHN ((ja_colloquial_book_t)63)
#define JA_COLLOQUIAL_BOOK_JUDE ((ja_colloquial_book_t)64)
#define JA_COLLOQUIAL_BOOK_REVELATION ((ja_colloquial_book_t)65)
#define JA_COLLOQUIAL_BOOK_COUNT 66

/* A verse returned by value. text is static UTF-8, is NUL-terminated,
 * and must not be freed or modified. */
typedef struct ja_colloquial_verse {
    ja_colloquial_book_t book;
    int chapter;
    int verse;
    const char *text;
} ja_colloquial_verse_t;

#ifdef __cplusplus
extern "C" {
#endif

/* Installs a 128-bit key and restarts the global random stream.
 * The key is little-endian a followed by little-endian b.
 * Always seed with unpredictable values when possible. */
void ja_colloquial_seed(uint64_t a, uint64_t b);

/* Returns JA_COLLOQUIAL_BOOK_COUNT. */
int ja_colloquial_book_count(void);

/* Returns the chapter count, or -1 for an invalid book. */
int ja_colloquial_book_chapters(ja_colloquial_book_t book);

/* Returns the count of present verse records, or -1 for invalid input.
 * The count can differ from the highest label where the source has gaps. */
int ja_colloquial_chapter_verses(ja_colloquial_book_t book, int chapter);

/* Returns the requested verse, or {-1, 0, 0, NULL} for invalid input. */
ja_colloquial_verse_t ja_colloquial_get_verse(
    ja_colloquial_book_t book,
    int chapter,
    int verse
);

/* Returns a uniformly selected verse. The unseeded stream is deterministic
 * and publicly predictable; call ja_colloquial_seed whenever possible. */
ja_colloquial_verse_t ja_colloquial_random_verse(void);

#ifdef __cplusplus
}
#endif

#endif
