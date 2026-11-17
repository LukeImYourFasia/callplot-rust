// The 26 trap tests of DESIGN-RUST.md section 4.4 (names follow the design's
// rule: the helper, then the trap).  Row 21 is the perf trap (one 100,000
// character line, under 5 seconds in a debug build); row 26 flips each of the
// first 200 bytes of every fixture to 0xFF one at a time and checks that
// render never panics.
//
// Copyright (C) 2004 dynamicsoft, Inc.
//
// This program is free software; you can redistribute it and/or modify it
// under the terms of the GNU Lesser General Public License as published by
// the Free Software Foundation; either version 2.1 of the License, or (at
// your option) any later version.  A copy of the license is in LICENSE.

use std::fs::read;
use std::fs::read_dir;
use std::panic::catch_unwind;
use std::time::Instant;

use callplot::api::render;
use callplot::cli::decode_java_utf8;
use callplot::constants::MODE_TEXT;
use callplot::errors::Ex;
use callplot::errors::ErrorKind;
use callplot::escape::esc_html;
use callplot::escape::esc_mermaid;
use callplot::jcompat::U;
use callplot::jcompat::CharBuf;
use callplot::jcompat::to_units;
use callplot::jcompat::from_units;
use callplot::jcompat::str_bytes;
use callplot::jcompat::bytes_str;
use callplot::jcompat::u_len;
use callplot::jcompat::read_lines;
use callplot::jcompat::java_trim;
use callplot::jcompat::parse_java_int;
use callplot::jcompat::idiv;
use callplot::jcompat::fdiv;
use callplot::jcompat::fmt_d;
use callplot::jcompat::java_split_slash;
use callplot::jcompat::char_buf_new;
use callplot::jcompat::buf_get;
use callplot::jcompat::buf_set;
use callplot::jcompat::buf_slice;
use callplot::jsnum::js_num;
use callplot::jsnum::js_round;
use callplot::jsnum::js_to_fixed;
use callplot::prep::awk_fields;
use callplot::prep::awk_num;

// ------------------------------------------------------------------ helpers

pub const NUMBER_FORMAT: i64 = 0;
pub const ILLEGAL_ARGUMENT: i64 = 1;
pub const INDEX: i64 = 2;
pub const NULL_POINTER: i64 = 3;
pub const ARITHMETIC: i64 = 4;
pub const NEGATIVE_ARRAY_SIZE: i64 = 5;
pub const TABLE_INPUT: i64 = 6;
pub const RUNTIME: i64 = 7;
pub const NO_ERROR: i64 = -1;

pub fn kind_of_num(k: ErrorKind) -> i64 {
    if k == ErrorKind::NUMBER_FORMAT {
        return NUMBER_FORMAT;
    }
    if k == ErrorKind::ILLEGAL_ARGUMENT {
        return ILLEGAL_ARGUMENT;
    }
    if k == ErrorKind::INDEX {
        return INDEX;
    }
    if k == ErrorKind::NULL_POINTER {
        return NULL_POINTER;
    }
    if k == ErrorKind::ARITHMETIC {
        return ARITHMETIC;
    }
    if k == ErrorKind::NEGATIVE_ARRAY_SIZE {
        return NEGATIVE_ARRAY_SIZE;
    }
    if k == ErrorKind::TABLE_INPUT {
        return TABLE_INPUT;
    }
    return RUNTIME;
}

pub fn kind_i(r: Result<i64, Ex>) -> i64 {
    match r {
        Err(e) => {
            return kind_of_num(e.kind);
        }
        Ok(_) => {
            return NO_ERROR;
        }
    }
}

pub fn kind_u(r: Result<u16, Ex>) -> i64 {
    match r {
        Err(e) => {
            return kind_of_num(e.kind);
        }
        Ok(_) => {
            return NO_ERROR;
        }
    }
}

pub fn kind_units(r: Result<U, Ex>) -> i64 {
    match r {
        Err(e) => {
            return kind_of_num(e.kind);
        }
        Ok(_) => {
            return NO_ERROR;
        }
    }
}

