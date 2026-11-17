// VML renderer, also used for -vxml (DESIGN.md section 3.4.4; vml.py).
// Flow.java:514-539 + 579-583 (drawPictureOn VML branches),
// Player.java:149-179 (drawPictureOn VML branch),
// Message.java:680-738 (drawPictureOn VML branch).
// -vxml opens <plot>/<picture> and never closes them.

use crate::constants::VML_TEXT_XOFF;
use crate::constants::VML_TEXT_YOFF;
use crate::constants::MODE_XML;
use crate::context::RunContext;
use crate::errors::Ex;
use crate::flow::Flow;
use crate::flow::xml_escape;
use crate::flow::column_of;
use crate::flow::start_message_numbers;
use crate::flow::next_message_number;
use crate::flow::get_do_note_numbering;
use crate::flow::get_total_height;
use crate::flow::get_total_width;
use crate::flow::get_column_pitch;
use crate::flow::get_stagger;
use crate::flow::get_vml_x_size;
use crate::flow::get_vml_y_size;
use crate::flow::get_vml_style;
use crate::flow::get_vml_arrow_style;
use crate::flow::get_vml_dotted_style;
use crate::flow::get_vml_box_style;
use crate::flow::get_vml_text_style;
use crate::flow::get_vml_box_text_style;
use crate::jcompat::U;
use crate::jcompat::fmt_d;
use crate::jcompat::fmt_i;
use crate::flow::get_vml_box_width_percentage;
use crate::jcompat::idiv;
use crate::jcompat::parse_java_int;
use crate::jcompat::ul_a;
use crate::jcompat::exc_index;
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

pub fn push_str(out: &mut U, s: String) {
    push_units(out, &ul_a(s));
}

// arr_at on a list of unit strings: a missing element throws IndexError.
fn arr_at(v: &Vec<U>, k: i64) -> Result<U, Ex> {
    if k < 0 || k >= v.len() as i64 {
        return Err(exc_index());
    }
    return Ok(v[k as usize].clone());
}

pub fn vml_on(f: &mut Flow, mode: usize, ctx: &mut RunContext) -> Result<U, Ex> {
    let mut out: U = vec![];
    start_message_numbers(f);

    if (mode & MODE_XML) != 0 {
        push_line(&mut out, &ul_a("<plot>".to_string()));
        push_line(&mut out, &ul_a("<picture>".to_string()));
    }

    // prologue, Flow.java:514-538
    push_line(&mut out, &ul_a("<html xmlns:v='urn:schemas-microsoft-com:vml'".to_string()));
    push_line(&mut out, &ul_a("xmlns='http://www.w3.org/TR/REC-html40'>".to_string()));
    push_line(&mut out, &ul_a("".to_string()));
    push_line(&mut out, &ul_a("<head>".to_string()));
    push_line(&mut out, &ul_a("".to_string()));
    push_line(&mut out, &ul_a("<!--[if !mso]>".to_string()));
    push_line(&mut out, &ul_a("<style>".to_string()));
    push_line(&mut out, &ul_a("v\\:* {behavior:url(#default#VML);}".to_string()));
    push_line(&mut out, &ul_a(".shape {behavior:url(#default#VML);}".to_string()));
    push_line(&mut out, &ul_a("</style>".to_string()));
    push_line(&mut out, &ul_a("<![endif]-->".to_string()));
    push_line(&mut out, &ul_a("".to_string()));
    push_line(&mut out, &ul_a("</head>".to_string()));
    push_line(&mut out, &ul_a("".to_string()));
    push_line(&mut out, &ul_a("<body>".to_string()));
    f.vml_y_size = get_total_height(f) * 2;
    f.vml_x_size = get_total_width(f);
    let vml_y_pt: i64 = idiv(432 * f.vml_y_size, 100)?;
    let vml_x_pt: i64 = idiv(432 * f.vml_x_size, 100)?;
    let mut ln: U = ul_a("<!-- totalHeight =".to_string());
    push_units(&mut ln, &fmt_i(get_total_height(f)));
    push_units(&mut ln, &ul_a("-->".to_string()));
    push_line(&mut out, &ln);
    let mut ln2: U = ul_a("<!-- totalWidth =".to_string());
    push_units(&mut ln2, &fmt_i(get_total_width(f)));
    push_units(&mut ln2, &ul_a("-->".to_string()));
    push_line(&mut out, &ln2);
    let mut gr: U = ul_a("<v:group style='position:relative;left:1pt;top:2pt;width:".to_string());
    push_units(&mut gr, &fmt_i(vml_x_pt));
    push_units(&mut gr, &ul_a("pt;height:".to_string()));
    push_units(&mut gr, &fmt_i(vml_y_pt));
    push_units(&mut gr, &ul_a("pt;' coordsize='".to_string()));
    push_units(&mut gr, &fmt_i(f.vml_x_size));
    push_units(&mut gr, &ul_a(",".to_string()));
    push_units(&mut gr, &fmt_i(f.vml_y_size));
    push_units(&mut gr, &ul_a("' >".to_string()));
    push_line(&mut out, &gr);
    push_line(&mut out, &ul_a("".to_string()));

    let np: usize = f.players.len();
    let mut p: usize = 0;
    while p < np {
        draw_player_vml(f, &f.players[p], &mut out, ctx)?;
        p += 1;
    }
    let nm: usize = f.messages.len();
    let mut m: usize = 0;
    while m < nm {
        draw_message_vml(f, m, &mut out)?;
        m += 1;
    }

    // epilogue, Flow.java:579-583: three trailing spaces on the last line
    push_line(&mut out, &ul_a("".to_string()));
    push_line(&mut out, &ul_a("</v:group>".to_string()));
    push_line(&mut out, &ul_a("</body>".to_string()));
    push_line(&mut out, &ul_a("</html>   ".to_string()));
    return Ok(out);
}

