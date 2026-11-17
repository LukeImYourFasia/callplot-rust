// Table pipeline (DESIGN.md section 3.13; renderers/table/__init__.py + the
// six style modules).  No XML intermediate: draw the XML line stream, validate
// like Xalan does, then serialize the rows exactly like each .xsl writes.

use crate::constants::MODE_XML;
use crate::errors::Ex;
use crate::errors::ErrorKind;
use crate::errors::exc;
use crate::options::RenderOptions;
use crate::options::TableStyle;
use crate::options::parse_table_style;
use crate::jcompat::U;
use crate::jcompat::ul_a;
use crate::jcompat::str_bytes;
use crate::jcompat::fmt_i;
use crate::model::Message;
use crate::flow::Flow;
use crate::text::draw_text_lines;

pub fn push_units(out: &mut U, t: &U) {
    for c in t {
        out.push(*c);
    }
}

pub fn push_str(out: &mut U, s: &str) {
    push_units(out, &ul_a(s.to_string()));
}

struct Row {
    num: i64,
    captions: Vec<U>,
    notes: Vec<U>,
    comments: Vec<U>,
}

// --- shared cells ---

fn link_cell(num: i64) -> U {
    let mut s: U = ul_a("<td><message><a name=\"t".to_string());
    push_units(&mut s, &fmt_i_shim(num));
    push_str(&mut s, "\"/><a href=\"#m");
    push_units(&mut s, &fmt_i_shim(num));
    push_str(&mut s, "\">");
    push_units(&mut s, &fmt_i_shim(num));
    push_str(&mut s, "</a></message></td>");
    return s;
}

fn plain_cell(num: i64) -> U {
    let mut s: U = ul_a("<td><message>".to_string());
    push_units(&mut s, &fmt_i_shim(num));
    push_str(&mut s, "</message></td>");
    return s;
}

fn td_cell(html: &U) -> U {
    if html.len() == 0 {
        return ul_a("<td/>".to_string());
    }
    let mut s: U = ul_a("<td>".to_string());
    push_units(&mut s, html);
    push_str(&mut s, "</td>");
    return s;
}

fn note_or_comment_body(items: &Vec<U>) -> U {
    let mut s: U = vec![];
    for n in items {
        s.push(0x0A);
        push_units(&mut s, &n);
        s.push(0x0A);
        push_str(&mut s, "<br/>");
    }
    return s;
}

fn note_body(items: &Vec<U>) -> U {
    return note_or_comment_body(items);
}

fn comment_body(items: &Vec<U>) -> U {
    return note_or_comment_body(items);
}

fn caption_body(items: &Vec<U>) -> U {
    let mut s: U = vec![];
    for c in items {
        push_units(&mut s, &c);
        push_str(&mut s, "<br/>");
    }
    return s;
}

fn tt_cell(comments: &Vec<U>) -> U {
    let body: U = note_or_comment_body(comments);
    let mut s: U = ul_a("<td>".to_string());
    if body.len() == 0 {
        push_str(&mut s, "<tt/>");
    } else {
        push_str(&mut s, "<tt>");
        push_units(&mut s, &body);
        push_str(&mut s, "</tt>");
    }
    push_str(&mut s, "</td>");
    return s;
}

fn strong_cell(captions: &Vec<U>) -> U {
    let mut s: U = vec![];
    for c in captions {
        push_str(&mut s, "<strong>");
        push_units(&mut s, &c);
        push_str(&mut s, "</strong><br/>");
    }
    return s;
}

fn code_cell(notes: &Vec<U>) -> U {
    let mut s: U = vec![];
    for n in notes {
        push_str(&mut s, "<code>\n");
        push_units(&mut s, &n);
        push_str(&mut s, "\n</code><br/>");
    }
    return s;
}

// --- Xalan validation (section 3.13.4) ---

