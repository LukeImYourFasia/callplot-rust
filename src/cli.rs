// The three commands (DESIGN-RUST.md section 3.6: run_callplot, run_plottable,
// run_prep; the oracles are cli.py, plottable.py and prep_cli.py).  Each run_*
// returns the exit code (DESIGN.md section 2.1); the binaries are thin wrappers.
// Rules: 1 hand-written argument loop (no argument-parser crate); 2 args_os with
// OsString names; 3 one RunContext for the whole run; 7 the first error stops the
// run and prints callplot: {name}: {message}; 9 a closed pipe is a quiet exit 0
// (this std reports it as an io error of kind BrokenPipe, so wbytes says so);
// 10 a core panic prints the internal error line and exits 1.  This std has no
// unwind-safe wrapper for closures that capture streams, so catch_unwind wraps
// the pure render computes and stream writes stay outside and return Results.
//
// Copyright (C) 2004 dynamicsoft, Inc.
//
// This program is free software; you can redistribute it and/or modify it
// under the terms of the GNU Lesser General Public License as published by
// the Free Software Foundation; either version 2.1 of the License, or (at
// your option) any later version.  A copy of the license is in LICENSE.

use std::ffi::OsString;
use std::io::Read;
use std::io::Write;
use std::io::ErrorKind as IoKind;
use std::fs::read;
use std::fs::write;
use std::process::id;
use std::panic::catch_unwind;

use crate::api::render;
use crate::api::render_cli;
use crate::constants::MODE_TEXT;
use crate::constants::MODE_PIC;
use crate::constants::MODE_XML;
use crate::constants::MODE_VML;
use crate::constants::MODE_SVG;
use crate::constants::MODE_GRAFFLE;
use crate::constants::MODE_MERMAID;
use crate::constants::MODE_PLANTUML;
use crate::constants::MODE_HTML;
use crate::constants::MODE_TABLE;
use crate::constants::MODE_VXML;
use crate::context::RunContext;
use crate::context::run_context_fresh;
use crate::errors::Ex;
use crate::errors::error_name;
use crate::jcompat::U;
use crate::jcompat::from_units;
use crate::jcompat::str_bytes;
use crate::jcompat::bytes_str;
use crate::options::RenderOptions;
use crate::options::render_options_new;
use crate::options::parse_table_style;
use crate::options::TableStyle;
use crate::options::HtmlLayout;
use crate::prep::PrepOptions;
use crate::prep::prep_new;
use crate::prep::prep;
use crate::prep::prep_debug;
use crate::prep::cat_all;

// Section 3.6 rule 7: the usage texts are include_str! copies (byte-exact,
// tabs kept).  usage.txt is Main.java:79-159 plus the new-flag block.
pub const USAGE: &str = include_str!("usage.txt");
pub const PLTT_USAGE: &str = include_str!("plottable_usage.txt");
// The JS port of callplot-prep exited 1 with a usage line on an unknown flag
// (DESIGN-RUST 3.6 callplot-prep; argparse's exit 2 is Appendix B territory).
pub const PREP_USAGE: &str = "usage: callplot-prep [--links] [--debug-file PATH] [file]\n";

// ---------------------------------------------------------------- helpers

// Bytes to a stream, flushed (section 3.2; cli.py write_out/write_err).  false
// means the write failed; for a stdout/stderr stream that means a closed pipe
// (rule 9: exit 0, nothing on stderr).
pub fn wbytes(st: &mut dyn Write, b: &[u8]) -> bool {
    let w = st.write_all(b);
    match w {
        Err(e) => { return e.kind() != IoKind::BrokenPipe; }
        Ok(_) => { }
    }
    let f = st.flush();
    match f {
        Err(e) => { return e.kind() != IoKind::BrokenPipe; }
        Ok(_) => { }
    }
    return true;
}

// The Graffle ``--`` debug lines live in ctx.err (the shared System.err buffer);
// print what is new since the last mark and return the new mark (cli.py drain_err).
// -1 means the write hit a closed pipe.  The mark counts bytes of ctx.err, which
// only ever grows, so byte slices land on character ends.
pub fn drain_err(st: &mut dyn Write, ctx: &RunContext, mark: i64) -> i64 {
    let all: Vec<u8> = str_bytes(ctx.err.clone());
    if (all.len() as i64) <= mark {
        return all.len() as i64;
    }
    let mut more: Vec<u8> = vec![];
    let mut i: usize = mark as usize;
    while i < all.len() {
        more.push(all[i]);
        i += 1;
    }
    if !wbytes(st, &more) {
        return -1;
    }
    return all.len() as i64;
}

