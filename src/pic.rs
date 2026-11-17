// PIC renderer (DESIGN.md section 3.7; pic.py).
// Flow.java:449-576 (plotOn), Player.java:124-126 (drawPictureOn),
// Message.java:235-252 (drawPictureOn).  PIC never sets or reads
// vml_x_size/vml_y_size (Java leaves them 0).

use crate::constants::MODE_XML;
use crate::errors::Ex;
use crate::jcompat::U;
use crate::jcompat::fmt_d;
use crate::jcompat::ul_a;
use crate::model::Message;
use crate::model::Player;
use crate::model::is_self;
use crate::model::is_undirected;
use crate::flow::Flow;
use crate::flow::start_message_numbers;
use crate::flow::get_total_height;

pub fn pic_on(f: &mut Flow, mode: usize) -> Result<U, Ex> {
    let mut out: U = vec![];
    start_message_numbers(f);

    if (mode & MODE_XML) != 0 {
        push_line(&mut out, &ul_a("<plot>".to_string()));
        push_line(&mut out, &ul_a("<picture>".to_string()));
    }

    // prologue, Flow.java:460-461
    let mut pro: U = ul_a(".PS ".to_string());
    push_units(&mut pro, &f.pic_x_size);
    push_units(&mut pro, &ul_a(" ".to_string()));
    push_units(&mut pro, &f.pic_y_size);
    push_line(&mut out, &pro);

    let np: usize = f.players.len();
    let mut p: usize = 0;
    while p < np {
        draw_player_pic(f, &f.players[p], &mut out);
        p += 1;
    }
    let nm: usize = f.messages.len();
    let mut m: usize = 0;
    while m < nm {
        draw_message_pic(f, &f.messages[m], &mut out);
        m += 1;
    }

    // epilogue, Flow.java:575-576: println(".PE\n") prints .PE plus a blank line
    push_line(&mut out, &ul_a(".PE".to_string()));
    out.push(0x0A);
    return Ok(out);
}

fn push_units(out: &mut U, t: &U) {
    for c in t {
        out.push(*c);
    }
}

fn push_line(out: &mut U, t: &U) {
    push_units(out, t);
    out.push(0x0A);
}

// Player.java:124-126
fn draw_player_pic(f: &Flow, p: &Player, out: &mut U) {
    let height_expr: U = ul_a(fmt_d(f64_of_i64(get_total_height(f)) / 6.0 + 0.25));
    push_units(out, &ul_a("P".to_string()));
    push_units(out, &p.name);
    push_units(out, &ul_a(": { box \"".to_string()));
    push_units(out, &p.caption_string);
    push_units(out, &ul_a("\"; down; L".to_string()));
    push_units(out, &p.name);
    push_units(out, &ul_a(": line ".to_string()));
    push_units(out, &height_expr);
    push_units(out, &ul_a("i from last box.s }; right; move 2i".to_string()));
    out.push(0x0A);
}

// Message.java:235-252
fn draw_message_pic(f: &Flow, m: &Message, out: &mut U) {
    // Java: -1.0D * line / 6.0D
    let off: U = ul_a(fmt_d(-1.0 * f64_of_i64(m.line) / 6.0));
    if is_self(m) {
        push_units(out, &ul_a("{ line invis \"".to_string()));
        push_units(out, &m.caption_string);
        push_units(out, &ul_a("\" \"\" from L".to_string()));
        push_units(out, &m.from_);
        push_units(out, &ul_a(".start + (0,".to_string()));
        push_units(out, &off);
        push_units(out, &ul_a(") to L".to_string()));
        push_units(out, &m.to);
        push_units(out, &ul_a(".start + (0.1,".to_string()));
        push_units(out, &off);
        push_units(out, &ul_a(");  }".to_string()));
        out.push(0x0A);
    } else if is_undirected(m) {
        push_units(out, &ul_a("{ line dotted \"".to_string()));
        push_units(out, &m.caption_string);
        push_units(out, &ul_a("\" \"\" from L".to_string()));
        push_units(out, &m.from_);
        push_units(out, &ul_a(".start + (0,".to_string()));
        push_units(out, &off);
        push_units(out, &ul_a(") to L".to_string()));
        push_units(out, &m.to);
        push_units(out, &ul_a(".start + (0,".to_string()));
        push_units(out, &off);
        push_units(out, &ul_a(");  }".to_string()));
        out.push(0x0A);
    } else {
        push_units(out, &ul_a("{ arrow \"".to_string()));
        push_units(out, &m.caption_string);
        push_units(out, &ul_a("\" \"\" from L".to_string()));
        push_units(out, &m.from_);
        push_units(out, &ul_a(".start + (0,".to_string()));
        push_units(out, &off);
        push_units(out, &ul_a(") to L".to_string()));
        push_units(out, &m.to);
        push_units(out, &ul_a(".start + (0,".to_string()));
        push_units(out, &off);
        push_units(out, &ul_a(");  }".to_string()));
        out.push(0x0A);
    }
}

fn f64_of_i64(v: i64) -> f64 {
    return v as f64;
}