pub fn kind_buf(r: Result<CharBuf, Ex>) -> i64 {
    match r {
        Err(e) => {
            return kind_of_num(e.kind);
        }
        Ok(_) => {
            return NO_ERROR;
        }
    }
}

pub fn us(lit: &str) -> U {
    return to_units(lit.to_string());
}

pub fn txt(u: &U) -> String {
    return from_units(u);
}

pub fn unit_lines(text: &str) -> Vec<U> {
    return read_lines(&to_units(text.to_string()));
}

// a line list as one string, joined with U+0001 (never in the rows)
pub fn joined_lines(ls: Vec<U>) -> String {
    let mut out: String = "".to_string();
    let mut k: usize = 0;
    for l in ls {
        if k > 0 {
            out = format!("{}{}", out, "\u{1}");
        }
        out = format!("{}{}", out, from_units(&l));
        k += 1;
    }
    return out;
}

pub fn same_bytes(a: Vec<u8>, b: Vec<u8>) -> bool {
    if a.len() != b.len() {
        return false;
    }
    let mut i: usize = 0;
    while i < a.len() {
        if a[i] != b[i] {
            return false;
        }
        i += 1;
    }
    return true;
}

pub fn text_of_bytes(v: Vec<u8>) -> String {
    return bytes_str(v);
}

pub fn bytes_of(lit: &str) -> Vec<u8> {
    return str_bytes(lit.to_string());
}

pub fn hex_bytes(lit: &str) -> Vec<u8> {
    let raw: Vec<u8> = bytes_of(lit);
    let mut out: Vec<u8> = vec![];
    let n: usize = raw.len();
    let mut i: usize = 0;
    while i + 1 < n {
        let hi: u8 = hex_digit(raw[i]);
        let lo: u8 = hex_digit(raw[i + 1]);
        if hi < 16 && lo < 16 {
            out.push((hi << 4) + lo);
            i += 2;
        } else {
            i += 1;
        }
    }
    return out;
}

pub fn hex_digit(c: u8) -> u8 {
    if c >= 0x30 && c <= 0x39 {
        return c - 0x30;
    }
    if c >= 0x61 && c <= 0x66 {
        return c - 0x61 + 10;
    }
    return 0xFF;
}

pub fn has_sub(hay: &U, needle: &U) -> bool {
    let n: usize = u_len(hay) as usize;
    let m: usize = u_len(needle) as usize;
    if m > n {
        return false;
    }
    let mut i: usize = 0;
    while i + m <= n {
        let mut ok: bool = true;
        let mut j: usize = 0;
        while j < m {
            if hay[i + j] != needle[j] {
                ok = false;
                break;
            }
            j += 1;
        }
        if ok {
            return true;
        }
        i += 1;
    }
    return false;
}

pub fn pipe_pos(l: &U) -> Vec<i64> {
    let mut out: Vec<i64> = vec![];
    let n: usize = u_len(l) as usize;
    let mut i: usize = 0;
    while i < n {
        if l[i] == 0x7C {
            out.push(i as i64);
        }
        i += 1;
    }
    return out;
}

pub fn same_pos(a: Vec<i64>, b: Vec<i64>) -> bool {
    if a.len() != b.len() {
        return false;
    }
    let mut i: usize = 0;
    while i < a.len() {
        if a[i] != b[i] {
            return false;
        }
        i += 1;
    }
    return true;
}

pub fn joined_parts(ls: Vec<U>) -> String {
    return joined_lines(ls);
}

pub const FIXTURE_DIR: &str = "../callplot-python/test/fixtures";

pub fn fixture_paths() -> Vec<String> {
    let mut names: Vec<String> = vec![];
    let rd = read_dir(FIXTURE_DIR.to_string());
    match rd {
        Err(_) => {
            return names;
        }
        Ok(entries) => {
            for e in entries {
                match e {
                    Err(_) => {
                        continue;
                    }
                    Ok(de) => {
                        let p = de.path();
                        match p.to_str() {
                            Some(x) => {
                                names.push(x.to_string());
                            }
                            None => {
                            }
                        }
                    }
                }
            }
        }
    }
    names.sort();
    return names;
}

