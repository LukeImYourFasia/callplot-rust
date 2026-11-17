// Parser.java:37-166, ported step by step (DESIGN.md section 3.1).
// The quirks in DESIGN.md section 3.8 are intentional. Do not "clean it up".

use crate::errors::Ex;
use crate::errors::exc;
use crate::errors::ErrorKind;
use crate::jcompat::U;
use crate::jcompat::read_lines;
use crate::jcompat::java_trim;
use crate::jcompat::java_split_slash;
use crate::jcompat::replace_slash;
use crate::jcompat::u_startsWith;
use crate::jcompat::u_eq;
use crate::jcompat::ul_a;
use crate::jcompat::u_len;
use crate::jcompat::u_at;
use crate::jcompat::u_sub;
use crate::model::Player;
use crate::model::player_new;
use crate::model::Message;
use crate::model::message_new;
use crate::model::message_new2;
use crate::flow::Flow;
use crate::flow::OPair;
use crate::flow::flow_new;
use crate::flow::opt_put;

// Parser.java:75 regex: split on every / whose preceding char is not \.
// Position 0 never splits (there is no preceding char).

// Java String.indexOf of a substring, or -1.
pub fn u_str_index(s: &U, sub: &U) -> i64 {
    let n: i64 = u_len(s);
    let m: i64 = u_len(sub);
    if m == 0 {
        return 0;
    }
    let mut i: i64 = 0;
    while i + m <= n {
        let mut ok: bool = true;
        let mut k: i64 = 0;
        while k < m {
            if u_at(s, i + k) != u_at(sub, k) {
                ok = false;
                k = m;
            }
            k += 1;
        }
        if ok {
            return i;
        }
        i += 1;
    }
    return -1;
}

// Parser.java:85 indexing: a missing field throws IndexError.
pub fn at(parts: &Vec<U>, k: i64) -> Result<U, Ex> {
    if k < 0 || k >= parts.len() as i64 {
        return Err(exc(ErrorKind::INDEX, "Index out of range".to_string()));
    }
    return Ok(parts[k as usize].clone());
}