// Returns the end index just past the entity match at i, or -1.
fn entity_match_at(line: &U, i: i64) -> i64 {
    let n: i64 = u_len(line);
    if u_at(line, i) != 0x26 {
        return -1;
    }
    // &lt; &gt; &amp; &quot; &apos;
    let words: Vec<String> = vec!["lt;", "gt;", "amp;", "quot;", "apos;"].iter().map(|x| x.to_string()).collect();
    for w in words {
        if match_lit(line, i + 1, &w) {
            return i + 1 + slen(&w);
        }
    }
    // #digits; or #xhex+;
    if i + 1 < n && u_at(line, i + 1) == 0x23 {
        if i + 2 < n && (u_at(line, i + 2) == 0x78 || u_at(line, i + 2) == 0x58) {
            // '#x' — [0-9a-fA-F]+ then ';' (Python char class excludes x)
            let mut j: i64 = i + 3;
            while j < n && is_hexdigit(u_at(line, j)) {
                j += 1;
            }
            if j > i + 3 && j < n && u_at(line, j) == 0x3B {
                return j + 1;
            }
            return -1;
        }
        let mut j: i64 = i + 2;
        while j < n && is_digit(u_at(line, j)) {
            j += 1;
        }
        if j > i + 2 && j < n && u_at(line, j) == 0x3B {
            return j + 1;
        }
    }
    return -1;
}

fn is_hexdigit(c: u16) -> bool {
    return (c >= 0x30 && c <= 0x39) || (c >= 0x61 && c <= 0x66) || (c >= 0x41 && c <= 0x46);
}

fn is_digit(c: u16) -> bool {
    return c >= 0x30 && c <= 0x39;
}

fn match_lit(line: &U, at: i64, w: &str) -> bool {
    let wb: Vec<u8> = str_bytes(w.to_string());
    let mut k: i64 = 0;
    while k < wb.len() as i64 {
        if at + k >= u_len(line) || u_at(line, at + k) != wb[k as usize] as u16 {
            return false;
        }
        k += 1;
    }
    return true;
}

fn slen(w: &str) -> i64 {
    return str_bytes(w.to_string()).len() as i64;
}

fn u_to_string(s: &U) -> String {
    return crate::jcompat::from_units(s);
}

fn u_len(s: &U) -> i64 {
    return s.len() as i64;
}

fn u_at(s: &U, i: i64) -> u16 {
    if i < 0 || i >= s.len() as i64 {
        return 0;
    }
    return s[i as usize];
}

fn str_to_u8(s: String) -> Vec<u8> {
    return str_bytes(s);
}

// check_picture_line: bare & anywhere throws TableInputError.
fn check_picture_line(line: &U) -> Result<(), Ex> {
    let n: i64 = u_len(line);
    let mut i: i64 = 0;
    while i < n {
        if u_at(line, i) == 0x26 && entity_match_at(line, i) == -1 {
            return Err(exc(ErrorKind::TABLE_INPUT, "picture text contains a bare '&'".to_string()));
        }
        i += 1;
    }
    return Ok(());
}

// Xalan's serializer re-quotes the copied link attributes (rule 3).
fn fix_link_quotes(line: &U) -> U {
    let mut out: U = vec![];
    let n: i64 = u_len(line);
    let mut i: i64 = 0;
    while i < n {
        // href='#(t\d+)'>
        if match_lit(line, i, "href='#") && i + 8 < n && u_at(line, i + 7) == 0x74 {
            let mut j: i64 = i + 8;
            while j < n && is_digit(u_at(line, j)) {
                j += 1;
            }
            if j > i + 8 && j + 1 < n && u_at(line, j) == 0x27 && u_at(line, j + 1) == 0x3E {
                push_str(&mut out, "href=\"");
                let mut k: i64 = i + 6;
                while k < j {
                    out.push(u_at(line, k));
                    k += 1;
                }
                push_str(&mut out, "\">");
                i = j + 2;
                continue;
            }
        }
        // name='(m\d+)'/>
        if match_lit(line, i, "name='") && i + 7 < n && u_at(line, i + 6) == 0x6D {
            let mut j: i64 = i + 7;
            while j < n && is_digit(u_at(line, j)) {
                j += 1;
            }
            if j > i + 7 && j + 2 < n && u_at(line, j) == 0x27 && u_at(line, j + 1) == 0x2F && u_at(line, j + 2) == 0x3E {
                push_str(&mut out, "name=\"");
                let mut k: i64 = i + 6;
                while k < j {
                    out.push(u_at(line, k));
                    k += 1;
                }
                push_str(&mut out, "\"/>");
                i = j + 3;
                continue;
            }
        }
        out.push(u_at(line, i));
        i += 1;
    }
    return out;
}

