// TEXT and XML renderers (DESIGN.md section 3.7; text.py).
// Flow.java:315-419 (plotOn), Player.java:99-119 (drawSelf),
// Message.java:126-182 (drawSelf), Message.java:192-224 (printComment).
// One buffer line at a time, exactly like the Java char[].

use crate::constants::MODE_XML;
use crate::errors::Ex;
use crate::errors::ErrorKind;
use crate::jcompat::U;
use crate::jcompat::CharBuf;
use crate::jcompat::char_buf_new;
use crate::jcompat::buf_get;
use crate::jcompat::buf_set;
use crate::jcompat::buf_copy_in;
use crate::jcompat::buf_slice;
use crate::jcompat::fmt_i;
use crate::jcompat::idiv;
use crate::jcompat::parse_java_int;
use crate::jcompat::ul_a;
use crate::jcompat::u_len;
use crate::jcompat::u_at;
use crate::jcompat::exc_index;
use crate::model::Message;
use crate::model::Player;
use crate::flow::Flow;
use crate::flow::label_line;
use crate::flow::column_of;
use crate::flow::next_message_number;
use crate::flow::start_message_numbers;
use crate::flow::should_print;
use crate::flow::get_total_width;
use crate::flow::get_total_height;

// arr_at on a list of unit strings: a missing element throws IndexError.
fn arr_at(v: &Vec<U>, k: i64) -> Result<U, Ex> {
    if k < 0 || k >= v.len() as i64 {
        return Err(exc_index());
    }
    return Ok(v[k as usize].clone());
}

// Append one unit string plus a newline to out.
fn push_units(out: &mut U, t: &U) {
    for c in t {
        out.push(*c);
    }
}

fn write_units_line(out: &mut U, text: &U) {
    push_units(out, text);
    out.push(0x0A);
}

pub fn text_on(f: &mut Flow, mode: usize) -> Result<U, Ex> {
    let mut out: U = vec![];
    if (mode & MODE_XML) != 0 {
        write_units_line(&mut out, &ul_a("<plot>".to_string()));
        write_units_line(&mut out, &ul_a("<picture>".to_string()));
    }

    let lines: Vec<U> = draw_text_lines(f, mode, None)?;
    let nl: usize = lines.len();
    let mut k: usize = 0;
    while k < nl {
        write_units_line(&mut out, &lines[k]);
        k += 1;
    }

    if (mode & MODE_XML) != 0 {
        write_units_line(&mut out, &ul_a("</picture>".to_string()));
        let nm: usize = f.messages.len();
        let mut i: usize = 0;
        while i < nm {
            print_comment(&f.messages[i], &mut out);
            i += 1;
        }
        write_units_line(&mut out, &ul_a("</plot>".to_string()));
    }
    return Ok(out);
}

// Flow.java:320-399: the picture-content line loop.  Returns the lines
// instead of writing them so the table pipeline can reuse them.  forceLinks
// None keeps Flow.links.
pub fn draw_text_lines(f: &mut Flow, mode: usize, force_links: Option<bool>) -> Result<Vec<U>, Ex> {
    start_message_numbers(f);
    let links: bool;
    match force_links {
        None => {
            links = f.links;
        }
        Some(b) => {
            links = b;
        }
    }

    let width: i64 = get_total_width(f);
    let mut buf: CharBuf = char_buf_new(width)?;
    let mut lines: Vec<U> = vec![];

    let total_height: i64 = get_total_height(f);
    let mut line: i64 = 0;
    while line <= total_height {
        let mut i: i64 = 0;
        while i < width {
            buf = buf_set(&buf, i, 0x20)?;
            i += 1;
        }

        let np: usize = f.players.len();
        let mut p: usize = 0;
        while p < np {
            buf = draw_player(f, f.players[p].clone(), line, buf)?;
            p += 1;
        }
        let nm: usize = f.messages.len();
        let mut m: usize = 0;
        while m < nm {
            buf = draw_message(f, m, line, buf)?;
            m += 1;
        }

        if should_print(f, &buf) {
            let mut last_char: i64 = width - 1;
            while buf_get(&buf, last_char)? == 0x20 && last_char > 0 {
                last_char -= 1;
            }
            if (mode & MODE_XML) != 0 {
                lines.push(write_line_xml(f, &buf, last_char, links)?);
            } else {
                lines.push(buf_slice(&buf, 0, last_char + 1)?);
            }
        }
        line += 1;
    }
    return Ok(lines);
}