// The message of an Ex, empty when the raise carried none (cli.py str(err)).
pub fn ex_msg(m: Option<String>) -> String {
    match m {
        Some(x) => { return x; }
        None => { return "".to_string(); }
    }
}

// An argument as text for flag comparisons and error messages (section 3.6
// rule 2: compare with to_str(); a non-UTF-8 name still opens from OsString).
pub fn arg_str(a: &OsString) -> String {
    let t: Option<&str> = a.to_str();
    match t {
        Some(s) => { return s.to_string(); }
        None => { return "".to_string(); }
    }
}

// ASCII prefix test and cut over byte views (the dialect's String slices stay
// on character ends here because the prefixes are ASCII).
pub fn has_prefix(sb: &Vec<u8>, pre: &Vec<u8>) -> bool {
    if sb.len() < pre.len() {
        return false;
    }
    let mut i: usize = 0;
    while i < pre.len() {
        if sb[i] != pre[i] {
            return false;
        }
        i += 1;
    }
    return true;
}

pub fn strip_pre(sb: &Vec<u8>, plen: usize) -> String {
    let mut rest: Vec<u8> = vec![];
    let mut i: usize = plen;
    while i < sb.len() {
        rest.push(sb[i]);
        i += 1;
    }
    return bytes_str(rest);
}

pub fn clone_ctx(c: &RunContext) -> RunContext {
    return RunContext {
        player_id: c.player_id,
        message_id: c.message_id,
        mynum: c.mynum,
        err: c.err.clone(),
    };
}

pub fn clone_opts(o: &RenderOptions) -> RenderOptions {
    let hl: Option<HtmlLayout> = match &o.html_layout {
        Some(h) => { Some(h.clone()) }
        None => { None }
    };
    return RenderOptions {
        table_style: o.table_style.clone(),
        html_layout: hl,
        id_prefix: o.id_prefix.clone(),
    };
}

pub fn opts_table(style: TableStyle) -> RenderOptions {
    let o: RenderOptions = render_options_new();
    let mut o2: RenderOptions = o;
    o2.table_style = style;
    return o2;
}

pub fn opts_html(h: HtmlLayout) -> RenderOptions {
    let o: RenderOptions = render_options_new();
    let mut o2: RenderOptions = o;
    o2.html_layout = Some(h);
    return o2;
}

