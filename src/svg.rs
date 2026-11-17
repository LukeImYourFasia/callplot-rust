// SVG renderer (DESIGN.md section 3.4.3; svg.py).
// Flow.java:462-513 + 562-578 (drawPictureOn SVG branches),
// Player.java:127-148 (drawPictureOn SVG branch),
// Message.java:226-231 (textout) and 253-279 (drawPictureOn SVG branch).
// Fixed text copied character for character from the Java literals.

use crate::constants::MODE_XML;
use crate::context::RunContext;
use crate::errors::Ex;
use crate::flow::Flow;
use crate::flow::xml_escape;
use crate::flow::column_of;
use crate::flow::start_message_numbers;
use crate::flow::get_total_height;
use crate::flow::get_total_width;
use crate::flow::get_column_pitch;
use crate::flow::get_stagger;
use crate::flow::get_vml_x_size;
use crate::flow::get_vml_y_size;
use crate::jcompat::U;
use crate::jcompat::fmt_d;
use crate::jcompat::fmt_i;
use crate::flow::get_vml_box_width_percentage;
use crate::jcompat::idiv;
use crate::jcompat::parse_java_int;
use crate::jcompat::ul_a;
use crate::model::Message;
use crate::model::Player;
use crate::model::is_self;
use crate::model::is_undirected;

pub fn push_units(out: &mut U, t: &U) {
    for c in t {
        out.push(*c);
    }
}

fn push_line(out: &mut U, t: &U) {
    push_units(out, t);
    out.push(0x0A);
}

// Append a String (e.g. from fmt_d) as units.
pub fn push_str(out: &mut U, s: String) {
    push_units(out, &ul_a(s));
}