fn exc_table(msg: String) -> Ex {
    return exc(ErrorKind::TABLE_INPUT, msg);
}

// escape_annotated: decode entities as Xalan's parse does (raw '<' and bare
// '&' throw), then re-print per rule 8 (only < > & become entities).
fn escape_annotated(text: &U) -> Result<U, Ex> {
    let mut res: U = vec![];
    let n: i64 = u_len(text);
    let mut i: i64 = 0;
    while i < n {
        let c: u16 = u_at(text, i);
        if c == 0x3C {
            return Err(exc_table("annotation text contains raw '<'".to_string()));
        }
        if c == 0x3E {
            push_str(&mut res, "&gt;");
            i += 1;
            continue;
        }
        if c != 0x26 {
            res.push(c);
            i += 1;
            continue;
        }
        let end: i64 = entity_match_at(text, i);
        if end == -1 {
            return Err(exc_table("annotation text contains a bare '&'".to_string()));
        }
        // ent = text[i..end]
        if match_lit(text, i, "&lt;") {
            push_str(&mut res, "&lt;");
        } else if match_lit(text, i, "&gt;") {
            push_str(&mut res, "&gt;");
        } else if match_lit(text, i, "&amp;") {
            push_str(&mut res, "&amp;");
        } else if match_lit(text, i, "&quot;") {
            res.push(0x22);
        } else if match_lit(text, i, "&apos;") {
            res.push(0x27);
        } else {
            // numeric reference
            let mut cp: i64 = 0;
            let hex: bool = i + 2 < n && (u_at(text, i + 2) == 0x78 || u_at(text, i + 2) == 0x58);
            let dstart: i64 = if hex { i + 3 } else { i + 2 };
            let mut j: i64 = dstart;
            while j < end - 1 {
                let d: u16 = u_at(text, j);
                let v: i64;
                if hex {
                    if d <= 0x39 {
                        v = (d as i64) - 0x30;
                    } else if d <= 0x5A {
                        v = (d as i64) - 0x41 + 10;
                    } else {
                        v = (d as i64) - 0x61 + 10;
                    }
                } else {
                    v = (d as i64) - 0x30;
                }
                if hex {
                    cp = cp * 16 + v;
                } else {
                    cp = cp * 10 + v;
                }
                j += 1;
            }
            if cp <= 0 || cp > 0x10FFFF {
                let mut ent: U = vec![];
                let mut k: i64 = i;
                while k < end {
                    ent.push(u_at(text, k));
                    k += 1;
                }
                let mut msg: String = "bad character reference ".to_string();
                let mref: String = u_to_string(&ent);
                msg += &mref;
                return Err(exc_table(msg));
            }
            if cp == 0x3C {
                push_str(&mut res, "&lt;");
            } else if cp == 0x3E {
                push_str(&mut res, "&gt;");
            } else if cp == 0x26 {
                push_str(&mut res, "&amp;");
            } else if cp < 0x10000 {
                res.push(cp as u16);
            } else {
                let c2: i64 = cp - 0x10000;
                res.push((0xD800 + (c2 >> 10)) as u16);
                res.push((0xDC00 + (c2 & 0x3FF)) as u16);
            }
        }
        i = end;
    }
    return Ok(res);
}

// --- entry point and styles ---