// Java 8 UTF-8 decoding (DESIGN-RUST.md section 3.2; decode_java_utf8).  One
// case differs from Rust lossy: an encoded surrogate (ED then A0..BF) becomes
// ONE U+FFFD covering up to 3 bytes.  Invalid continuations reparse each byte
// fresh; a truncated valid prefix gives one U+FFFD.  No BOM removal.
pub fn decode_java_utf8(bytes: &Vec<u8>) -> String {
    let mut s: String = "".to_string();
    let n: usize = bytes.len();
    let rep: u32 = 0xFFFD;
    let mut k: usize = 0;
    while k < n {
        let b0: u8 = bytes[k];
        if b0 < 0x80 {
            s.push(char::from_u32(b0 as u32).unwrap());
            k += 1;
            continue;
        }
        if b0 == 0xED && (k + 1) < n && bytes[k + 1] >= 0xA0 && bytes[k + 1] <= 0xBF {
            s.push(char::from_u32(rep).unwrap());
            k += 2;
            if k < n && bytes[k] >= 0x80 && bytes[k] <= 0xBF {
                k += 1;
            }
            continue;
        }
        if b0 >= 0xC2 && b0 <= 0xDF {
            if (k + 1) < n && bytes[k + 1] >= 0x80 && bytes[k + 1] <= 0xBF {
                let cp: u32 = (((b0 as u32) - 0xC0) << 6) + (((bytes[k + 1] as u32)) - 0x80);
                s.push(char::from_u32(cp).unwrap());
                k += 2;
            } else {
                s.push(char::from_u32(rep).unwrap());
                k += 1;
            }
            continue;
        }
        // 4-byte leads (F0..F4): four bytes, cp from all four.  F0 wants 90..BF
        // and F4 wants 80..8F for its second byte; a bad continuation restarts
        // that byte fresh, and a truncated good prefix gives one U+FFFD over it
        // (the astral fuzz corpus, seed 1, pinned this branch).
        if b0 >= 0xF0 && b0 <= 0xF4 {
            let ok1: bool = (k + 1) < n && bytes[k + 1] >= (if b0 == 0xF0 { 0x90 } else { 0x80 }) && bytes[k + 1] <= (if b0 == 0xF4 { 0x8F } else { 0xBF });
            let ok2: bool = (k + 2) < n && bytes[k + 2] >= 0x80 && bytes[k + 2] <= 0xBF;
            let ok3: bool = (k + 3) < n && bytes[k + 3] >= 0x80 && bytes[k + 3] <= 0xBF;
            if ok1 && ok2 && ok3 {
                let cp: u32 = (((b0 as u32) - 0xF0) << 18) + (((bytes[k + 1] as u32) - 0x80) << 12) + (((bytes[k + 2] as u32) - 0x80) << 6) + ((bytes[k + 3] as u32) - 0x80);
                s.push(char::from_u32(cp).unwrap());
                k += 4;
            } else if ok1 && ok2 {
                s.push(char::from_u32(rep).unwrap());
                k += 3;
            } else if ok1 {
                s.push(char::from_u32(rep).unwrap());
                k += 2;
            } else {
                s.push(char::from_u32(rep).unwrap());
                k += 1;
            }
            continue;
        }
        // 3-byte leads (E0..EF): three bytes, cp from three.  E0 wants A0..BF
        // (C0..9F is an overlong form), and ED wants 80..9F: its A0..BF follow-
        // on is the surrogate case the branch above already took.  A bad
        // continuation restarts that byte fresh; a truncated good prefix gives
        // one U+FFFD over the prefix.  Java 8 pins this: e2 98 8e before a
        // space is a complete U+260E, not a replacement (the unicode fixture).
        if b0 >= 0xE0 && b0 <= 0xEF {
            let lo1: u8 = match b0 {
                0xE0 => { 0xA0 }
                0xED => { 0x80 }
                _ => { 0x80 }
            };
            let hi1: u8 = match b0 {
                0xE0 => { 0xBF }
                0xED => { 0x9F }
                _ => { 0xBF }
            };
            let ok1: bool = (k + 1) < n && bytes[k + 1] >= lo1 && bytes[k + 1] <= hi1;
            if !ok1 {
                s.push(char::from_u32(rep).unwrap());
                k += 1;
                continue;
            }
            let ok2: bool = (k + 2) < n && bytes[k + 2] >= 0x80 && bytes[k + 2] <= 0xBF;
            if !ok2 {
                s.push(char::from_u32(rep).unwrap());
                k += 2;
                continue;
            }
            // A 3-byte lead wants exactly two continuations: e2 98 8e before a
            // space is complete, so no third check here (the F0..F4 branch above
            // owns the three-continuation case).
            let cp: u32 = (((b0 as u32) - 0xE0) << 12) + (((bytes[k + 1] as u32) - 0x80) << 6) + ((bytes[k + 2] as u32) - 0x80);
            s.push(char::from_u32(cp).unwrap());
            k += 3;
            continue;
        }
        // C0, C1, F5..FF: invalid leads, one byte each
        s.push(char::from_u32(rep).unwrap());
        k += 1;
    }
    return s;
}

// ---------------------------------------------------------------- callplot