pub fn svg_on(f: &mut Flow, mode: usize, ctx: &mut RunContext) -> Result<U, Ex> {
    let mut out: U = vec![];
    start_message_numbers(f);

    if (mode & MODE_XML) != 0 {
        push_line(&mut out, &ul_a("<plot>".to_string()));
        push_line(&mut out, &ul_a("<picture>".to_string()));
    }

    // prologue, Flow.java:462-513
    push_line(&mut out, &ul_a("<?xml version='1.0'?>".to_string()));
    push_line(&mut out, &ul_a("<!DOCTYPE svg PUBLIC '-//W3C//DTD SVG 1.0//EN'".to_string()));
    push_line(&mut out, &ul_a("'http://www.w3.org/TR/2001/REC-SVG-20010904/DTD/svg10.dtd'>".to_string()));
    f.vml_y_size = get_total_height(f) * 2;
    f.vml_x_size = get_total_width(f);
    let mut ln: U = ul_a("<!-- totalHeight =".to_string());
    push_units(&mut ln, &fmt_i(get_total_height(f)));
    push_units(&mut ln, &ul_a("-->".to_string()));
    push_line(&mut out, &ln);
    let mut ln2: U = ul_a("<!-- totalWidth =".to_string());
    push_units(&mut ln2, &fmt_i(get_total_width(f)));
    push_units(&mut ln2, &ul_a("-->".to_string()));
    push_line(&mut out, &ln2);
    let mut sv: U = ul_a("<svg width='".to_string());
    push_units(&mut sv, &fmt_i(idiv(get_total_width(f), 10)?));
    push_units(&mut sv, &ul_a("in' height='".to_string()));
    push_units(&mut sv, &fmt_i(idiv(get_total_height(f), 6)?));
    push_units(&mut sv, &ul_a("in' viewBox='0 0 ".to_string()));
    push_units(&mut sv, &fmt_i(get_total_width(f)));
    push_units(&mut sv, &ul_a(" ".to_string()));
    push_units(&mut sv, &fmt_i(get_total_height(f)));
    push_units(&mut sv, &ul_a("'>".to_string()));
    push_line(&mut out, &sv);
    push_line(&mut out, &ul_a("".to_string()));
    push_line(&mut out, &ul_a("  <defs>".to_string()));
    push_line(&mut out, &ul_a("    <style type='text/css'><![CDATA[".to_string()));
    push_line(&mut out, &ul_a("      rect {".to_string()));
    push_line(&mut out, &ul_a("        fill: white;".to_string()));
    push_line(&mut out, &ul_a("        stroke: grey;".to_string()));
    push_line(&mut out, &ul_a("        stroke-width: 0.1".to_string()));
    push_line(&mut out, &ul_a("      }".to_string()));
    push_line(&mut out, &ul_a("      text {".to_string()));
    push_line(&mut out, &ul_a("        font-size: 1;".to_string()));
    push_line(&mut out, &ul_a("      }".to_string()));
    push_line(&mut out, &ul_a("      line {".to_string()));
    push_line(&mut out, &ul_a("        stroke: black;".to_string()));
    push_line(&mut out, &ul_a("        stroke-width: 0.1".to_string()));
    push_line(&mut out, &ul_a("      }".to_string()));
    push_line(&mut out, &ul_a("      line.dash {".to_string()));
    push_line(&mut out, &ul_a("        stroke-dasharray: 1,1;".to_string()));
    push_line(&mut out, &ul_a("        stroke: black;".to_string()));
    push_line(&mut out, &ul_a("        stroke-width: 0.1".to_string()));
    push_line(&mut out, &ul_a("      }".to_string()));
    push_line(&mut out, &ul_a("    ]]></style>".to_string()));
    push_line(&mut out, &ul_a("    <marker id='Triangle'".to_string()));
    push_line(&mut out, &ul_a("      viewBox='0 0 10 10' refX='0' refY='5' ".to_string()));
    push_line(&mut out, &ul_a("      markerUnits='strokeWidth'".to_string()));
    push_line(&mut out, &ul_a("      markerWidth='4' markerHeight='3'".to_string()));
    push_line(&mut out, &ul_a("      orient='auto'>".to_string()));
    push_line(&mut out, &ul_a("      <path d='M 0 0 L 10 5 L 0 10 z' />".to_string()));
    push_line(&mut out, &ul_a("    </marker>".to_string()));
    push_line(&mut out, &ul_a("    <filter id='dropShadow' x='-.1' y='-.1' width='200' height='120'>".to_string()));
    push_line(&mut out, &ul_a("       <feOffset in='SourceGraphic' dx='0' dy='0' result='orig' />".to_string()));
    push_line(&mut out, &ul_a("       <feGaussianBlur in='SourceAlpha' stdDeviation='.3' result='shadow' />".to_string()));
    push_line(&mut out, &ul_a("       <feOffset in='shadow' dx='0' dy='.1' result='movedShadow' />".to_string()));
    push_line(&mut out, &ul_a("       <feMerge>".to_string()));
    push_line(&mut out, &ul_a("           <feMergeNode in='movedShadow' />".to_string()));
    push_line(&mut out, &ul_a("           <feMergeNode in='orig' />".to_string()));
    push_line(&mut out, &ul_a("       </feMerge>".to_string()));
    push_line(&mut out, &ul_a("    </filter>".to_string()));
    push_line(&mut out, &ul_a("  </defs>".to_string()));

    let np: usize = f.players.len();
    let mut p: usize = 0;
    while p < np {
        draw_player_svg(f, &f.players[p], &mut out, ctx)?;
        p += 1;
    }
    let nm: usize = f.messages.len();
    let mut m: usize = 0;
    while m < nm {
        draw_message_svg(f, &f.messages[m], &mut out)?;
        m += 1;
    }

    push_line(&mut out, &ul_a("</svg>".to_string()));
    return Ok(out);
}

// Player.java:127-148
fn draw_player_svg(f: &Flow, p: &Player, out: &mut U, ctx: &mut RunContext) -> Result<(), Ex> {
    let wid: f64 = (f64_of_i64(get_column_pitch(f)) * f64_of_i64(parse_java_int(&get_vml_box_width_percentage(f))?)) / 100.0;
    let ht: f64 = 2.0;
    let mut stagger: f64 = 0.0;
    if get_stagger(f) {
        stagger = (ht + 2.0) * f64_of_i64(ctx.mynum % 2);
        ctx.mynum += 1;
    }
    let ycenter: f64 = stagger - ht / 2.0;
    let center: i64 = idiv(p.column * get_vml_x_size(f), get_total_width(f))?;

    push_str(out, "<rect x='".to_string());
    push_str(out, fmt_d(f64_of_i64(center) - wid / 2.0));
    push_str(out, "' y='".to_string());
    push_str(out, fmt_d(ycenter - ht / 2.0));
    push_str(out, "' width='".to_string());
    push_str(out, fmt_d(wid));
    push_str(out, "' height='".to_string());
    push_str(out, fmt_d(ht));
    push_str(out, "' rx='.6' ry='.6' filter='url(#dropShadow)'>".to_string());
    push_units(out, &ul_a("</rect>".to_string()));
    out.push(0x0A);

    let nc: usize = p.caption.len();
    let mut i: usize = 0;
    while i < nc {
        let mut ln: U = ul_a(" <text text-anchor='middle' y='".to_string());
        push_str(&mut ln, fmt_d(ycenter + f64_of_usize(i) + 0.4));
        push_units(&mut ln, &ul_a("' x='".to_string()));
        push_str(&mut ln, fmt_d(f64_of_i64(center)));
        push_units(&mut ln, &ul_a("'>".to_string()));
        push_line(out, &ln);
        push_line(out, &xml_escape(&p.caption[i]));
        push_line(out, &ul_a(" </text> ".to_string()));
        i += 1;
    }

    let mut ln3: U = ul_a("<line x1='".to_string());
    push_str(&mut ln3, fmt_d(f64_of_i64(center)));
    push_units(&mut ln3, &ul_a("' y1='".to_string()));
    push_str(&mut ln3, fmt_d(ycenter - ht / 2.0 + ht));
    push_units(&mut ln3, &ul_a("' x2='".to_string()));
    push_str(&mut ln3, fmt_d(f64_of_i64(center)));
    push_units(&mut ln3, &ul_a("' y2='".to_string()));
    push_units(&mut ln3, &fmt_i(get_vml_y_size(f)));
    push_units(&mut ln3, &ul_a("'>".to_string()));
    push_units(out, &ln3);
    push_units(out, &ul_a("</line>".to_string()));
    out.push(0x0A);
    return Ok(());
}