// Flow.java:359-395: XML-escape and turn (n) into links.  Reproduces the
// switch fallthrough: '(' falls to default unless the try block breaks.
fn write_line_xml(f: &Flow, buf: &CharBuf, last_char: i64, links: bool) -> Result<U, Ex> {
    let mut s: U = vec![];
    let mut i: i64 = 0;
    while i <= last_char {
        let ch: u16 = buf_get(&buf, i)?;
        let wrote: bool;
        if ch == 0x3C {
            push_units(&mut s, &ul_a("&lt;".to_string()));
            wrote = true;
        } else if ch == 0x3E {
            push_units(&mut s, &ul_a("&gt;".to_string()));
            wrote = true;
        } else if ch == 0x28 && links {
            let num_start: i64 = i + 1;
            while i <= last_char && buf_get(&buf, i)? != 0x29 {
                i += 1;
            }
            let mut linked: bool = false;
            let closing: u16 = buf_get(&buf, i)?;
            if closing == 0x29 {
                let seg: U = buf_slice(&buf, num_start, i - num_start)?;
                match parse_java_int(&seg) {
                    Ok(num) => {
                        push_units(&mut s, &ul_a("(<a href='#t".to_string()));
                        push_units(&mut s, &fmt_i(num));
                        push_units(&mut s, &ul_a("'>".to_string()));
                        push_units(&mut s, &fmt_i(num));
                        push_units(&mut s, &ul_a("</a>)<a name='m".to_string()));
                        push_units(&mut s, &fmt_i(num));
                        push_units(&mut s, &ul_a("'/>".to_string()));
                        linked = true;
                    }
                    Err(e) => {
                        if e.kind != ErrorKind::NUMBER_FORMAT {
                            return Err(e);
                        }
                    }
                }
            }
            if linked {
                i += 1;
                continue;
            }
            i = num_start - 1;
            wrote = false;
        } else {
            wrote = false;
        }
        if !wrote {
            s.push(buf_get(&buf, i)?);
        }
        i += 1;
    }
    return Ok(s);
}

// Player.java:99-119
fn draw_player(f: &Flow, p: Player, line: i64, buf: CharBuf) -> Result<CharBuf, Ex> {
    let line_in_caption: i64 = label_line(f, line);
    let image: U;
    if line_in_caption >= 0 {
        if line_in_caption < p.caption.len() as i64 {
            image = p.caption[line_in_caption as usize].clone();
        } else {
            // blank = arrayOf(" ") (Player.java:51): one space
            image = ul_a(" ".to_string());
        }
    } else {
        // glyph (Player.java:50)
        image = ul_a("|".to_string());
    }
    let ilen: i64 = u_len(&image);
    if ilen == 1 {
        return buf_set(&buf, p.column, u_at(&image, 0));
    }
    let bias: i64 = idiv(ilen, 2)?;
    return buf_copy_in(&buf, &image, p.column - bias, ilen);
}