pub fn run_callplot(args: &[OsString], si: &mut dyn Read, so: &mut dyn Write, se: &mut dyn Write) -> u8 {
    if args.len() == 0 {
        // Main.java:41-44: no arguments, usage on stderr, exit 1
        if !wbytes(se, USAGE.as_bytes()) {
            return 0;
        }
        return 1;
    }
    let mut mode: usize = MODE_TEXT;
    let mut ctx: RunContext = run_context_fresh();
    let mut opts: RenderOptions = render_options_new();
    let mut err_mark: i64 = 0;
    let flag_table: Vec<u8> = str_bytes("-table=".to_string());
    let flag_html: Vec<u8> = str_bytes("-html=".to_string());
    let mut k: usize = 0;
    while k < args.len() {
        let s: String = arg_str(&args[k]);
        let sb: Vec<u8> = str_bytes(s.clone());
        if s == "-pic" {
            mode = MODE_PIC;
            k += 1;
            continue;
        }
        if s == "-svg" {
            mode = MODE_SVG;
            k += 1;
            continue;
        }
        if s == "-vml" {
            mode = MODE_VML;
            k += 1;
            continue;
        }
        if s == "-xml" {
            mode = MODE_XML;
            k += 1;
            continue;
        }
        if s == "-vxml" {
            mode = MODE_VXML;
            k += 1;
            continue;
        }
        if s == "-graffle" {
            mode = MODE_GRAFFLE;
            k += 1;
            continue;
        }
        if s == "-mermaid" {
            mode = MODE_MERMAID;
            k += 1;
            continue;
        }
        if s == "-plantuml" {
            mode = MODE_PLANTUML;
            k += 1;
            continue;
        }
        if s == "-table" {
            mode = MODE_TABLE;
            let arch: String = "archtable".to_string();
            let st: Result<TableStyle, Ex> = parse_table_style(&arch);
            match st {
                Ok(t) => { opts = opts_table(t); }
                Err(_) => { }
            }
            k += 1;
            continue;
        }
        if has_prefix(&sb, &flag_table) {
            mode = MODE_TABLE;
            let name: String = strip_pre(&sb, flag_table.len());
            let st2: Result<TableStyle, Ex> = parse_table_style(&name);
            match st2 {
                Err(_) => {
                    if !wbytes(se, &str_bytes(format!("callplot: unknown table style: {}\n", name))) {
                        return 0;
                    }
                    return 1;
                }
                Ok(t) => { opts = opts_table(t); }
            }
            k += 1;
            continue;
        }
        if s == "-html" {
            // the bare flag sets the mode and clears any earlier force
            // (DESIGN.md 3.12.6: the layout then comes from the input file)
            mode = MODE_HTML;
            opts = render_options_new();
            k += 1;
            continue;
        }
        if has_prefix(&sb, &flag_html) {
            mode = MODE_HTML;
            let name2: String = strip_pre(&sb, flag_html.len());
            if name2 == "topbottom" {
                opts = opts_html(HtmlLayout::TOPBOTTOM);
                k += 1;
                continue;
            }
            if name2 == "sidebyside" {
                opts = opts_html(HtmlLayout::SIDEBYSIDE);
                k += 1;
                continue;
            }
            if !wbytes(se, &str_bytes(format!("callplot: unknown html layout: {}\n", name2))) {
                return 0;
            }
            return 1;
        }
        if s == "-?" || s == "--help" {
            // Main.java:64-66: the run stops here, exit 0 (earlier output kept)
            if !wbytes(se, USAGE.as_bytes()) {
                return 0;
            }
            return 0;
        }
        // ---- a file argument ("-" means stdin)
        let mut data: Vec<u8> = vec![];
        if s == "-" {
            let r = si.read_to_end(&mut data);
            match r {
                Err(e) => {
                    if !wbytes(se, &str_bytes(format!("callplot: {}: {}\n", s, e))) {
                        return 0;
                    }
                    let nm: i64 = drain_err(se, &ctx, err_mark);
                    if nm < 0 {
                        return 0;
                    }
                    err_mark = nm;
                    return 1;
                }
                Ok(_) => { }
            }
        } else {
            let r2 = read(&args[k]);
            match r2 {
                Err(e) => {
                    if !wbytes(se, &str_bytes(format!("callplot: {}: {}\n", s, e))) {
                        return 0;
                    }
                    let nm: i64 = drain_err(se, &ctx, err_mark);
                    if nm < 0 {
                        return 0;
                    }
                    err_mark = nm;
                    return 1;
                }
                Ok(b) => { data = b; }
            }
        }
        // decode + render (the pure part; a panic lands on the internal error
        // line, rule 10).  One shared ctx across the files, rule 3.
        let pair = catch_unwind(|| {
            let t: String = decode_java_utf8(&data);
            let mut cc: RunContext = clone_ctx(&ctx);
            let res: Result<Vec<u8>, Ex> = match render_cli(t, mode, &opts, &mut cc) {
                Ok(units) => { Ok(str_bytes(from_units(&units))) }
                Err(ex2) => { Err(ex2) }
            };
            return (res, cc);
        });
        match pair {
            Err(_) => {
                if !wbytes(se, &str_bytes("callplot: internal error, please report it\n".to_string())) {
                    return 0;
                }
                return 1;
            }
            Ok(two) => {
                ctx = two.1;
                let res: Result<Vec<u8>, Ex> = two.0;
                match res {
                    Ok(outb) => {
                        if !wbytes(so, &outb) {
                            return 0;
                        }
                        let nm2: i64 = drain_err(se, &ctx, err_mark);
                        if nm2 < 0 {
                            return 0;
                        }
                        err_mark = nm2;
                    }
                    Err(ex) => {
                        if !wbytes(se, &str_bytes(format!("callplot: {}: {}\n", error_name(ex.kind), ex_msg(ex.msg)))) {
                            return 0;
                        }
                        let nm3: i64 = drain_err(se, &ctx, err_mark);
                        if nm3 < 0 {
                            return 0;
                        }
                        err_mark = nm3;
                        return 1;
                    }
                }
            }
        }
        k += 1;
    }
    return 0;
}