// Player.java:149-179
fn draw_player_vml(f: &Flow, p: &Player, out: &mut U, ctx: &mut RunContext) -> Result<(), Ex> {
    let mut wid: i64 = idiv(get_column_pitch(f) * get_vml_x_size(f), get_total_width(f))?;
    wid = wid * parse_java_int(&get_vml_box_width_percentage(f))?;
    wid = idiv(wid, 100)?;
    let ht: f64 = 4.0;
    let mut stagger: f64 = 0.0;
    if get_stagger(f) {
        stagger = (ht + 2.0) * f64_of_i64(ctx.mynum % 2);
        ctx.mynum += 1;
    }
    let ycenter: f64 = stagger - ht / 2.0;
    let center: i64 = idiv(p.column * get_vml_x_size(f), get_total_width(f))?;

    push_str(out, "<v:rect ".to_string());
    push_units(out, &get_vml_box_style(f));
    push_str(out, " style='position:absolute;left:".to_string());
    push_str(out, fmt_d(f64_of_i64(center) - f64_of_i64(wid) / 2.0));
    push_str(out, ";top:".to_string());
    push_str(out, fmt_d(ycenter - ht / 2.0));
    push_str(out, ";width:".to_string());
    push_units(out, &fmt_i(wid));
    push_str(out, ";height:".to_string());
    push_str(out, fmt_d(ht));
    push_str(out, "'>".to_string());
    let mut ln: U = ul_a("  <v:stroke   ".to_string());
    push_units(&mut ln, &get_vml_style(f));
    push_units(&mut ln, &ul_a(" />".to_string()));
    push_line(out, &ln);
    push_units(out, &ul_a("</v:rect>".to_string()));

    let mut sh: U = ul_a(" <v:shape fillcolor='black' strokeweight='0pt' coordorigin='0 0' coordsize='100 100' style='position:absolute;top:".to_string());
    push_str(&mut sh, fmt_d(0.2 + ycenter));
    push_str(&mut sh, ";left:".to_string());
    push_str(&mut sh, fmt_d(f64_of_i64(center) - f64_of_i64(wid) / 2.0 + 0.3));
    push_str(&mut sh, ";width:".to_string());
    push_units(&mut sh, &fmt_i(wid));
    push_str(&mut sh, ";height:".to_string());
    push_str(&mut sh, fmt_d(ht));
    push_str(&mut sh, "' path = 'm 0,0 l 100,0  e'>".to_string());
    push_line(out, &sh);
    push_line(out, &ul_a(" <v:path textpathok='True'/> ".to_string()));
    let cap0: U = arr_at(&p.caption, 0)?;
    let mut tp: U = ul_a(" <v:textpath on='True' string=\"".to_string());
    push_units(&mut tp, &xml_escape(&cap0));
    push_str(&mut tp, "\" ".to_string());
    push_units(&mut tp, &get_vml_box_text_style(f));
    push_str(&mut tp, "/> ".to_string());
    push_line(out, &tp);
    push_line(out, &ul_a(" </v:shape> ".to_string()));

    let mut ln2: U = ul_a("<v:line  from='".to_string());
    push_str(&mut ln2, fmt_d(f64_of_i64(center)));
    push_str(&mut ln2, ",".to_string());
    push_str(&mut ln2, fmt_d(ycenter - ht / 2.0 + ht));
    push_str(&mut ln2, "' to='".to_string());
    push_str(&mut ln2, fmt_d(f64_of_i64(center)));
    push_str(&mut ln2, ",".to_string());
    push_units(&mut ln2, &fmt_i(get_vml_y_size(f)));
    push_str(&mut ln2, "'>".to_string());
    push_units(out, &ln2);
    let mut ln3: U = ul_a("  <v:stroke ".to_string());
    push_units(&mut ln3, &get_vml_style(f));
    push_units(&mut ln3, &ul_a(" />".to_string()));
    push_line(out, &ln3);
    push_units(out, &ul_a("</v:line>".to_string()));
    return Ok(());
}

