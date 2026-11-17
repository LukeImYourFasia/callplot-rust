// The inline SVG of the -html page (DESIGN.md 3.12.3 + 3.12.4;
// renderers/html/svg.py + render/html/svg.js).  Numbers print as JS prints
// them (jsnum): the layout is JS arithmetic and the snapshot is JS.
// Output is built as U (UTF-16 units) like every other renderer.

use crate::ir::Model;
use crate::ir::Kind;
use crate::ir::Event;
use crate::ir::Participant;
use crate::html_layout::compute_layout;
use crate::html_layout::Band;
use crate::html_layout::HtmlLayoutData;
use crate::html_layout::CW;
use crate::html_layout::LABW;
use crate::html_layout::LH;
use crate::html_layout::M;
use crate::html_layout::TOP;
use crate::html_layout::NOTE_DX;
use crate::html_layout::NOTE_PAD;
use crate::html_layout::NOTE_DY;
use crate::html_layout::SELF_W;
use crate::html_layout::SELF_H;
use crate::html_layout::SELF_DX;
use crate::html_layout::HL_DY;
use crate::html_layout::HL_BOT;
use crate::jsnum::js_num;
use crate::errors::Ex;
use crate::jcompat::U;
use crate::jcompat::to_units;
use crate::jcompat::ul_a;
use crate::jcompat::u_len;
use crate::jcompat::fmt_i;
use crate::escape::esc_html;

// parts joined into one String; every literal arrives .to_string()-ed at the
// call site so no + operator mixing is needed.
pub fn cat(parts: Vec<String>) -> String {
    let mut out: String = "".to_string();
    for p in parts {
        out += &p;
    }
    return out;
}

pub fn kind_class(kind: Kind) -> String {
    if kind == Kind::MEDIA_REPEAT {
        return "media".to_string();
    }
    if kind == Kind::NOTE {
        return "note".to_string();
    }
    if kind == Kind::SELF {
        return "self".to_string();
    }
    if kind == Kind::MEDIA {
        return "media".to_string();
    }
    if kind == Kind::REPLY {
        return "reply".to_string();
    }
    return "message".to_string();
}

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

pub fn line_cat(out: &mut U, parts: Vec<String>) {
    let s: String = cat(parts);
    push_units(out, &to_units(s));
    out.push(0x0A);
}

// push a JS-number rendering of v (String -> units)
pub fn push_num(out: &mut U, v: f64) -> Result<(), Ex> {
    let s: String = js_num(v)?;
    push_units(out, &to_units(s));
    return Ok(());
}

pub fn svg_on(model: &Model, prefix: &str) -> Result<U, Ex> {
    let out: U = vec![];
    let mut out: U = out;
    svg_write(&model, prefix, &mut out)?;
    return Ok(out);
}