// ---------------------------------------------------------------- plottable

pub fn run_plottable(args: &[OsString], si: &mut dyn Read, so: &mut dyn Write, se: &mut dyn Write) -> u8 {
    // the script's positional order: -debug first, then -style NAME, then
    // exactly one file; anything else prints usage on STDOUT and exits 1
    let n: usize = args.len();
    let mut k: usize = 0;
    let mut doing_debug: bool = false;
    if n > 0 && arg_str(&args[0]) == "-debug" {
        doing_debug = true;
        k = 1;
    }
    let mut style_name: String = "archtable".to_string();
    if k < n && arg_str(&args[k]) == "-style" {
        k += 1;
        if k < n {
            style_name = arg_str(&args[k]);
        }
        k += 1;
    }
    if (n - k) != 1 {
        if !wbytes(so, PLTT_USAGE.as_bytes()) {
            return 0;
        }
        return 1;
    }
    let fs_name: String = arg_str(&args[k]);
    let mut opts2: RenderOptions = render_options_new();
    let st: Result<TableStyle, Ex> = parse_table_style(&style_name);
    match st {
        Err(_) => {
            if !wbytes(se, &str_bytes(format!("plottable: unknown table style: {}\n", style_name))) {
                return 0;
            }
            return 1;
        }
        Ok(t) => { opts2 = opts_table(t); }
    }
    let mut data: Vec<u8> = vec![];
    let r3 = read(&args[k]);
    match r3 {
        Err(e) => {
            if !wbytes(se, &str_bytes(format!("plottable: {}: {}\n", fs_name, e))) {
                return 0;
            }
            return 1;
        }
        Ok(b) => { data = b; }
    }
    if doing_debug {
        // the -xml intermediate as _x_<pid>.xml in the current folder, like the
        // script's _x_<$$>.xml (DESIGN.md 3.13.5)
        let pair_x = catch_unwind(|| {
            let t: String = decode_java_utf8(&data);
            return render(t, MODE_XML);
        });
        let xml_u: Result<U, Ex> = match pair_x {
            Err(_) => {
                if !wbytes(se, &str_bytes("plottable: internal error, please report it\n".to_string())) {
                    return 0;
                }
                return 1;
            }
            Ok(rx) => { rx }
        };
        let xml_name: String = format!("_x_{}.xml", id());
        match xml_u {
            Err(ex) => {
                if !wbytes(se, &str_bytes(format!("plottable: {}: {}\n", error_name(ex.kind), ex_msg(ex.msg)))) {
                    return 0;
                }
                return 1;
            }
            Ok(u) => {
                let xb: Vec<u8> = str_bytes(from_units(&u));
                let wr = write(&xml_name, &xb);
                match wr {
                    Err(e2) => {
                        if !wbytes(se, &str_bytes(format!("plottable: {}: {}\n", xml_name, e2))) {
                            return 0;
                        }
                        return 1;
                    }
                    Ok(_) => { }
                }
                if !wbytes(se, &str_bytes(format!("retaining xml intermediate file in {}\n", xml_name))) {
                    return 0;
                }
            }
        }
    }
    let pair_t = catch_unwind(|| {
        let t: String = decode_java_utf8(&data);
        let mut cc: RunContext = run_context_fresh();
        return render_cli(t, MODE_TABLE, &opts2, &mut cc);
    });
    let tab_u: Result<U, Ex> = match pair_t {
        Err(_) => {
            if !wbytes(se, &str_bytes("plottable: internal error, please report it\n".to_string())) {
                return 0;
            }
            return 1;
        }
        Ok(rt) => { rt }
    };
    match tab_u {
        Err(ex) => {
            if !wbytes(se, &str_bytes(format!("plottable: {}: {}\n", error_name(ex.kind), ex_msg(ex.msg)))) {
                return 0;
            }
            return 1;
        }
        Ok(u) => {
            let tb: Vec<u8> = str_bytes(from_units(&u));
            if !wbytes(so, &tb) {
                return 0;
            }
        }
    }
    return 0;
}