// Message.java:680-738
fn draw_message_vml(f: &mut Flow, idx: usize, out: &mut U) -> Result<(), Ex> {
    let m: Message = f.messages[idx].clone();
    let mut v_y: i64 = idiv(m.line * get_vml_y_size(f), get_total_height(f))?;
    let from_x: i64 = idiv(column_of(f, &m.from_)? * get_vml_x_size(f), get_total_width(f))?;
    let to_x: i64 = idiv(column_of(f, &m.to)? * get_vml_x_size(f), get_total_width(f))?;
    let start_x: i64 = VML_TEXT_XOFF + (if from_x < to_x { from_x } else { to_x });
    let text_off: i64 = VML_TEXT_YOFF;

    let cap0: U = arr_at(&m.caption, 0)?;
    let mut xml_caption: U = xml_escape(&cap0);

    // honor message numbering (Message.java:690-696)
    if !is_self(&m) || get_do_note_numbering(f) {
        let num: i64 = next_message_number(f);
        if num != -1 {
            f.messages[idx].my_number = num;
            let mut numbered: U = ul_a("(".to_string());
            push_units(&mut numbered, &fmt_i(num));
            push_str(&mut numbered, ") ".to_string());
            push_units(&mut numbered, &xml_caption);
            xml_caption = numbered;
        }
    }

    // first line (Message.java:700-707)
    write_shape(out, v_y - text_off, start_x, &xml_caption, f);

    // other lines (Message.java:710-720): v_y moves down per extra caption line
    let nc: usize = m.caption.len();
    let mut ci: usize = 1;
    while ci < nc {
        v_y = v_y + idiv(get_vml_y_size(f), get_total_height(f))?;
        let cap: U = arr_at(&m.caption, ci64(ci))?;
        write_shape(out, v_y - text_off, start_x, &xml_escape(&cap), f);
        ci += 1;
    }

    // arrows, at the FINAL v_y (DESIGN.md section 3.8 #5)
    if is_self(&m) {
        // Message.java:723-724: nothing to do
    } else if is_undirected(&m) {
        let mut ln: U = ul_a(" <v:line from='".to_string());
        push_units(&mut ln, &fmt_i(from_x));
        push_str(&mut ln, ",".to_string());
        push_units(&mut ln, &fmt_i(v_y));
        push_str(&mut ln, "' to='".to_string());
        push_units(&mut ln, &fmt_i(to_x));
        push_str(&mut ln, ",".to_string());
        push_units(&mut ln, &fmt_i(v_y));
        push_str(&mut ln, "'>".to_string());
        push_line(out, &ln);
        let mut st: U = ul_a("  <v:stroke ".to_string());
        push_units(&mut st, &get_vml_dotted_style(f));
        push_str(&mut st, "  ".to_string());
        push_units(&mut st, &get_vml_style(f));
        push_str(&mut st, " />".to_string());
        push_line(out, &st);
        push_line(out, &ul_a(" </v:line>".to_string()));
    } else {
        let mut ln: U = ul_a(" <v:line from='".to_string());
        push_units(&mut ln, &fmt_i(from_x));
        push_str(&mut ln, ",".to_string());
        push_units(&mut ln, &fmt_i(v_y));
        push_str(&mut ln, "' to='".to_string());
        push_units(&mut ln, &fmt_i(to_x));
        push_str(&mut ln, ",".to_string());
        push_units(&mut ln, &fmt_i(v_y));
        push_str(&mut ln, "'>".to_string());
        push_line(out, &ln);
        let mut st: U = ul_a("  <v:stroke   ".to_string());
        push_units(&mut st, &get_vml_style(f));
        push_str(&mut st, " ".to_string());
        push_units(&mut st, &get_vml_arrow_style(f));
        push_str(&mut st, " />".to_string());
        push_line(out, &st);
        push_line(out, &ul_a(" </v:line>".to_string()));
    }
    return Ok(());
}

// The two v:shape text blocks (first line and later lines) share this shape.
fn write_shape(out: &mut U, top: i64, left: i64, text: &U, f: &Flow) {
    let mut sh: U = ul_a(" <v:shape fillcolor='black' strokeweight='0pt' coordorigin='0 0' coordsize='100 100' style='position:absolute;top:".to_string());
    push_units(&mut sh, &fmt_i(top));
    push_str(&mut sh, ";left:".to_string());
    push_units(&mut sh, &fmt_i(left));
    push_str(&mut sh, ";width:1;height:1' path = 'm 0,0 l 100,0  e'>".to_string());
    push_line(out, &sh);
    push_line(out, &ul_a(" <v:path textpathok='True'/> ".to_string()));
    let mut tp: U = ul_a(" <v:textpath on='True' string=\"".to_string());
    push_units(&mut tp, text);
    push_str(&mut tp, "\" ".to_string());
    push_units(&mut tp, &get_vml_text_style(f));
    push_str(&mut tp, " /> ".to_string());
    push_line(out, &tp);
    push_line(out, &ul_a(" </v:shape> ".to_string()));
}

fn f64_of_i64(v: i64) -> f64 {
    return v as f64;
}

fn ci64(v: usize) -> i64 {
    return v as i64;
}
