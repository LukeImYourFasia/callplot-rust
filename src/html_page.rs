// The whole -html page (DESIGN.md 3.12.2 + 3.12.6; renderers/html/page.py,
// a merge of JS html/page.js + html/table.js).  One self-contained file, no
// JS, nothing from the network.  The layout switch is a hidden radio pair.

use crate::ir::Model;
use crate::ir::Event;
use crate::ir::Participant;
use crate::ir::Kind;
use crate::options::RenderOptions;
use crate::options::HtmlLayout;
use crate::options::parse_html_layout;
use crate::jcompat::U;
use crate::jcompat::to_units;
use crate::jcompat::from_units;
use crate::jcompat::ul_a;
use crate::jcompat::fmt_i;
use crate::escape::esc_html;
use crate::html_style::CSS;
use crate::html_svg::svg_on;
use crate::errors::Ex;

pub fn push_lit(out: &mut U, s: &str) {
    push_units(out, &to_units(s.to_string()));
}

pub fn push_units(out: &mut U, t: &U) {
    for c in t {
        out.push(*c);
    }
}

pub fn line_lit(out: &mut U, s: &str) {
    push_lit(out, s);
    out.push(0x0A);
}

// one U wrapped in two literals
pub fn wrap_us(inner: &U, open: String, close: String) -> U {
    let mut out: U = ul_a(open);
    push_units(&mut out, inner);
    push_units(&mut out, &ul_a(close));
    return out;
}

// join U parts with a separator (U sep)
pub fn join_us(parts: &Vec<U>, sep: &U) -> U {
    let mut out: U = vec![];
    let n: usize = parts.len();
    let mut i: usize = 0;
    while i < n {
        if i > 0 {
            push_units(&mut out, sep);
        }
        push_units(&mut out, &parts[i]);
        i += 1;
    }
    return out;
}

pub fn pick_layout(model: &Model, options: &RenderOptions) -> Result<HtmlLayout, Ex> {
    let forced: Option<HtmlLayout> = options.html_layout.clone();
    if forced != None {
        return Ok(forced.unwrap());
    }
    let raw: Option<U> = clone_opt_u(&model.options.html_layout);
    if raw == None {
        return Ok(HtmlLayout::TOPBOTTOM);
    }
    let s: String = from_units(&raw.unwrap());
    return parse_html_layout(&s);
}

pub fn clone_opt_u(o: &Option<U>) -> Option<U> {
    match o {
        None => { return None; }
        Some(v) => { return Some(v.clone()); }
    }
}

// label lines joined with a space (column 3)
pub fn label_one(p: &Participant) -> U {
    return esc_html(&join_us(&p.label, &ul_a(" ".to_string())));
}

pub fn kind_word(kind: Kind) -> String {
    if kind == Kind::MEDIA_REPEAT {
        return "repeat".to_string();
    }
    return kind_class(kind);
}

pub fn kind_class(kind: Kind) -> String {
    if kind == Kind::NOTE {
        return "note".to_string();
    }
    if kind == Kind::SELF {
        return "self".to_string();
    }
    if kind == Kind::MEDIA_REPEAT {
        return "mediaRepeat".to_string();
    }
    if kind == Kind::MEDIA {
        return "media".to_string();
    }
    if kind == Kind::REPLY {
        return "reply".to_string();
    }
    return "message".to_string();
}

// the back link: a row-header cell pinned at the left edge
pub fn write_back_link(out: &mut U, e: &Event, prefix: &str) {
    push_lit(out, "<a class=\"back\" href=\"#");
    push_lit(out, prefix);
    push_lit(out, "flow-");
    push_lit(out, &e.id);
    push_lit(out, "\" aria-label=\"");
    let en: Option<i64> = e.num;
    if en.is_some() {
        push_lit(out, "Back to message ");
        push_units(out, &fmt_i(en.unwrap()));
        push_lit(out, " in the diagram");
        push_lit(out, "\">");
        out.push(0x2191);
        out.push(0x20);
        push_units(out, &fmt_i(en.unwrap()));
    } else {
        push_lit(out, "Back to this event in the diagram\">");
        out.push(0x2191);
        out.push(0x20);
        out.push(0x00B7);
    }
    push_lit(out, "</a>");
}

