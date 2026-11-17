// Data model for players and messages (DESIGN.md section 3.2; model.py).
//
// Copyright (C) 2004 dynamicsoft, Inc.
//
// This program is free software; you can redistribute it and/or modify it
// under the terms of the GNU Lesser General Public License as published by
// the Free Software Foundation; either version 2.1 of the License, or (at
// your option) any later version.  A copy of the license is in LICENSE.

use crate::jcompat::U;
use crate::jcompat::tokenize;
use crate::jcompat::u_eq;
use crate::jcompat::u_ap;
use crate::jcompat::ul_a;

// Caption digest: StringTokenizer on "^" drops empties; PIC joins lines
// with quote-space-quote (Player.java:74-90, Message.java:72-88).
pub struct Dig {
    pub lines: Vec<U>,
    pub joined: U,
}

pub fn join_units(lines: Vec<U>, sep: &U) -> U {
    let mut out: U = vec![];
        let mut i: usize = 0;
    for t in &lines {
        if i > 0 {
            for c in sep {
                out.push(*c);
            }
        }
        for c in t {
            out.push(*c);
        }
        i += 1;
    }
    return out;
}

pub fn digest(caption_raw: &U) -> Dig {
    let lines: Vec<U> = tokenize(caption_raw.clone(), 0x5E);
    let sep: U = ul_a("\" \"".to_string());
    return Dig { lines: lines.clone(), joined: join_units(lines, &sep) };
}

#[derive(Clone, PartialEq)]
pub struct Player {
    pub name: U,
    pub caption_raw: U,
    pub caption: Vec<U>,
    pub caption_string: U,
    pub column: i64,
}

pub fn player_new(name: U, caption_raw: U) -> Player {
    let d: Dig = digest(&caption_raw);
    return Player {
        name: name,
        caption_raw: caption_raw,
        caption: d.lines,
        caption_string: d.joined,
        column: 10,
    };
}

pub fn player_caption_lines(p: &Player) -> i64 {
    return p.caption.len() as i64;
}

#[derive(Clone, PartialEq)]
pub struct Message {
    pub from_: U,
    pub to: U,
    pub caption_raw: U,
    pub filler: u16,
    pub arrow_from: bool,
    pub arrow_to: bool,
    pub repeat_label: bool,
    pub line: i64,
    pub my_number: i64,
    pub notes: Vec<U>,
    pub comments: Vec<U>,
    pub src_line: i64,
    pub is_reply: bool,
    pub caption: Vec<U>,
    pub caption_string: U,
}

pub fn message_new(from_: U, to: U, caption_raw: U, filler: u16, arrow_from: bool, arrow_to: bool) -> Message {
    return message_new2(from_, to, caption_raw, filler, arrow_from, arrow_to, false);
}

pub fn message_new2(from_: U, to: U, caption_raw: U, filler: u16, arrow_from: bool, arrow_to: bool, repeat_label: bool) -> Message {
    let d: Dig = digest(&caption_raw);
    return Message {
        from_: from_,
        to: to,
        caption_raw: caption_raw,
        filler: filler,
        arrow_from: arrow_from,
        arrow_to: arrow_to,
        repeat_label: repeat_label,
        line: 0,
        my_number: 0,
        notes: vec![],
        comments: vec![],
        src_line: 0,
        is_reply: false,
        caption: d.lines,
        caption_string: d.joined,
    };
}

pub fn message_length(m: &Message) -> i64 {
    return m.caption.len() as i64;
}

pub fn message_add_note(m: &mut Message, note: U) {
    m.notes.push(note);
}

pub fn message_add_comment(m: &mut Message, comment: U) {
    m.comments.push(comment);
}

pub fn is_self(m: &Message) -> bool {
    return u_eq(&m.from_, &m.to);
}

pub fn is_undirected(m: &Message) -> bool {
    return !m.arrow_from && !m.arrow_to;
}

pub fn is_note(m: &Message) -> bool {
    return is_self(m) && is_undirected(m);
}