pub fn table_on(f: &mut Flow, options: &RenderOptions) -> Result<U, Ex> {
    let lines: Vec<U> = draw_text_lines(f, MODE_XML, None)?;

    let mut picture: Vec<U> = vec![];
    for line in lines {
        check_picture_line(&line)?;
        picture.push(fix_link_quotes(&line));
    }

    let mut rows: Vec<Row> = vec![];
    let nm: usize = f.messages.len();
    let mut mi: usize = 0;
    while mi < nm {
        if f.messages[mi].my_number != 0 {
            let mut r: Row = Row {
                num: f.messages[mi].my_number,
                captions: vec![],
                notes: vec![],
                comments: vec![],
            };
            let mut ci: usize = 0;
            while ci < f.messages[mi].caption.len() {
                r.captions.push(escape_annotated(&f.messages[mi].caption[ci])?);
                ci += 1;
            }
            let mut ni: usize = 0;
            while ni < f.messages[mi].notes.len() {
                r.notes.push(escape_annotated(&f.messages[mi].notes[ni])?);
                ni += 1;
            }
            let mut ki: usize = 0;
            while ki < f.messages[mi].comments.len() {
                r.comments.push(escape_annotated(&f.messages[mi].comments[ki])?);
                ki += 1;
            }
            rows.push(r);
        }
        mi += 1;
    }

    let mut picture_doc: U = ul_a("<picture>\n".to_string());
    let pl: usize = picture.len();
    let mut pi: usize = 0;
    while pi < pl {
        if pi > 0 {
            picture_doc.push(0x0A);
        }
        push_units(&mut picture_doc, &picture[pi]);
        pi += 1;
    }
    push_str(&mut picture_doc, "\n</picture>");

    if options.table_style == TableStyle::ARCHTABLE {
        return Ok(archtable(&picture_doc, &rows));
    }
    if options.table_style == TableStyle::HTMLTABLE {
        return Ok(htmltable(&picture_doc, &rows));
    }
    if options.table_style == TableStyle::HTMLTABLE2 {
        return Ok(htmltable2(&picture_doc, &rows));
    }
    if options.table_style == TableStyle::HTMLTABLE3 {
        return Ok(htmltable3(&picture_doc, &rows));
    }
    if options.table_style == TableStyle::NOLINKS {
        return Ok(nolinks(&picture_doc, &rows));
    }
    return Ok(wordtable(&picture_doc, &rows));
}

pub fn archtable(picture: &U, rows: &Vec<Row>) -> U {
    let mut out: U = ul_a("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<html><body><pre>".to_string());
    push_units(&mut out, picture);
    push_str(&mut out, "</pre>");
    if rows.len() == 0 {
        push_str(&mut out, "<table border=\"true\"/>");
    } else {
        push_str(&mut out, "<table border=\"true\">");
        let mut i: usize = 0;
        while i < rows.len() {
            let r: &Row = &rows[i];
            push_str(&mut out, "<tr>");
            push_units(&mut out, &link_cell(r.num));
            let nb: U = note_body(&r.notes);
            push_units(&mut out, &td_cell(&nb));
            push_units(&mut out, &tt_cell(&r.comments));
            push_str(&mut out, "</tr>");
            i += 1;
        }
        push_str(&mut out, "</table>");
    }
    push_str(&mut out, "</body></html>");
    return out;
}

pub fn htmltable(picture: &U, rows: &Vec<Row>) -> U {
    let mut out: U = ul_a("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<html><body><pre>".to_string());
    push_units(&mut out, picture);
    push_str(&mut out, "</pre>");
    if rows.len() == 0 {
        push_str(&mut out, "<table border=\"1\"/>");
    } else {
        push_str(&mut out, "<table border=\"1\">");
        let mut i: usize = 0;
        while i < rows.len() {
            let r: &Row = &rows[i];
            push_str(&mut out, "<tr>");
            push_units(&mut out, &link_cell(r.num));
            let cb: U = caption_body(&r.captions);
            let nb: U = note_body(&r.notes);
            let mut merged: U = cb;
            push_units(&mut merged, &nb);
            push_units(&mut out, &td_cell(&merged));
            push_units(&mut out, &tt_cell(&r.comments));
            push_str(&mut out, "</tr>");
            i += 1;
        }
        push_str(&mut out, "</table>");
    }
    push_str(&mut out, "</body></html>");
    return out;
}