pub fn detail_table(out: &mut U, model: &Model, prefix: &str) {
    line_lit(out, "<table class=\"cpf-table\">");
    push_lit(out, "<thead><tr><th>#</th><th>Kind</th><th>From ");
    out.push(0x2192);
    push_lit(out, " To</th><th>Message and details</th><th>Notes</th><th>Line</th></tr></thead>");
    out.push(0x0A);
    line_lit(out, "<tbody>");
    let ne: usize = model.events.len();
    let mut k: usize = 0;
    while k < ne {
        let e: &Event = &model.events[k];
        push_lit(out, "<tr id=\"");
        push_lit(out, prefix);
        push_lit(out, "detail-");
        push_lit(out, &e.id);
        push_lit(out, "\">");
        out.push(0x0A);

        push_lit(out, "<th class=\"k\" scope=\"row\">");
        write_back_link(out, e, prefix);        push_lit(out, "</th>");
        out.push(0x0A);

        push_lit(out, "<td><span class=\"pill\">");
        push_lit(out, &kind_word(e.kind.clone()));
        push_lit(out, "</span></td>");
        out.push(0x0A);

        // column 3: From -> To; media an ellipsis, a note "note on A"
        let a: U = label_one(&model.participants[e.from_ as usize]);
        let b: U = label_one(&model.participants[e.to as usize]);
        push_lit(out, "<td>");
        if e.kind == Kind::NOTE {
            push_lit(out, "note on ");
            push_units(out, &a);
        } else if e.kind == Kind::MEDIA || e.kind == Kind::MEDIA_REPEAT {
            push_units(out, &a);
            out.push(0x20);
            out.push(0x22EF);
            out.push(0x20);
            push_units(out, &b);
        } else {
            push_units(out, &a);
            out.push(0x20);
            out.push(0x2192);
            out.push(0x20);
            push_units(out, &b);
        }
        push_lit(out, "</td>");
        out.push(0x0A);

        // column 4: caption lines bold, then !! notes in mono, one per line
        let mut parts: Vec<U> = vec![];
        let nc: usize = e.caption.len();
        let mut ci: usize = 0;
        while ci < nc {
            let one: U = wrap_us(&esc_html(&e.caption[ci]), "<strong>".to_string(), "</strong>".to_string());
            parts.push(one);
            ci += 1;
        }
        let nn: usize = e.notes.len();
        let mut ni: usize = 0;
        while ni < nn {
            let one2: U = wrap_us(&esc_html(&e.notes[ni]), "<code>".to_string(), "</code>".to_string());
            parts.push(one2);
            ni += 1;
        }
        push_lit(out, "<td>");
        push_units(out, &join_us(&parts, &ul_a("<br>".to_string())));
        push_lit(out, "</td>");
        out.push(0x0A);

        // column 5: ! comments as a list, an em-dash when there are none
        push_lit(out, "<td>");
        if e.comments.len() == 0 {
            out.push(0x2014);
        } else {
            push_lit(out, "<ul>");
            let ncm: usize = e.comments.len();
            let mut cmi: usize = 0;
            while cmi < ncm {
                push_lit(out, "<li>");
                push_units(out, &esc_html(&e.comments[cmi]));
                push_lit(out, "</li>");
                cmi += 1;
            }
            push_lit(out, "</ul>");
        }
        push_lit(out, "</td>");
        out.push(0x0A);

        // column 6: source line, right-aligned by CSS with tabular digits
        push_lit(out, "<td>");
        push_units(out, &fmt_i(e.src_line));
        push_lit(out, "</td>");
        out.push(0x0A);

        line_lit(out, "</tr>");
        k += 1;
    }
    line_lit(out, "</tbody>");
    line_lit(out, "</table>");
}