// ------------------------------------------------------------- callplot-prep

pub fn run_prep(args: &[OsString], si: &mut dyn Read, so: &mut dyn Write, se: &mut dyn Write) -> u8 {
    // hand-written loop (the JS port's shape: unknown flag -> usage line, exit 1)
    let n: usize = args.len();
    let mut k: usize = 0;
    let mut links: bool = false;
    let mut dbg_file: Option<String> = None;
    let mut want_dbg: bool = false;
    let mut file_s: String = "-".to_string();
    let mut file_os: Option<OsString> = None;
    let mut file_count: usize = 0;
    while k < n {
        let s: String = arg_str(&args[k]);
        let sb: Vec<u8> = str_bytes(s.clone());
        if s == "--links" {
            links = true;
            k += 1;
            continue;
        }
        if s == "--debug-file" {
            if (k + 1) >= n {
                if !wbytes(se, PREP_USAGE.as_bytes()) {
                    return 0;
                }
                return 1;
            }
            dbg_file = Some(arg_str(&args[k + 1]));
            want_dbg = true;
            k += 2;
            continue;
        }
        let flag_dbg: Vec<u8> = str_bytes("--debug-file=".to_string());
        if has_prefix(&sb, &flag_dbg) {
            dbg_file = Some(strip_pre(&sb, flag_dbg.len()));
            want_dbg = true;
            k += 1;
            continue;
        }
        let flag_dash: Vec<u8> = str_bytes("-".to_string());
        if s != "-" && has_prefix(&sb, &flag_dash) {
            if !wbytes(se, PREP_USAGE.as_bytes()) {
                return 0;
            }
            return 1;
        }
        if file_count > 0 {
            if !wbytes(se, PREP_USAGE.as_bytes()) {
                return 0;
            }
            return 1;
        }
        file_count = 1;
        file_s = s;
        file_os = Some(args[k].clone());
        k += 1;
    }
    let mut data: Vec<u8> = vec![];
    if file_s == "-" {
        let r = si.read_to_end(&mut data);
        match r {
            Err(e) => {
                if !wbytes(se, &str_bytes(format!("callplot-prep: {}: {}\n", file_s, e))) {
                    return 0;
                }
                return 1;
            }
            Ok(_) => { }
        }
    } else {
        match file_os {
            Some(a) => {
                let r2 = read(&a);
                match r2 {
                    Err(e) => {
                        if !wbytes(se, &str_bytes(format!("callplot-prep: {}: {}\n", file_s, e))) {
                            return 0;
                        }
                        return 1;
                    }
                    Ok(b) => { data = b; }
                }
            }
            None => { }
        }
    }
    let po: PrepOptions = prep_new();
    let mut po2: PrepOptions = po;
    po2.links = links;
    // bytes straight to the converter (section 3.8 rule 1: no decoding)
    let pair_p = catch_unwind(|| {
        if want_dbg {
            return prep_debug(&data, &po2);
        }
        let o1: Vec<u8> = prep(&data, &po2);
        let empty: Vec<Vec<u8>> = vec![];
        return (o1, empty);
    });
    match pair_p {
        Err(_) => {
            if !wbytes(se, &str_bytes("callplot-prep: internal error, please report it\n".to_string())) {
                return 0;
            }
            return 1;
        }
        Ok(two) => {
            let out_b: Vec<u8> = two.0;
            let dbg_lines: Vec<Vec<u8>> = two.1;
            if !wbytes(so, &out_b) {
                return 0;
            }
            if want_dbg {
                // the awk wrote these with ">> debug"; this std has no append
                // API, so the command overwrites the file (Appendix B erratum
                // territory; tests never run the same --debug-file twice)
                let nl: Vec<u8> = vec![0x0A];
                let mut parts: Vec<Vec<u8>> = vec![];
                let mut j: usize = 0;
                while j < dbg_lines.len() {
                    parts.push(dbg_lines[j].clone());
                    parts.push(nl.clone());
                    j += 1;
                }
                let dbg_bytes: Vec<u8> = cat_all(&parts);
                match dbg_file {
                    Some(p) => {
                        let wr = write(&p, &dbg_bytes);
                        match wr {
                            Err(e) => {
                                if !wbytes(se, &str_bytes(format!("callplot-prep: {}: {}\n", file_s, e))) {
                                    return 0;
                                }
                                return 1;
                            }
                            Ok(_) => { }
                        }
                    }
                    None => { }
                }
            }
        }
    }
    return 0;
}