pub fn htmltable2(picture: &U, rows: &Vec<Row>) -> U {
    let mut out: U = ul_a("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<html><body><small><pre>".to_string());
    push_units(&mut out, picture);
    push_str(&mut out, "</pre></small><table border=\"1\"><tr><th>Seq</th><th>Message and Details</th><th>Notes</th></tr>");
    let mut i: usize = 0;
    while i < rows.len() {
        let r: &Row = &rows[i];
        push_str(&mut out, "<tr>");
        push_units(&mut out, &link_cell(r.num));
        let sc: U = strong_cell(&r.captions);
        let cc: U = code_cell(&r.notes);
        let mut merged: U = sc;
        push_units(&mut merged, &cc);
        push_units(&mut out, &td_cell(&merged));
        let cb: U = comment_body(&r.comments);
        push_units(&mut out, &td_cell(&cb));
        push_str(&mut out, "</tr>");
        i += 1;
    }
    push_str(&mut out, "</table></body></html>");
    return out;
}

pub fn htmltable3(picture: &U, rows: &Vec<Row>) -> U {
    let mut out: U = ul_a("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<html><body><small><small><pre>".to_string());
    push_units(&mut out, picture);
    push_str(&mut out, "</pre></small></small><table border=\"1\"><tr><th>Seq</th><th>Message and Details</th><th>Notes</th></tr>");
    let mut i: usize = 0;
    while i < rows.len() {
        let r: &Row = &rows[i];
        push_str(&mut out, "<tr>");
        push_units(&mut out, &link_cell(r.num));
        let sc: U = strong_cell(&r.captions);
        let cc: U = code_cell(&r.notes);
        let mut merged: U = sc;
        push_units(&mut merged, &cc);
        push_units(&mut out, &td_cell(&merged));
        let cb: U = comment_body(&r.comments);
        push_units(&mut out, &td_cell(&cb));
        push_str(&mut out, "</tr>");
        i += 1;
    }
    push_str(&mut out, "</table></body></html>");
    return out;
}

pub fn nolinks(picture: &U, rows: &Vec<Row>) -> U {
    let mut out: U = ul_a("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<html><body><pre>".to_string());
    push_units(&mut out, picture);
    push_str(&mut out, "</pre>");
    if rows.len() == 0 {
        push_str(&mut out, "<table border=\"true\"/>");
    } else {
        push_str(&mut out, "<table border=\"true\">");
        let mut i: usize = 0;
        while i < rows.len() {
            let r: &Row = &rows[i];
            push_str(&mut out, "<tr>");
            push_units(&mut out, &plain_cell(r.num));
            let cb: U = caption_body(&r.captions);
            let nb: U = note_body(&r.notes);
            let mut merged: U = cb;
            push_units(&mut merged, &nb);
            push_units(&mut out, &td_cell(&merged));
            push_units(&mut out, &tt_cell(&r.comments));
            push_str(&mut out, "</tr>");
            i += 1;
        }
        push_str(&mut out, "</table>");
    }
    push_str(&mut out, "</body></html>");
    return out;
}