pub fn page_on(model: &Model, options: &RenderOptions) -> Result<U, Ex> {
    let out: U = vec![];
    let mut out: U = out;
    page_write(&model, options, &mut out)?;
    return Ok(out);
}

fn page_write(model: &Model, options: &RenderOptions, out: &mut U) -> Result<(), Ex> {
    let layout: HtmlLayout = pick_layout(model, options)?;
    let prefix: &str = &options.id_prefix;

    let esc_title: U = {
        let t: Option<U> = clone_opt_u(&model.title);
        if t == None {
            esc_html(&ul_a("Call flow".to_string()))
        } else {
            esc_html(&t.unwrap())
        }
    };

    line_lit(out, "<!doctype html>");
    line_lit(out, "<html lang=\"en\">");
    line_lit(out, "<head>");
    line_lit(out, "<meta charset=\"utf-8\">");
    line_lit(out, "<meta name=\"viewport\" content=\"width=device-width, initial-scale=1\">");
    push_lit(out, "<title>");
    push_units(out, &esc_title);
    push_lit(out, "</title>");
    out.push(0x0A);
    line_lit(out, "<style>");
    push_units(out, &to_units(CSS.to_string()));
    out.push(0x0A);
    line_lit(out, "</style>");
    line_lit(out, "</head>");
    line_lit(out, "<body>");
    line_lit(out, "<div class=\"cpf\">");

    // the layout switch: both radios always present, the chosen one checked
    let tb: String = if layout == HtmlLayout::TOPBOTTOM { " checked".to_string() } else { "".to_string() };
    let sbs: String = if layout == HtmlLayout::SIDEBYSIDE { " checked".to_string() } else { "".to_string() };
    push_lit(out, "<input type=\"radio\" name=\"");
    push_lit(out, prefix);
    push_lit(out, "layout\" id=\"");
    push_lit(out, prefix);
    push_lit(out, "lay-tb\" class=\"cpf-radio\"");
    push_lit(out, &tb);
    push_lit(out, ">");
    out.push(0x0A);
    push_lit(out, "<input type=\"radio\" name=\"");
    push_lit(out, prefix);
    push_lit(out, "layout\" id=\"");
    push_lit(out, prefix);
    push_lit(out, "lay-sbs\" class=\"cpf-radio\"");
    push_lit(out, &sbs);
    push_lit(out, ">");
    out.push(0x0A);

    line_lit(out, "<header class=\"cpf-head\">");
    push_lit(out, "<h1>");
    push_units(out, &esc_title);
    push_lit(out, "</h1>");
    out.push(0x0A);
    push_lit(out, "<p>");
    push_units(out, &fmt_i(model.participants.len() as i64));
    push_lit(out, " participants");
    out.push(0x20);
    out.push(0x00B7);
    out.push(0x20);
    push_units(out, &fmt_i(model.events.len() as i64));
    push_lit(out, " events</p>");
    out.push(0x0A);
    line_lit(out, "<div class=\"cpf-switch\" role=\"group\" aria-label=\"Layout\">");
    push_lit(out, "<label for=\"");
    push_lit(out, prefix);
    push_lit(out, "lay-tb\">Top and bottom</label>");
    out.push(0x0A);
    push_lit(out, "<label for=\"");
    push_lit(out, prefix);
    push_lit(out, "lay-sbs\">Side by side</label>");
    out.push(0x0A);
    line_lit(out, "</div>");
    line_lit(out, "</header>");

    line_lit(out, "<div class=\"cpf-main\">");
    line_lit(out, "<section class=\"cpf-flow\" aria-label=\"Diagram\"><div class=\"cpf-scroll\">");
    let svg: U = svg_on(model, prefix)?;
    push_units(out, &svg);
    line_lit(out, "</div></section>");
    line_lit(out, "<section class=\"cpf-detail\" aria-label=\"Details\"><div class=\"cpf-tscroll\">");
    detail_table(out, model, prefix);
    line_lit(out, "</div></section>");
    line_lit(out, "</div>");

    line_lit(out, "</div>");
    line_lit(out, "</body>");
    line_lit(out, "</html>");
    return Ok(());
}