// Message.java:126-182
fn draw_message(f: &mut Flow, idx: usize, line: i64, mut buf: CharBuf) -> Result<CharBuf, Ex> {
    let m: Message = f.messages[idx].clone();
    let fcol: i64 = column_of(f, &m.from_)?;
    let tcol: i64 = column_of(f, &m.to)?;
    let locol: i64 = if fcol < tcol { fcol } else { tcol };
    let hicol: i64 = if fcol < tcol { tcol } else { fcol };

    let offset: i64 = m.line - line;
    let clen: i64 = m.caption.len() as i64;

    if offset == 0 && locol != hicol {
        let mut j: i64 = locol + 1;
        while j < hicol {
            buf = buf_set(&buf, j, m.filler)?;
            j += 1;
        }
        if (m.arrow_from && locol == fcol) || (m.arrow_to && locol == tcol) {
            buf = buf_set(&buf, locol + 1, 0x3C)?;
        }
        if (m.arrow_from && hicol == fcol) || (m.arrow_to && hicol == tcol) {
            buf = buf_set(&buf, hicol - 1, 0x3E)?;
        }
    } else if offset == clen && (locol != hicol || f.do_note_numbering) {
        let num: i64 = next_message_number(f);
        let mut final_caption: U;
        if num != -1 {
            f.messages[idx].my_number = num;
            final_caption = ul_a("(".to_string());
            push_units(&mut final_caption, &fmt_i(num));
            push_units(&mut final_caption, &ul_a(") ".to_string()));
            push_units(&mut final_caption, &arr_at(&m.caption, 0)?);
        } else {
            final_caption = arr_at(&m.caption, 0)?;
        }
        buf = copy_caption(buf, locol, &final_caption)?;
    } else if 0 < offset && offset <= clen {
        let idx2: i64 = clen - offset;
        buf = copy_caption(buf, locol, &arr_at(&m.caption, idx2)?)?;
    }
    return Ok(buf);
}

// Message.java:162-169 and 173-180: clamp the copy, mark truncation with '*'
fn copy_caption(buf: CharBuf, locol: i64, text: &U) -> Result<CharBuf, Ex> {
    let width: i64 = u_len(&buf.a);
    let tlen: i64 = u_len(text);
    let mut copy_size: i64 = tlen;
    if locol + 1 + tlen > width {
        copy_size = width - (locol + 1);
    }
    let mut out: CharBuf = buf_copy_in(&buf, text, locol + 1, copy_size)?;
    if copy_size != tlen {
        out = buf_set(&out, width - 1, 0x2A)?;
    }
    return Ok(out);
}

// Message.java:192-224 (XML epilogue; the Graffle epilogue shares it)
pub fn print_comment(m: &Message, out: &mut U) {
    if m.my_number != 0 {
        write_units_line(out, &ul_a("<message>".to_string()));
        let mut num_line: U = ul_a("<number>".to_string());
        push_units(&mut num_line, &fmt_i(m.my_number));
        push_units(&mut num_line, &ul_a("</number>".to_string()));
        write_units_line(out, &num_line);
        write_units_line(out, &ul_a("<captions>".to_string()));
        let nc: usize = m.caption.len();
        let mut i: usize = 0;
        while i < nc {
            let mut cap_line: U = ul_a("<caption>".to_string());
            push_units(&mut cap_line, &m.caption[i]);
            push_units(&mut cap_line, &ul_a("</caption>".to_string()));
            write_units_line(out, &cap_line);
            i += 1;
        }
        write_units_line(out, &ul_a("</captions>".to_string()));
        write_units_line(out, &ul_a("<notes>".to_string()));
        let nn: usize = m.notes.len();
        let mut j: usize = 0;
        while j < nn {
            write_units_line(out, &ul_a("<note>".to_string()));
            write_units_line(out, &m.notes[j]);
            write_units_line(out, &ul_a("</note>".to_string()));
            j += 1;
        }
        write_units_line(out, &ul_a("</notes>".to_string()));
        write_units_line(out, &ul_a("<comments>".to_string()));
        let ncm: usize = m.comments.len();
        let mut k: usize = 0;
        while k < ncm {
            write_units_line(out, &ul_a("<comment>".to_string()));
            write_units_line(out, &m.comments[k]);
            write_units_line(out, &ul_a("</comment>".to_string()));
            k += 1;
        }
        write_units_line(out, &ul_a("</comments>".to_string()));
        write_units_line(out, &ul_a("</message>".to_string()));
    }
}