// -------------------------------------------------------------------- traps

// 1. Java trim strips only chars <= U+0020, so U+00A0 stays
#[test]
fn trap01_jtrim_nbsp_kept() {
    let u: U = us("\u{a0}x\u{a0}");
    let t: U = java_trim(u.clone());
    assert_eq!(t, u);
}

// 2. read_lines never splits on a lone \x0b
#[test]
fn trap02_read_lines_lone_vtab_is_one_line() {
    let ls: Vec<U> = unit_lines("a\u{b}b");
    assert_eq!(ls.len(), 1);
    assert_eq!(joined_lines(ls), "a\u{b}b");
}

// 3. the seven read_lines rows of DESIGN-RUST.md section 3.2
#[test]
fn trap03_read_lines_rows_of_section_3_2() {
    assert_eq!(unit_lines("").len(), 0);
    assert_eq!(unit_lines("a").len(), 1);
    assert_eq!(joined_lines(unit_lines("a")), "a");
    assert_eq!(unit_lines("a\n").len(), 1);
    assert_eq!(joined_lines(unit_lines("a\n")), "a");
    assert_eq!(unit_lines("a\n\n").len(), 2);
    assert_eq!(joined_lines(unit_lines("a\n\n")), "a\u{1}");
    assert_eq!(unit_lines("\n").len(), 1);
    assert_eq!(joined_lines(unit_lines("\n")), "");
    assert_eq!(unit_lines("a\r\nb\rc").len(), 3);
    assert_eq!(joined_lines(unit_lines("a\r\nb\rc")), "a\u{1}b\u{1}c");
    assert_eq!(unit_lines("a\u{b}b").len(), 1);
    assert_eq!(joined_lines(unit_lines("a\u{b}b")), "a\u{b}b");
}

// 4. awk splits on space, tab and newline: \x0b and U+00A0 keep one field
// (awk_fields pads NF to 4, so count the non-empty pieces)
#[test]
fn trap04_awk_fields_vtab_nbsp_keeps_one_field() {
    let line: Vec<u8> = vec![0x61, 0x0B, 0x62, 0xC2, 0xA0, 0x63];
    let fs: Vec<Vec<u8>> = awk_fields(&line);
    let mut real: i64 = 0;
    for f in &fs {
        if !f.is_empty() {
            real += 1;
        }
    }
    assert_eq!(real, 1);
    assert!(same_bytes(fs[0].clone(), line));
}

// 5. Integer.parseInt: only ASCII digits, one optional sign, int range
#[test]
fn trap05_parse_java_int_accepts_plus_7_only_of_that_row() {
    assert_eq!(kind_i(parse_java_int(&us(" 5"))), NUMBER_FORMAT);
    assert_eq!(kind_i(parse_java_int(&us("5 "))), NUMBER_FORMAT);
    assert_eq!(kind_i(parse_java_int(&us("1_0"))), NUMBER_FORMAT);
    assert_eq!(kind_i(parse_java_int(&us("\u{663}"))), NUMBER_FORMAT);
    assert_eq!(kind_i(parse_java_int(&us(""))), NUMBER_FORMAT);
    assert_eq!(kind_i(parse_java_int(&us("2147483648"))), NUMBER_FORMAT);
    assert_eq!(kind_i(parse_java_int(&us("+7"))), NO_ERROR);
    match parse_java_int(&us("+7")) {
        Ok(v) => {
            assert_eq!(v, 7);
        }
        Err(_) => {
            assert!(false);
        }
    }
}