// Message.java:226-231 (textout)
fn textout(out: &mut U, textx: f64, texty: f64, i: f64, text: &U) {
    let mut ln: U = ul_a(" <text y='".to_string());
    push_str(&mut ln, fmt_d(texty + i));
    push_units(&mut ln, &ul_a("' x='".to_string()));
    push_str(&mut ln, fmt_d(textx));
    push_units(&mut ln, &ul_a("'>".to_string()));
    push_line(out, &ln);
    push_line(out, text);
    push_line(out, &ul_a(" </text> ".to_string()));
}

// Message.java:253-279
fn draw_message_svg(f: &Flow, m: &Message, out: &mut U) -> Result<(), Ex> {
    let fx: i64 = column_of(f, &m.from_)?;
    let tx: i64 = column_of(f, &m.to)?;
    let startx: i64 = if fx < tx { fx } else { tx };
    let textx: f64 = 0.2 + f64_of_i64(startx);
    let texty: f64 = -0.2 + f64_of_i64(m.line);
    let arrowx: f64 = 0.4;

    if is_self(m) {
        let nc: usize = m.caption.len();
        let mut i: usize = 0;
        while i < nc {
            textout(out, textx, texty, f64_of_usize(i), &xml_escape(&m.caption[i]));
            i += 1;
        }
    } else if is_undirected(m) {
        let nc: usize = m.caption.len();
        let mut i: usize = 0;
        while i < nc {
            textout(out, textx, texty, f64_of_usize(i), &xml_escape(&m.caption[i]));
            i += 1;
        }
        let mut ln: U = ul_a(" <line class='dash' x1='".to_string());
        push_units(&mut ln, &fmt_i(fx));
        push_units(&mut ln, &ul_a("' y1='".to_string()));
        push_units(&mut ln, &fmt_i(m.line));
        push_units(&mut ln, &ul_a("' x2='".to_string()));
        push_units(&mut ln, &fmt_i(tx));
        push_units(&mut ln, &ul_a("' y2='".to_string()));
        push_units(&mut ln, &fmt_i(m.line));
        push_units(&mut ln, &ul_a("'/>".to_string()));
        push_line(out, &ln);
    } else {
        let nc: usize = m.caption.len();
        let mut i: usize = 0;
        while i < nc {
            textout(out, textx, texty, f64_of_usize(i), &xml_escape(&m.caption[i]));
            i += 1;
        }
        let dx: f64 = if tx > fx { -arrowx } else { arrowx };
        let mut ln: U = ul_a(" <line marker-end='url(#Triangle)' x1='".to_string());
        push_units(&mut ln, &fmt_i(fx));
        push_units(&mut ln, &ul_a("' y1='".to_string()));
        push_units(&mut ln, &fmt_i(m.line));
        push_units(&mut ln, &ul_a("' x2='".to_string()));
        push_str(&mut ln, fmt_d(f64_of_i64(tx) + dx));
        push_units(&mut ln, &ul_a("' y2='".to_string()));
        push_units(&mut ln, &fmt_i(m.line));
        push_units(&mut ln, &ul_a("'/>".to_string()));
        push_line(out, &ln);
    }
    return Ok(());
}

fn f64_of_i64(v: i64) -> f64 {
    return v as f64;
}

fn f64_of_usize(v: usize) -> f64 {
    return v as f64;
}

fn i64_of_usize(v: usize) -> i64 {
    return v as i64;
}