pub fn wordtable(picture: &U, rows: &Vec<Row>) -> U {
    let mut out: U = ul_a(
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<html xmlns=\"http://www.w3.org/TR/REC-html40\" xmlns:w=\"urn:schemas-microsoft-com:office:word\" xmlns:o=\"urn:schemas-microsoft-com:office:office\"><head><meta content=\"text/html; charset=windows-1252\" http-equiv=\"Content-Type\"/><meta content=\"Word.Document\" name=\"ProgId\"/><meta content=\"Microsoft Word 9\" name=\"Generator\"/><meta content=\"Microsoft Word 9\" name=\"Originator\"/><style/></head><body style=\"tab-interval:.5in\" lang=\"EN-US\"><div class=\"Section1\"><pre>".to_string()
    );
    push_units(&mut out, picture);
    push_str(&mut out, "</pre>");
    push_str(&mut out, "<table style=\"border-collapse:collapse;border:none;mso-border-alt:solid windowtext .5pt;mso-padding-alt:0in 5.4pt 0in 5.4pt\" cellpadding=\"0\" cellspacing=\"0\" border=\"1\">");
    push_str(&mut out, "<tr><td style=\"width:48.3pt;border:solid windowtext .5pt;  padding:0in 5.4pt 0in 5.4pt\" valign=\"top\" width=\"64\"><p class=\"MsoNormal\"><b>Number<o:p/></b></p></td><td style=\"width:171.0pt;border:solid windowtext .5pt;border-left:none;mso-border-left-alt:solid windowtext .5pt;padding:0in 5.4pt 0in 5.4pt\" valign=\"top\" width=\"228\"><p class=\"MsoNormal\"><b>Annotation<o:p/></b></p></td><td style=\"width:3.2in;border:solid windowtext .5pt;border-left:none;mso-border-left-alt:solid windowtext .5pt;padding:0in 5.4pt 0in 5.4pt\" valign=\"top\" width=\"307\"><p class=\"MsoNormal\"><b>Packet<o:p/></b></p></td></tr>");
    let mut i: usize = 0;
    while i < rows.len() {
        let r: &Row = &rows[i];
        push_str(&mut out, "<tr>");
        push_str(&mut out, "<td style=\"width:48.3pt;border:solid windowtext .5pt;border-top:none;mso-border-top-alt:solid windowtext .5pt;padding:0in 5.4pt 0in 5.4pt\" valign=\"top\" width=\"64\">");
        push_str(&mut out, "<p class=\"MsoNormal\"><message><a name=\"t");
        push_units(&mut out, &fmt_i_shim(r.num));
        push_str(&mut out, "\"/><a href=\"#m");
        push_units(&mut out, &fmt_i_shim(r.num));
        push_str(&mut out, "\">");
        push_units(&mut out, &fmt_i_shim(r.num));
        push_str(&mut out, "</a></message></p></td>");
        push_str(&mut out, "<td style=\"width:171.0pt;border-top:none;border-left:none;border-bottom:solid windowtext .5pt;border-right:solid windowtext .5pt;mso-border-top-alt:solid windowtext .5pt;mso-border-left-alt:solid windowtext .5pt;padding:0in 5.4pt 0in 5.4pt\" valign=\"top\" width=\"228\">");
        push_str(&mut out, "<p class=\"MsoNormal\">");
        push_units(&mut out, &caption_body(&r.captions));
        push_units(&mut out, &note_body(&r.notes));
        push_str(&mut out, "</p></td>");
        push_str(&mut out, "<td style=\"width:3.2in;border-top:none;border-left:none;border-bottom:solid windowtext .5pt;border-right:solid windowtext .5pt;mso-border-top-alt:solid windowtext .5pt;mso-border-left-alt:solid windowtext .5pt;padding:0in 5.4pt 0in 5.4pt\" valign=\"top\" width=\"307\">");
        push_str(&mut out, "<p class=\"MsoNormal\"><span style=\"font-family:&quot;Courier New&quot;\">");
        push_units(&mut out, &comment_body(&r.comments));
        push_str(&mut out, "<o:p/></span></p></td>");
        push_str(&mut out, "</tr>");
        i += 1;
    }
    push_str(&mut out, "</table></div></body></html>");
    return out;
}

// fmt_i on i64 (text.rs keeps fmt_i local-imported; alias for clarity).
pub fn fmt_i_shim(n: i64) -> U {
    return fmt_i_of(n);
}

fn fmt_i_of(n: i64) -> U {
    return fmt_i(n);
}