// 6. int division rounds toward zero, both signs
#[test]
fn trap06_idiv_rounds_toward_zero_both_signs() {
    assert_eq!(kind_i(idiv(-7, 2)), NO_ERROR);
    assert_eq!(kind_i(idiv(7, -2)), NO_ERROR);
    match idiv(-7, 2) {
        Ok(v) => {
            assert_eq!(v, -3);
        }
        Err(_) => {
            assert!(false);
        }
    }
    match idiv(7, -2) {
        Ok(v) => {
            assert_eq!(v, -3);
        }
        Err(_) => {
            assert!(false);
        }
    }
}

// 7. a zero divisor throws Arithmetic
#[test]
fn trap07_idiv_by_zero_throws_arithmetic() {
    assert_eq!(kind_i(idiv(1, 0)), ARITHMETIC);
}

// 8. INT_MIN / -1 is INT_MIN and never panics
#[test]
fn trap08_idiv_int_min_over_minus_one_is_int_min() {
    assert_eq!(kind_i(idiv(-2147483648, -1)), NO_ERROR);
    match idiv(-2147483648, -1) {
        Ok(v) => {
            assert_eq!(v, -2147483648);
        }
        Err(_) => {
            assert!(false);
        }
    }
}

// 9. the Java int boundary.  The JInt newtype of section 3.3 is still a
// deferred step of this port, so this trap pins the int range Java's
// Integer.parseInt enforced, which every counter in the port respects.
#[test]
fn trap09_parse_java_int_keeps_the_java_int_range() {
    assert_eq!(kind_i(parse_java_int(&us("2147483647"))), NO_ERROR);
    assert_eq!(kind_i(parse_java_int(&us("2147483648"))), NUMBER_FORMAT);
    assert_eq!(kind_i(parse_java_int(&us("-2147483648"))), NO_ERROR);
    assert_eq!(kind_i(parse_java_int(&us("-2147483649"))), NUMBER_FORMAT);
    match parse_java_int(&us("2147483647")) {
        Ok(v) => {
            assert_eq!(v, 2147483647);
        }
        Err(_) => {
            assert!(false);
        }
    }
    match parse_java_int(&us("-2147483648")) {
        Ok(v) => {
            assert_eq!(v, -2147483648);
        }
        Err(_) => {
            assert!(false);
        }
    }
}

// 10. fmt_d prints Java's Double.toString for the 19 rows of section 3.3
#[test]
fn trap10_fmt_d_prints_java_double_text() {
    assert_eq!(fmt_d(50.0), "50.0");
    assert_eq!(fmt_d(0.0), "0.0");
    assert_eq!(fmt_d(-0.0), "-0.0");
    assert_eq!(fmt_d(10.2), "10.2");
    assert_eq!(fmt_d(-0.4), "-0.4");
    assert_eq!(fmt_d(1.6 * 2.0 * 14.0), "44.800000000000004");
    assert_eq!(fmt_d(0.001), "0.001");
    assert_eq!(fmt_d(0.0001), "1.0E-4");
    assert_eq!(fmt_d(9999999.0), "9999999.0");
    assert_eq!(fmt_d(10000000.0), "1.0E7");
    assert_eq!(fmt_d(12345678.0), "1.2345678E7");
    assert_eq!(fmt_d(1.0 / 0.0), "Infinity");
    assert_eq!(fmt_d(1e16), "1.0E16");
    assert_eq!(fmt_d(1.5e-5), "1.5E-5");
    assert_eq!(fmt_d(123456789012345680.0), "1.2345678901234568E17");
    assert_eq!(fmt_d(0.2 + 15.0), "15.2");
    assert_eq!(fmt_d(-0.2 + 9.0), "8.8");
    assert_eq!(fmt_d(1.6 * 3.0 * 14.0), "67.20000000000002");
    assert_eq!(fmt_d(16.0 / 6.0 + 0.25), "2.9166666666666665");
}

// 11. Java double division by zero gives Infinity, -Infinity and NaN
#[test]
fn trap11_fdiv_by_zero_gives_java_infinities() {
    assert_eq!(fmt_d(fdiv(1.0, 0.0)), "Infinity");
    assert_eq!(fmt_d(fdiv(-1.0, 0.0)), "-Infinity");
    assert_eq!(fmt_d(fdiv(0.0, 0.0)), "NaN");
}