fn svg_write(model: &Model, prefix: &str, out: &mut U) -> Result<(), Ex> {
    let lay: HtmlLayoutData = compute_layout(model);
    let bottom: f64 = lay.h - HL_BOT;

    line_cat(out, vec!["<svg class=\"cpf-svg\" xmlns=\"http://www.w3.org/2000/svg\" width=\"".to_string(),
                       js_num(lay.w)?, "\" height=\"".to_string(), js_num(lay.h)?,
                       "\" viewBox=\"0 0 ".to_string(), js_num(lay.w)?, " ".to_string(), js_num(lay.h)?,
                       "\" role=\"img\" aria-label=\"Call flow diagram\">".to_string()]);

    // lifelines: from just under the boxes down to the last band's bottom
    let nx: usize = lay.x.len();
    let mut xi_i: usize = 0;
    while xi_i < nx {
        let xi: f64 = lay.x[xi_i];
        line_cat(out, vec!["<line class=\"lf\" x1=\"".to_string(), js_num(xi)?, "\" x2=\"".to_string(),
                           js_num(xi)?, "\" y1=\"".to_string(), js_num(TOP + lay.box_h)?,
                           "\" y2=\"".to_string(), js_num(bottom)?,
                           "\" stroke=\"var(--cpf-line, #b7c2cc)\" stroke-dasharray=\"1 3\"/>".to_string()]);
        xi_i += 1;
    }

    // participant boxes with their labels
    let np: usize = model.participants.len();
    let mut i: usize = 0;
    while i < np {
        let p: &Participant = &model.participants[i];
        let xi: f64 = lay.x[i];
        let max_len: i64 = longest_label(p);
        let bw: f64 = (max_len as f64) * LABW + 16.0;
        let bh: f64 = (p.label.len() as f64) * LH + 14.0;
        line_lit(out, "<g class=\"pf\">");
        line_cat(out, vec!["<rect class=\"pf-box\" x=\"".to_string(), js_num(xi - bw / 2.0)?,
                           "\" y=\"".to_string(), js_num(TOP)?, "\" width=\"".to_string(), js_num(bw)?,
                           "\" height=\"".to_string(), js_num(bh)?,
                           "\" rx=\"4\" fill=\"var(--cpf-head, #e8eef4)\" stroke=\"var(--cpf-line, #b7c2cc)\"/>".to_string()]);
        let nl: usize = p.label.len();
        let mut j: usize = 0;
        while j < nl {
            // <text ... >escaped label</text>
            push_lit(out, "<text class=\"lb\" x=\"");
            push_num(out, xi)?;
            push_lit(out, "\" y=\"");
            push_num(out, TOP + 11.0 + (j as f64) * LH)?;
            push_lit(out, "\" fill=\"var(--cpf-ink, #17232e)\" font-family=\"ui-sans-serif, system-ui, sans-serif\" font-size=\"13\" text-anchor=\"middle\">");
            push_units(out, &esc_html(&p.label[j]));
            push_lit(out, "</text>");
            out.push(0x0A);
            j += 1;
        }
        line_lit(out, "</g>");
        i += 1;
    }

    // events, one group each
    let ne: usize = model.events.len();
    let mut k: usize = 0;
    while k < ne {
        let e: &Event = &model.events[k];
        let band: &Band = &lay.bands[k];
        let y: f64 = band.top;
        let h: f64 = band.bh;

        push_lit(out, "<g class=\"ev ev-");
        push_lit(out, &kind_class(e.kind.clone()));
        push_lit(out, "\" id=\"");
        push_lit(out, prefix);
        push_lit(out, "flow-");
        push_lit(out, &e.id);
        push_lit(out, "\">");
        out.push(0x0A);

        push_lit(out, "<rect class=\"hl\" x=\"0\" y=\"");
        push_num(out, y - HL_DY)?;
        push_lit(out, "\" width=\"100%\" height=\"");
        push_num(out, h)?;
        push_lit(out, "\" fill=\"none\"/>");
        out.push(0x0A);

        let lines_n: i64 = e.caption.len() as i64;
        let ay: f64 = y + (lines_n as f64) * LH + 15.0;

        if e.kind != Kind::NOTE && e.kind != Kind::SELF {
            let x1: f64 = lay.x[e.from_ as usize];
            let x2: f64 = lay.x[e.to as usize];
            let direction: f64 = if x2 >= x1 { 1.0 } else { -1.0 };
            let mut dash: String = "".to_string();
            let mut color: String = "var(--cpf-ink, #17232e)".to_string();
            if e.kind == Kind::REPLY {
                dash = " stroke-dasharray=\"6 4\"".to_string();
            } else if e.kind == Kind::MEDIA {
                color = "#008080".to_string();
                dash = " stroke-dasharray=\"5 4\"".to_string();
            } else if e.kind == Kind::MEDIA_REPEAT {
                color = "#008080".to_string();
                dash = " stroke-dasharray=\"2 3\"".to_string();
            }
            push_lit(out, "<line class=\"ln\" x1=\"");
            push_num(out, x1)?;
            push_lit(out, "\" y1=\"");
            push_num(out, ay)?;
            push_lit(out, "\" x2=\"");
            push_num(out, x2 - direction * 9.0)?;
            push_lit(out, "\" y2=\"");
            push_num(out, ay)?;
            push_lit(out, "\" stroke=\"");
            push_lit(out, &color);
            push_lit(out, "\"");
            push_lit(out, &dash);
            push_lit(out, "/>");
            out.push(0x0A);
            if e.kind == Kind::MESSAGE || e.kind == Kind::REPLY {
                push_lit(out, "<polygon class=\"ah\" points=\"");
                push_num(out, x2)?;
                push_lit(out, ",");
                push_num(out, ay)?;
                push_lit(out, " ");
                push_num(out, x2 - direction * 9.0)?;
                push_lit(out, ",");
                push_num(out, ay - 4.0)?;
                push_lit(out, " ");
                push_num(out, x2 - direction * 9.0)?;
                push_lit(out, ",");
                push_num(out, ay + 4.0)?;
                push_lit(out, "\" fill=\"");
                push_lit(out, &color);
                push_lit(out, "\"/>");
                out.push(0x0A);
            }
        } else if e.kind == Kind::SELF {
            let x1: f64 = lay.x[e.from_ as usize];
            push_lit(out, "<polyline class=\"ln\" points=\"");
            push_num(out, x1)?;
            push_lit(out, ",");
            push_num(out, ay - SELF_H / 2.0)?;
            push_lit(out, " ");
            push_num(out, x1 + SELF_W)?;
            push_lit(out, ",");
            push_num(out, ay - SELF_H / 2.0)?;
            push_lit(out, " ");
            push_num(out, x1 + SELF_W)?;
            push_lit(out, ",");
            push_num(out, ay + SELF_H / 2.0)?;
            push_lit(out, " ");
            push_num(out, x1)?;
            push_lit(out, ",");
            push_num(out, ay + SELF_H / 2.0)?;
            push_lit(out, "\" stroke=\"var(--cpf-ink, #17232e)\" fill=\"none\"/>");
            out.push(0x0A);
            push_lit(out, "<polygon class=\"ah\" points=\"");
            push_num(out, x1)?;
            push_lit(out, ",");
            push_num(out, ay + SELF_H / 2.0)?;
            push_lit(out, " ");
            push_num(out, x1 + 9.0)?;
            push_lit(out, ",");
            push_num(out, ay + SELF_H / 2.0 - 4.0)?;
            push_lit(out, " ");
            push_num(out, x1 + 9.0)?;
            push_lit(out, ",");
            push_num(out, ay + SELF_H / 2.0 + 4.0)?;
            push_lit(out, "\" fill=\"var(--cpf-ink, #17232e)\"/>");
            out.push(0x0A);
        } else {
            let x1: f64 = lay.x[e.from_ as usize];
            let box_w: f64 = (lay.cap_lens[k] as f64) * CW + NOTE_PAD;
            push_lit(out, "<rect class=\"nt\" x=\"");
            push_num(out, x1 + NOTE_DX)?;
            push_lit(out, "\" y=\"");
            push_num(out, y + NOTE_DY)?;
            push_lit(out, "\" width=\"");
            push_num(out, box_w)?;
            push_lit(out, "\" height=\"");
            push_num(out, (lines_n as f64) * LH + 12.0)?;
            push_lit(out, "\" rx=\"4\" fill=\"var(--cpf-note-fill, #fff8dd)\" stroke=\"var(--cpf-note-line, #c8b45f)\"/>");
            out.push(0x0A);
        }

        // caption lines
        let mut tx: f64 = 0.0;
        let anchor: String;
        let mono: String;
        if e.kind == Kind::SELF {
            tx = lay.x[e.from_ as usize] + SELF_DX;
            anchor = "start".to_string();
            mono = "".to_string();
        } else if e.kind == Kind::NOTE {
            tx = lay.x[e.from_ as usize] + NOTE_DX + ((lay.cap_lens[k] as f64) * CW + NOTE_PAD) / 2.0;
            anchor = "middle".to_string();
            mono = " font-family=\"ui-monospace, Menlo, Consolas, monospace\"".to_string();
        } else {
            tx = (lay.x[e.from_ as usize] + lay.x[e.to as usize]) / 2.0;
            anchor = "middle".to_string();
            mono = " font-family=\"ui-monospace, Menlo, Consolas, monospace\"".to_string();
        }
        let cl: usize = e.caption.len();
        let mut c: usize = 0;
        while c < cl {
            push_lit(out, "<text class=\"cap\" x=\"");
            push_num(out, tx)?;
            push_lit(out, "\" y=\"");
            push_num(out, y + 12.0 + (c as f64) * LH)?;
            push_lit(out, "\" text-anchor=\"");
            push_lit(out, &anchor);
            push_lit(out, "\" fill=\"var(--cpf-ink, #17232e)\"");
            push_lit(out, &mono);
            push_lit(out, " font-size=\"12.5\">");
            push_units(out, &esc_html(&e.caption[c]));
            push_lit(out, "</text>");
            out.push(0x0A);
            c += 1;
        }

        // badge: numbered events show (N) over a circle r=10, others a dot r=5
        let bx: f64;
        let by: f64;
        if e.kind == Kind::NOTE || e.kind == Kind::SELF {
            bx = lay.x[e.from_ as usize];
            by = y + h / 2.0;
        } else {
            bx = (lay.x[e.from_ as usize] + lay.x[e.to as usize]) / 2.0;
            by = ay;
        }
        push_lit(out, "<a class=\"badge\" href=\"#");
        push_lit(out, prefix);
        push_lit(out, "detail-");
        push_lit(out, &e.id);
        push_lit(out, "\">");
        out.push(0x0A);
        let en: Option<i64> = e.num;
        if en.is_some() {
            push_lit(out, "<circle class=\"bdg\" cx=\"");
            push_num(out, bx)?;
            push_lit(out, "\" cy=\"");
            push_num(out, by)?;
            push_lit(out, "\" r=\"10\" fill=\"var(--cpf-accent, #2f6b8f)\"/>");
            out.push(0x0A);
            push_lit(out, "<text class=\"bn\" x=\"");
            push_num(out, bx)?;
            push_lit(out, "\" y=\"");
            push_num(out, by + 4.0)?;
            push_lit(out, "\" font-size=\"12\" text-anchor=\"middle\" fill=\"var(--cpf-page, #ffffff)\">");
            push_units(out, &fmt_i(en.unwrap()));
            push_lit(out, "</text>");
            out.push(0x0A);
        } else {
            push_lit(out, "<circle class=\"bdg\" cx=\"");
            push_num(out, bx)?;
            push_lit(out, "\" cy=\"");
            push_num(out, by)?;
            push_lit(out, "\" r=\"5\" fill=\"var(--cpf-accent, #2f6b8f)\"/>");
            out.push(0x0A);
        }
        line_lit(out, "</a>");
        line_lit(out, "</g>");
        k += 1;
    }
    line_lit(out, "</svg>");
    return Ok(());
}

pub fn longest_label(p: &Participant) -> i64 {
    let mut length: i64 = 0;
    let n: usize = p.label.len();
    let mut i: usize = 0;
    while i < n {
        let l: i64 = u_len(&p.label[i]);
        if l > length {
            length = l;
        }
        i += 1;
    }
    return length;
}