pub fn parse(text: &U) -> Result<Flow, Ex> {
    let mut players: Vec<Player> = vec![];
    let mut messages: Vec<Message> = vec![];
    let mut opts: Vec<OPair> = vec![];

    let mut last_msg: Option<Message> = None;
    let mut dummy: bool = false;
    let mut new_dummy: bool = false;
    let mut line_no: i64 = 0;

    for line0 in read_lines(text) {
        let mut line: U = line0;
        line_no = line_no + 1;
        let t_line: U = java_trim(line.clone());

        // Parser.java:56
        if u_startsWith(&t_line, &ul_a("#".to_string())) || u_len(&t_line) == 0 {
            continue;
        }

        // Parser.java:60-66
        if u_startsWith(&t_line, &ul_a("!!".to_string())) {
            match last_msg {
                None => {
                    return Err(exc(ErrorKind::NULL_POINTER, "".to_string()));
                }
                Some(m) => {
                    let mut mm: Message = m;
                    mm.notes.push(u_sub(&t_line, 2, u_len(&t_line) - 2));
                    last_msg = Some(mm);
                }
            }
            continue;
        }
        if u_startsWith(&t_line, &ul_a("!".to_string())) {
            match last_msg {
                None => {
                    return Err(exc(ErrorKind::NULL_POINTER, "".to_string()));
                }
                Some(m) => {
                    let mut mm: Message = m;
                    mm.comments.push(u_sub(&t_line, 1, u_len(&t_line) - 1));
                    last_msg = Some(mm);
                }
            }
            continue;
        }

        // Parser.java:69-72: slices the RAW line, not the trimmed line
        if u_startsWith(&t_line, &ul_a("-".to_string())) {
            new_dummy = true;
            line = u_sub(&line, 1, u_len(&line) - 1);
        }

        // Parser.java:75
        let split_line_escaped: Vec<U> = java_split_slash(line);

        // Parser.java:78-83: unescape \/ for all fields except the command
        let mut parts: Vec<U> = vec![];
        let nsp: usize = split_line_escaped.len();
        let mut si: usize = 0;
        while si < nsp {
            if si == 0 {
                parts.push(split_line_escaped[si].clone());
            } else {
                parts.push(replace_slash(split_line_escaped[si].clone()));
            }
            si += 1;
        }

        // Parser.java:85: command = splitLine[tkn++]
        let command: U = at(&parts, 0)?;
        let mut tkn: i64 = 1;

        if u_eq(&command, &ul_a("guy".to_string())) {
            // Parser.java:86-89
            let name: U = at(&parts, tkn)?;
            tkn = tkn + 1;
            let caption: U = at(&parts, tkn)?;
            tkn = tkn + 1;
            players.push(player_new(name, caption));
        } else if u_eq(&command, &ul_a("note".to_string())) {
            // Parser.java:90-100
            let from_: U = at(&parts, tkn)?;
            tkn = tkn + 1;
            if !dummy {
                match last_msg {
                    None => {
                    }
                    Some(m) => {
                        messages.push(m.clone());
                    }
                }
            }
            dummy = new_dummy;
            new_dummy = false;
            let cap: U = at(&parts, tkn)?;
            tkn = tkn + 1;
            let mut m: Message = message_new(from_.clone(), from_.clone(), cap, 0x20, false, false);
            m.src_line = line_no;
            last_msg = Some(m);
        } else if u_eq(&command, &ul_a("opt".to_string())) {
            // Parser.java:101-106: values are not trimmed; later put wins
            let name: U = at(&parts, tkn)?;
            tkn = tkn + 1;
            let value: U = at(&parts, tkn)?;
            tkn = tkn + 1;
            opt_put(&mut opts, name, value);
        } else if u_str_index(&command, &ul_a("->".to_string())) >= 0 {
            // Parser.java:107-132
            let arrow_idx: i64 = u_str_index(&command, &ul_a("->".to_string()));
            let from_: U = java_trim(u_sub(&command, 0, arrow_idx));
            let to: U = java_trim(u_sub(&command, arrow_idx + 2, u_len(&command) - (arrow_idx + 2)));
            let mut reply: bool = false;
            while tkn < parts.len() as i64 {
                if !dummy {
                    match last_msg {
                        None => {
                        }
                        Some(m) => {
                            messages.push(m.clone());
                        }
                    }
                }
                dummy = new_dummy;
                new_dummy = false;
                let cap: U = at(&parts, tkn)?;
                tkn = tkn + 1;
                let mut m: Message;
                if reply {
                    m = message_new(to.clone(), from_.clone(), cap, 0x2D, false, true);
                } else {
                    m = message_new(from_.clone(), to.clone(), cap, 0x2D, false, true);
                }
                m.src_line = line_no;
                m.is_reply = reply;
                last_msg = Some(m);
                reply = !reply;
            }
        } else if u_str_index(&command, &ul_a("..".to_string())) >= 0 {
            // Parser.java:133-145: filler '.', no arrows; extra fields ignored
            let dot_idx: i64 = u_str_index(&command, &ul_a("..".to_string()));
            let from_: U = java_trim(u_sub(&command, 0, dot_idx));
            let to: U = java_trim(u_sub(&command, dot_idx + 2, u_len(&command) - (dot_idx + 2)));
            if !dummy {
                match last_msg {
                    None => {
                    }
                    Some(m) => {
                        messages.push(m.clone());
                    }
                }
            }
            dummy = new_dummy;
            new_dummy = false;
            let cap: U = at(&parts, tkn)?;
            tkn = tkn + 1;
            let mut m: Message = message_new(from_, to, cap, 0x2E, false, false);
            m.src_line = line_no;
            last_msg = Some(m);
        } else if u_str_index(&command, &ul_a("--".to_string())) >= 0 {
            // Parser.java:146-159: like .. but repeat_label=true (filler '.' too)
            let dot_idx: i64 = u_str_index(&command, &ul_a("--".to_string()));
            let from_: U = java_trim(u_sub(&command, 0, dot_idx));
            let to: U = java_trim(u_sub(&command, dot_idx + 2, u_len(&command) - (dot_idx + 2)));
            if !dummy {
                match last_msg {
                    None => {
                    }
                    Some(m) => {
                        messages.push(m.clone());
                    }
                }
            }
            dummy = new_dummy;
            new_dummy = false;
            let cap: U = at(&parts, tkn)?;
            tkn = tkn + 1;
            let mut m: Message = message_new2(from_, to, cap, 0x2E, false, false, true);
            m.src_line = line_no;
            last_msg = Some(m);
        }
        // Parser.java:159: anything else: ignore the line, no error
    }

    // Parser.java:161-164
    if !dummy {
        match last_msg {
            None => {
            }
            Some(m) => {
                messages.push(m.clone());
            }
        }
    }
    return Ok(flow_new(players, messages, opts));
}