// 12. jlen counts UTF-16 units, not bytes and not code points
#[test]
fn trap12_jlen_counts_utf16_units() {
    assert_eq!(u_len(&us("a\u{1F600}b")), 4);
}

// 13. layout width of a caption with an emoji: the emoji is 2 units, so the
// pipes of the caption line land on the same columns as the arrow line
#[test]
fn trap13_layout_emoji_caption_counts_two_units() {
    let text: String = "guy/a/First\nThe\n\nguy/b/Second\nPlayer\n\na->b/x\u{1F600}y\n".to_string();
    let r = render(text, MODE_TEXT);
    match r {
        Err(_) => {
            assert!(false);
        }
        Ok(out) => {
            assert_eq!(u_len(&us("x\u{1F600}y")), 4);
            let ls: Vec<U> = read_lines(&out);
            let cap: U = us("x\u{1F600}y");
            let arrow: U = us("-->");
            let mut cap_pipes: Vec<i64> = vec![];
            let mut arrow_pipes: Vec<i64> = vec![];
            for l in &ls {
                if has_sub(l, &cap) && cap_pipes.is_empty() {
                    cap_pipes = pipe_pos(l);
                } else if has_sub(l, &arrow) && arrow_pipes.is_empty() {
                    arrow_pipes = pipe_pos(l);
                }
            }
            assert!(!cap_pipes.is_empty());
            assert!(!arrow_pipes.is_empty());
            assert!(same_pos(cap_pipes, arrow_pipes));
        }
    }
}

// 14. CharBuf index checks: -1 and width both throw Index
#[test]
fn trap14_charbuf_index_out_of_range_throws_index() {
    let made = char_buf_new(3);
    match made {
        Err(_) => {
            assert!(false);
        }
        Ok(b) => {
            assert_eq!(kind_u(buf_get(&b, -1)), INDEX);
            assert_eq!(kind_u(buf_get(&b, 3)), INDEX);
            let s = buf_set(&b, 3, 0x78);
            assert_eq!(kind_buf(s), INDEX);
        }
    }
}

// 15. new char[-1] throws NegativeArraySize
#[test]
fn trap15_charbuf_new_negative_width_throws_negative_array_size() {
    let made = char_buf_new(-1);
    match made {
        Err(e) => {
            assert_eq!(kind_of_num(e.kind), NEGATIVE_ARRAY_SIZE);
        }
        Ok(_) => {
            assert!(false);
        }
    }
}

// 16. a lone high surrogate prints as ?
#[test]
fn trap16_charbuf_lone_high_surrogate_prints_question() {
    let made = char_buf_new(2);
    match made {
        Err(_) => {
            assert!(false);
        }
        Ok(b0) => {
            let s1 = buf_set(&b0, 0, 0x61);
            match s1 {
                Err(_) => {
                    assert!(false);
                }
                Ok(b1) => {
                    let s2 = buf_set(&b1, 1, 0xD83D);
                    match s2 {
                        Err(_) => {
                            assert!(false);
                        }
                        Ok(b2) => {
                            let sl = buf_slice(&b2, 0, 2);
                            match sl {
                                Err(_) => {
                                    assert!(false);
                                }
                                Ok(u) => {
                                    assert_eq!(txt(&u), "a?");
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

// 17. decode_java_utf8: the 18 rows of the DESIGN.md section 3.5 table
#[test]
fn trap17_decode_java_utf8_matches_java_eight() {
    let rep: String = "\u{FFFD}".to_string();
    let two: String = format!("{}{}", rep, rep);
    let three: String = format!("{}{}{}", rep, rep, rep);
    let four: String = format!("{}{}{}{}", rep, rep, rep, rep);
    let five: String = format!("{}{}{}{}{}", rep, rep, rep, rep, rep);
    let b0: Vec<u8> = hex_bytes("ed a0 80");
    assert_eq!(decode_java_utf8(&b0), rep);
    let b1: Vec<u8> = hex_bytes("ed bf bf");
    assert_eq!(decode_java_utf8(&b1), rep);
    let b2: Vec<u8> = hex_bytes("ed b0 80");
    assert_eq!(decode_java_utf8(&b2), rep);
    let b3: Vec<u8> = hex_bytes("ed a0");
    assert_eq!(decode_java_utf8(&b3), rep);
    let b4: Vec<u8> = hex_bytes("ed a0 41");
    assert_eq!(decode_java_utf8(&b4), format!("{}A", rep));
    let b5: Vec<u8> = hex_bytes("ed a0 80 ed a0 80");
    assert_eq!(decode_java_utf8(&b5), two);
    let b6: Vec<u8> = hex_bytes("e2 82 ed a0 80");
    assert_eq!(decode_java_utf8(&b6), two);
    let b7: Vec<u8> = hex_bytes("f0 9f ed a0 80");
    assert_eq!(decode_java_utf8(&b7), two);
    let b8: Vec<u8> = hex_bytes("41 ed a0 ed a0");
    assert_eq!(decode_java_utf8(&b8), format!("A{}", two));
    let b9: Vec<u8> = hex_bytes("e2 82");
    assert_eq!(decode_java_utf8(&b9), rep);
    let b10: Vec<u8> = hex_bytes("f0 9f 98");
    assert_eq!(decode_java_utf8(&b10), rep);
    let b11: Vec<u8> = hex_bytes("c0 af");
    assert_eq!(decode_java_utf8(&b11), two);
    let b12: Vec<u8> = hex_bytes("e0 80 80");
    assert_eq!(decode_java_utf8(&b12), three);
    let b13: Vec<u8> = hex_bytes("e0 9f 80");
    assert_eq!(decode_java_utf8(&b13), three);
    let b14: Vec<u8> = hex_bytes("f4 90 80 80");
    assert_eq!(decode_java_utf8(&b14), four);
    let b15: Vec<u8> = hex_bytes("f0 8f 80 80");
    assert_eq!(decode_java_utf8(&b15), four);
    let b16: Vec<u8> = hex_bytes("f8 88 80 80 80");
    assert_eq!(decode_java_utf8(&b16), five);
    let b17: Vec<u8> = hex_bytes("61 ff 62");
    assert_eq!(decode_java_utf8(&b17), format!("a{}b", rep));
}

// 18. esc_html turns ' into &#39;
#[test]
fn trap18_esc_html_single_quote_becomes_numeric() {
    assert_eq!(txt(&esc_html(&us("'"))), "&#39;");
}

// 19. esc_mermaid is one pass over the characters
#[test]
fn trap19_esc_mermaid_one_pass_on_hash_semicolon() {
    assert_eq!(txt(&esc_mermaid(&us("#;"))), "#35;#59;");
}

// 20. awk_num prints like awk's %.6g (the 12 rows of DESIGN-RUST 3.8)
#[test]
fn trap20_awk_num_prints_like_awk_six_g() {
    assert_eq!(text_of_bytes(awk_num(3.0)), "3");
    assert_eq!(text_of_bytes(awk_num(0.5)), "0.5");
    assert_eq!(text_of_bytes(awk_num(1234567.5)), "1.23457e+06");
    assert_eq!(text_of_bytes(awk_num(123456.5)), "123456");
    assert_eq!(text_of_bytes(awk_num(0.00012345678)), "0.000123457");
    assert_eq!(text_of_bytes(awk_num(0.000025)), "2.5e-05");
    assert_eq!(text_of_bytes(awk_num(0.125)), "0.125");
    assert_eq!(text_of_bytes(awk_num(1.0000005)), "1");
    assert_eq!(text_of_bytes(awk_num(59.999)), "59.999");
    assert_eq!(text_of_bytes(awk_num(3723.456)), "3723.46");
    assert_eq!(text_of_bytes(awk_num(-0.25)), "-0.25");
    assert_eq!(text_of_bytes(awk_num(1e20)), "100000000000000000000");
}

// 21. one 100,000-character line parses in well under 5 seconds (debug)
#[test]
fn trap21_parse_100000_char_line_under_5_seconds() {
    let mut cap: String = "".to_string();
    let mut i: i64 = 0;
    while i < 100000 {
        cap.push('x');
        i += 1;
    }
    let text: String = format!("{}{}{}", "guy/a/Name\n\nPlayer\na->b/", cap, "\n");
    let t0 = Instant::now();
    let r = render(text, MODE_TEXT);
    let ms = t0.elapsed().as_millis();
    match r {
        Err(_) => {
            println!("trap21 raised: {} ms", ms);
        }
        Ok(_) => {
            println!("trap21 ok: {} ms", ms);
        }
    }
    assert!(ms < 5000);
}

// 22. the slash split never splits at position 0, and Java drops the trailing
// empty parts.  (DESIGN-RUST.md row 22 prints "[] for ///"; both oracles give
// ["/"], which is what the goldens pin, so this test pins the oracle.)
#[test]
fn trap22_java_split_slash_drops_trailing_empty_parts() {
    let tri: Vec<U> = java_split_slash(us("///"));
    assert_eq!(tri.len(), 1);
    assert_eq!(joined_parts(tri), "/");
    let empt: Vec<U> = java_split_slash(us(""));
    assert_eq!(empt.len(), 1);
    assert_eq!(joined_parts(empt), "");
    let ab: Vec<U> = java_split_slash(us("a/b/"));
    assert_eq!(ab.len(), 2);
    assert_eq!(joined_parts(ab), "a\u{1}b");
    let lead: Vec<U> = java_split_slash(us("/a"));
    assert_eq!(lead.len(), 1);
    assert_eq!(joined_parts(lead), "/a");
}

// 23. js_to_fixed rounds the exact binary value, ties away from zero
#[test]
fn trap23_js_to_fixed_rounds_exact_binary_ties_up() {
    assert_eq!(js_to_fixed(0.25, 1), "0.3");
    assert_eq!(js_to_fixed(2.5, 0), "3");
    assert_eq!(js_to_fixed(1.005, 2), "1.00");
}

// 24. js_round is JS Math.round: halves go toward +Infinity
#[test]
fn trap24_js_round_is_math_round_half_up() {
    assert_eq!(js_round(-2.5), -2.0);
    assert_eq!(js_round(2.5), 3.0);
    assert_eq!(js_round(0.49999999999999994), 0.0);
}

// 25. js_num prints -0 as 0
#[test]
fn trap25_js_num_negative_zero_prints_zero() {
    match js_num(-0.0) {
        Ok(s) => {
            assert_eq!(s, "0");
        }
        Err(_) => {
            assert!(false);
        }
    }
}

// 26. flipping each of the first 200 bytes of every fixture to 0xFF never
// panics: a bad byte is an Ex (or accepted text), never a Rust panic
#[test]
fn trap26_flipping_input_bytes_to_ff_never_panics() {
    let paths: Vec<String> = fixture_paths();
    assert!(!paths.is_empty());
    for path in paths {
        let raw = read(&path);
        match raw {
            Err(_) => {
                continue;
            }
            Ok(bytes) => {
                let n: usize = bytes.len();
                let mut lim: usize = n;
                if n > 200 {
                    lim = 200;
                }
                let mut i: usize = 0;
                while i < lim {
                    let one: Vec<u8> = flip_at(&bytes, i);
                    let text: String = decode_java_utf8(&one);
                    if !no_panic(text) {
                        println!("trap26 panicked on {}", path);
                        assert!(false);
                    }
                    i += 1;
                }
            }
        }
    }
}

pub fn flip_at(bytes: &Vec<u8>, i: usize) -> Vec<u8> {
    let mut one: Vec<u8> = vec![];
    let n: usize = bytes.len();
    let mut k: usize = 0;
    while k < n {
        if k == i {
            one.push(0xFF);
        } else {
            one.push(bytes[k]);
        }
        k += 1;
    }
    return one;
}

pub fn no_panic(text: String) -> bool {
    let r = catch_unwind(|| {
        let out = render(text.to_string(), MODE_TEXT);
        match out {
            Err(_) => {
            }
            Ok(_) => {
            }
        }
    });
    match r {
        Err(_) => {
            return false;
        }
        Ok(_) => {
            return true;
        }
    }
}

// 27. decode_java_utf8 reads 4-byte leads exactly like Java 8: cp from all
// four bytes, one char; F0 wants 90..BF, F4 wants 80..8F; a bad continuation
// restarts fresh; a truncated valid prefix gives one U+FFFD (the astral fuzz
// corpus, seed 1, exposed the 3-byte cp bug; confirmed on OpenJDK 1.8.0_512
// via tools/jref.sh on the saved cases).
#[test]
fn trap27_decode_java_utf8_four_byte_leads() {
    let rep: String = "\u{FFFD}".to_string();
    let two: String = format!("{}{}", rep, rep);
    let b0: Vec<u8> = hex_bytes("f0 9f 98 80");
    assert_eq!(decode_java_utf8(&b0), "\u{1F600}");
    let b1: Vec<u8> = hex_bytes("f4 8f bf bf");
    assert_eq!(decode_java_utf8(&b1), "\u{10FFFF}");
    let b2: Vec<u8> = hex_bytes("f0 9f 9f 98");
    assert_eq!(decode_java_utf8(&b2), "\u{1F7D8}");
    let b3: Vec<u8> = hex_bytes("f0 9f 98 c2");
    assert_eq!(decode_java_utf8(&b3), two);
    let b4: Vec<u8> = hex_bytes("f0 9f 98 80 41");
    assert_eq!(decode_java_utf8(&b4), "\u{1F600}A");
}

// 28. decode_java_utf8 reads 3-byte leads exactly like Java 8: exactly two
// continuations make a character, so e2 98 8e before a space is a complete U+260E
// (the old shared branch demanded a third continuation and turned it into
// U+FFFD, which broke the unicode fixture); E0 wants A0..BF and ED wants 80..9F,
// so an overlong or surrogate follow-on is rejected one byte at a time.
// Every row was confirmed byte-for-byte against the oracle through tools/jref.sh
// on a one-message fixture (2026-11-17).
#[test]
fn trap28_decode_java_utf8_three_byte_leads() {
    let rep: String = "\u{FFFD}".to_string();
    let three: String = format!("{rep}{rep}{rep}");
    let four: String = format!("{rep}{rep}{rep}{rep}");
    let b0: Vec<u8> = hex_bytes("e2 98 8e 41");
    assert_eq!(decode_java_utf8(&b0), "\u{260E}A");
    let b1: Vec<u8> = hex_bytes("e2 98 80");
    assert_eq!(decode_java_utf8(&b1), "\u{2600}");
    let b2: Vec<u8> = hex_bytes("e2 98");
    assert_eq!(decode_java_utf8(&b2), rep);
    let b3: Vec<u8> = hex_bytes("e2 98 41");
    assert_eq!(decode_java_utf8(&b3), rep + "A");
    let b4: Vec<u8> = hex_bytes("ed 9f bf");
    assert_eq!(decode_java_utf8(&b4), "\u{D7FF}");
    let b5: Vec<u8> = hex_bytes("e0 80 80");
    assert_eq!(decode_java_utf8(&b5), three);
    let b6: Vec<u8> = hex_bytes("ed c0 80");
    assert_eq!(decode_java_utf8(&b6), three);
    let b7: Vec<u8> = hex_bytes("ef c0 80");
    assert_eq!(decode_java_utf8(&b7), three);
    let b8: Vec<u8> = hex_bytes("f4 90 80 80");
    assert_eq!(decode_java_utf8(&b8), four);
    let b9: Vec<u8> = hex_bytes("f0 80 80 80");
    assert_eq!(decode_java_utf8(&b9), four);
}
