// OmniGraffle renderer (DESIGN.md section 3.4.5; graffle.py).
// Flow.java:540-559 and 584-598 (drawPictureOn GRAFFLE branches; messages come
// BEFORE players here),
// Player.java:182-352 (drawPictureOn GRAFFLE branch),
// Message.java:282-678 (drawPictureOn GRAFFLE branch).
// Fixed plist text copied verbatim from the Java literals, one array entry
// per Java println.  repeatLabel debug lines go to ctx.err (stderr is never
// compared); caption[0] gains the "N. " numbering prefix and survives to the
// epilogue comments.

use crate::constants::MODE_XML;
use crate::constants::GRAFFLE_MULT;
use crate::constants::GRAFFLE_TOP_MARG;
use crate::constants::GRAFFLE_LINE_TXT_OFST;
use crate::context::RunContext;
use crate::errors::Ex;
use crate::flow::Flow;
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
use crate::jcompat::U;
use crate::jcompat::fmt_d;
use crate::jcompat::fmt_i;
use crate::flow::get_vml_box_width_percentage;
use crate::jcompat::idiv;
use crate::jcompat::parse_java_int;
use crate::jcompat::ul_a;
use crate::jcompat::from_units;
use crate::jcompat::exc_index;
use crate::model::Message;
use crate::model::Player;
use crate::model::is_note;
use crate::model::is_self;
use crate::model::is_undirected;
use crate::text::print_comment;

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

fn arr_at(v: &Vec<U>, k: i64) -> Result<U, Ex> {
    if k < 0 || k >= v.len() as i64 {
        return Err(exc_index());
    }
    return Ok(v[k as usize].clone());
}

fn emit(out: &mut U, lines: Vec<U>) {
    for s in lines {
        push_line(out, &s);
    }
}

// Player.java:213-271 (the box style dict)
fn emit_player_style(out: &mut U) {
    let lines: Vec<U> = vec![
        ul_a("<key>Style</key>".to_string()),
        ul_a("<dict>".to_string()),
        ul_a("<key>shadow</key>".to_string()),
        ul_a("<dict>".to_string()),
        ul_a("<key>Draws</key>".to_string()),
        ul_a("<string>YES</string>".to_string()),
        ul_a("</dict>".to_string()),
        ul_a("<key>fill</key>".to_string()),
        ul_a("<dict>".to_string()),
        ul_a("<key>FillType</key>".to_string()),
        ul_a("<integer>3</integer>".to_string()),
        ul_a("<key>GradientCenter</key>".to_string()),
        ul_a("<string>{0.266667, -0.190476}</string>".to_string()),
        ul_a("<key>GradientColor</key>".to_string()),
        ul_a("<dict>".to_string()),
        ul_a("<key>a</key>".to_string()),
        ul_a("<string>0.51</string>".to_string()),
        ul_a("<key>b</key>".to_string()),
        ul_a("<string>0.869565</string>".to_string()),
        ul_a("<key>g</key>".to_string()),
        ul_a("<string>0.869565</string>".to_string()),
        ul_a("<key>r</key>".to_string()),
        ul_a("<string>0.869565</string>".to_string()),
        ul_a("</dict>".to_string()),
        ul_a("</dict>".to_string()),
        ul_a("<key>stroke</key>".to_string()),
        ul_a("<dict>".to_string()),
        ul_a("<key>CornerRadius</key>".to_string()),
        ul_a("<real>5</real>".to_string()),
        ul_a("<key>Color</key>".to_string()),
        ul_a("<dict>".to_string()),
        ul_a("<key>a</key>".to_string()),
        ul_a("<string>0.14</string>".to_string()),
        ul_a("<key>b</key>".to_string()),
        ul_a("<string>0</string>".to_string()),
        ul_a("<key>g</key>".to_string()),
        ul_a("<string>0</string>".to_string()),
        ul_a("<key>r</key>".to_string()),
        ul_a("<string>0</string>".to_string()),
        ul_a("</dict>".to_string()),
        ul_a("</dict>".to_string()),
        ul_a("</dict>".to_string()),
        ul_a("<key>Text</key>".to_string()),
        ul_a("<dict>".to_string()),
        ul_a("<key>Align</key>".to_string()),
        ul_a("<integer>0</integer>".to_string()),
        ul_a("<key>Text</key>".to_string()),
    ];
    emit(out, lines);
}

// The RTF header lines (Player.java:281-284, Message.java:485-488).
fn emit_rtf_head(out: &mut U) {
    let lines: Vec<U> = vec![
        ul_a(r"<string>{\rtf1\mac\ansicpg10000\cocoartf824\cocoasubrtf110".to_string()),
        ul_a(r"{\fonttbl\f0\fswiss\fcharset77 Helvetica;}".to_string()),
        ul_a(r"{\colortbl;\red255\green255\blue255;}".to_string()),
        ul_a(r"\pard\tx560\tx1120\tx1680\tx2240\tx2800\tx3360\tx3920\tx4480\tx5040\tx5600\tx6160\tx6720\qc".to_string()),
    ];
    emit(out, lines);
}

// Player.java:314-347 (lifeline style plus Tail link)
fn emit_player_line_tail(out: &mut U) {
    let lines: Vec<U> = vec![
        ul_a("<key>Style</key>".to_string()),
        ul_a("<dict>".to_string()),
        ul_a("<key>stroke</key>".to_string()),
        ul_a("<dict>".to_string()),
        ul_a("<key>Cap</key>".to_string()),
        ul_a("<integer>0</integer>".to_string()),
        ul_a("<key>HeadArrow</key>".to_string()),
        ul_a("<string>0</string>".to_string()),
        ul_a("<key>HeadScale</key>".to_string()),
        ul_a("<real>0.5</real>".to_string()),
        ul_a("<key>TailArrow</key>".to_string()),
        ul_a("<string>0</string>".to_string()),
        ul_a("<key>TailScale</key>".to_string()),
        ul_a("<real>0.5</real>".to_string()),
        ul_a("</dict>".to_string()),
        ul_a("</dict>".to_string()),
        ul_a("<key>Tail</key>".to_string()),
        ul_a("<dict>".to_string()),
        ul_a("<key>ID</key>".to_string()),
    ];
    emit(out, lines);
}

// Message.java:310-314 (group wrapper head)
fn emit_group_head(out: &mut U) {
    let lines: Vec<U> = vec![
        ul_a("<dict>".to_string()),
        ul_a("<key>Class</key>".to_string()),
        ul_a("<string>Group</string>".to_string()),
        ul_a("<key>Graphics</key>".to_string()),
        ul_a("<array>".to_string()),
    ];
    emit(out, lines);
}

// Message.java:349-426 (note text box style)
fn emit_note_style(out: &mut U) {
    let lines: Vec<U> = vec![
        ul_a("<key>Style</key>".to_string()),
        ul_a("<dict>".to_string()),
        ul_a("<key>fill</key>".to_string()),
        ul_a("<dict>".to_string()),
        ul_a("<key>Color</key>".to_string()),
        ul_a("<dict>".to_string()),
        ul_a("<key>a</key>".to_string()),
        ul_a("<string>0.72</string>".to_string()),
        ul_a("<key>b</key>".to_string()),
        ul_a("<string>0.804798</string>".to_string()),
        ul_a("<key>g</key>".to_string()),
        ul_a("<string>1</string>".to_string()),
        ul_a("<key>r</key>".to_string()),
        ul_a("<string>1</string>".to_string()),
        ul_a("</dict>".to_string()),
        ul_a("</dict>".to_string()),
        ul_a("<key>shadow</key>".to_string()),
        ul_a("<dict>".to_string()),
        ul_a("<key>Color</key>".to_string()),
        ul_a("<dict>".to_string()),
        ul_a("<key>a</key>".to_string()),
        ul_a("<string>0.21</string>".to_string()),
        ul_a("<key>b</key>".to_string()),
        ul_a("<string>0</string>".to_string()),
        ul_a("<key>g</key>".to_string()),
        ul_a("<string>0</string>".to_string()),
        ul_a("<key>r</key>".to_string()),
        ul_a("<string>0</string>".to_string()),
        ul_a("</dict>".to_string()),
        ul_a("<key>Fuzziness</key>".to_string()),
        ul_a("<real>9.0033035278320312</real>".to_string()),
        ul_a("<key>ShadowVector</key>".to_string()),
        ul_a("<string>{0, 2}</string>".to_string()),
        ul_a("</dict>".to_string()),
        ul_a("<key>stroke</key>".to_string()),
        ul_a("<dict>".to_string()),
        ul_a("<key>Cap</key>".to_string()),
        ul_a("<integer>0</integer>".to_string()),
        ul_a("<key>CornerRadius</key>".to_string()),
        ul_a("<real>7</real>".to_string()),
        ul_a("<key>Width</key>".to_string()),
        ul_a("<string>2</string>".to_string()),
        ul_a("<key>Color</key>".to_string()),
        ul_a("<dict>".to_string()),
        ul_a("<key>a</key>".to_string()),
        ul_a("<string>0.8</string>".to_string()),
        ul_a("<key>b</key>".to_string()),
        ul_a("<string>0</string>".to_string()),
        ul_a("<key>g</key>".to_string()),
        ul_a("<string>0.95</string>".to_string()),
        ul_a("<key>r</key>".to_string()),
        ul_a("<string>0.95</string>".to_string()),
        ul_a("</dict>".to_string()),
        ul_a("</dict>".to_string()),
        ul_a("</dict>".to_string()),
    ];
    emit(out, lines);
}

// Message.java:431-473 (non-note text box style)
fn emit_text_style(out: &mut U) {
    let lines: Vec<U> = vec![
        ul_a("<key>Style</key>".to_string()),
        ul_a("<dict>".to_string()),
        ul_a("<key>fill</key>".to_string()),
        ul_a("<dict>".to_string()),
        ul_a("<key>Color</key>".to_string()),
        ul_a("<dict>".to_string()),
        ul_a("<key>a</key>".to_string()),
        ul_a("<string>0.42</string>".to_string()),
        ul_a("<key>b</key>".to_string()),
        ul_a("<string>1</string>".to_string()),
        ul_a("<key>g</key>".to_string()),
        ul_a("<string>1</string>".to_string()),
        ul_a("<key>r</key>".to_string()),
        ul_a("<string>1</string>".to_string()),
        ul_a("</dict>".to_string()),
        ul_a("</dict>".to_string()),
        ul_a("<key>shadow</key>".to_string()),
        ul_a("<dict>".to_string()),
        ul_a("<key>Draws</key>".to_string()),
        ul_a("<string>NO</string>".to_string()),
        ul_a("</dict>".to_string()),
        ul_a("<key>stroke</key>".to_string()),
        ul_a("<dict>".to_string()),
        ul_a("<key>Cap</key>".to_string()),
        ul_a("<integer>0</integer>".to_string()),
        ul_a("<key>CornerRadius</key>".to_string()),
        ul_a("<real>7</real>".to_string()),
        ul_a("<key>Draws</key>".to_string()),
        ul_a("<string>NO</string>".to_string()),
        ul_a("</dict>".to_string()),
        ul_a("</dict>".to_string()),
    ];
    emit(out, lines);
}

// Message.java:550-576 (shadow block, only inside a note's line style)
fn emit_line_note_shadow(out: &mut U) {
    let lines: Vec<U> = vec![
        ul_a("<key>shadow</key>".to_string()),
        ul_a("<dict>".to_string()),
        ul_a("<key>Color</key>".to_string()),
        ul_a("<dict>".to_string()),
        ul_a("<key>a</key>".to_string()),
        ul_a("<string>0.26</string>".to_string()),
        ul_a("<key>b</key>".to_string()),
        ul_a("<string>0</string>".to_string()),
        ul_a("<key>g</key>".to_string()),
        ul_a("<string>0</string>".to_string()),
        ul_a("<key>r</key>".to_string()),
        ul_a("<string>0</string>".to_string()),
        ul_a("</dict>".to_string()),
        ul_a("<key>Fuzziness</key>".to_string()),
        ul_a("<real>6</real>".to_string()),
        ul_a("<key>ShadowVector</key>".to_string()),
        ul_a("<string>{0, 1}</string>".to_string()),
        ul_a("<key>Draws</key>".to_string()),
        ul_a("<string>YES</string>".to_string()),
        ul_a("</dict>".to_string()),
    ];
    emit(out, lines);
}

// Message.java:611-628 (width+color, only inside a note's stroke dict)
fn emit_line_note_style(out: &mut U) {
    let lines: Vec<U> = vec![
        ul_a("<key>Width</key>".to_string()),
        ul_a("<string>2</string>".to_string()),
        ul_a("<key>Color</key>".to_string()),
        ul_a("<dict>".to_string()),
        ul_a("<key>a</key>".to_string()),
        ul_a("<string>0.8</string>".to_string()),
        ul_a("<key>b</key>".to_string()),
        ul_a("<string>0</string>".to_string()),
        ul_a("<key>g</key>".to_string()),
        ul_a("<string>0.95</string>".to_string()),
        ul_a("<key>r</key>".to_string()),
        ul_a("<string>0.95</string>".to_string()),
        ul_a("</dict>".to_string()),
    ];
    emit(out, lines);
}

pub fn graffle_on(f: &mut Flow, mode: usize, ctx: &mut RunContext) -> Result<U, Ex> {
    let mut out: U = vec![];
    start_message_numbers(f);

    if (mode & MODE_XML) != 0 {
        push_line(&mut out, &ul_a("<plot>".to_string()));
        push_line(&mut out, &ul_a("<picture>".to_string()));
    }

    // prologue, Flow.java:540-546
    push_line(&mut out, &ul_a("<?xml version=\"1.0\" encoding=\"UTF-8\"?>".to_string()));
    let mut dt: U = ul_a("<!DOCTYPE plist PUBLIC \"-//Apple Computer//DTD PLIST 1.0//EN\" ".to_string());
    push_str(&mut dt, "\"http://www.apple.com/DTDs/PropertyList-1.0.dtd\">".to_string());
    push_line(&mut out, &dt);
    push_line(&mut out, &ul_a("<plist version=\"1.0\">".to_string()));
    push_line(&mut out, &ul_a("<dict>".to_string()));
    push_line(&mut out, &ul_a("<key>GraphicsList</key>".to_string()));
    push_line(&mut out, &ul_a("<array>".to_string()));

    // Flow.java:548-549 (H, not 2H)
    f.vml_y_size = get_total_height(f);
    f.vml_x_size = get_total_width(f);

    // messages first (Flow.java:551-554), then players (556-559)
    let nm: usize = f.messages.len();
    let mut m: usize = 0;
    while m < nm {
        draw_message_graffle(f, m, &mut out, ctx)?;
        m += 1;
    }
    let np: usize = f.players.len();
    let mut p: usize = 0;
    while p < np {
        draw_player_graffle(f, &f.players[p], &mut out, ctx)?;
        p += 1;
    }

    // epilogue, Flow.java:584-598
    push_line(&mut out, &ul_a("</array>".to_string()));
    push_line(&mut out, &ul_a("</dict>".to_string()));
    push_line(&mut out, &ul_a("</plist>".to_string()));

    if (mode & MODE_XML) != 0 {
        push_line(&mut out, &ul_a("</picture>".to_string()));
        let nm2: usize = f.messages.len();
        let mut i: usize = 0;
        while i < nm2 {
            print_comment(&f.messages[i], &mut out);
            i += 1;
        }
        push_line(&mut out, &ul_a("</plot>".to_string()));
    }
    return Ok(out);
}

// Player.java:182-352
fn draw_player_graffle(f: &Flow, p: &Player, out: &mut U, ctx: &mut RunContext) -> Result<(), Ex> {
    let pct: i64 = parse_java_int(&get_vml_box_width_percentage(f))?;
    let wid: f64 = (f64_of_i64(GRAFFLE_MULT * get_column_pitch(f) * pct)) / 100.0;
    // ht = 1.6 * caption.length * GRAFFLE_MULT (keep this operator order)
    let ht: f64 = 1.6 * f64_of_usize(p.caption.len()) * f64_of_i64(GRAFFLE_MULT);
    let mut stagger: f64 = 0.0;
    if get_stagger(f) {
        stagger = (ht + 2.0) * f64_of_i64(ctx.mynum % 2);
        ctx.mynum += 1;
    }
    let ycenter: f64 = (stagger * f64_of_i64(GRAFFLE_MULT) + f64_of_i64(GRAFFLE_TOP_MARG)) / 2.0;
    let center: i64 = idiv(p.column * get_vml_x_size(f) * GRAFFLE_MULT, get_total_width(f))?;

    // text box, Player.java:194-200
    push_line(out, &ul_a("<dict>".to_string()));
    push_line(out, &ul_a("<key>Bounds</key>".to_string()));
    let mut ln: U = ul_a("<string>{{".to_string());
    push_str(&mut ln, fmt_d(f64_of_i64(center) - wid / 2.0));
    push_str(&mut ln, ", ".to_string());
    push_str(&mut ln, fmt_d(ycenter - ht / 2.0));
    push_str(&mut ln, " }, {".to_string());
    push_str(&mut ln, fmt_d(wid));
    push_str(&mut ln, ", ".to_string());
    push_str(&mut ln, fmt_d(ht));
    push_str(&mut ln, "}}</string>".to_string());
    push_line(out, &ln);
    push_line(out, &ul_a("<key>Class</key>".to_string()));
    push_line(out, &ul_a("<string>ShapedGraphic</string>".to_string()));
    push_line(out, &ul_a("<key>FitText</key>".to_string()));
    push_line(out, &ul_a("<string>NO</string>".to_string()));
    push_line(out, &ul_a("<key>ID</key>".to_string()));
    ctx.player_id += 1; // Player.java:207: int textId = ++id
    let text_id: i64 = ctx.player_id;
    let mut idl: U = ul_a("<integer>".to_string());
    push_units(&mut idl, &fmt_i(ctx.player_id));
    push_units(&mut idl, &ul_a("</integer>".to_string()));
    push_line(out, &idl);

    // Player.java:210-281
    push_line(out, &ul_a("<key>Shape</key>".to_string()));
    push_line(out, &ul_a("<string>Rectangle</string>".to_string()));
    emit_player_style(out);

    // Player.java:285-291 (RTF text; caption[len-1] throws IndexError when
    // there is no caption line, DESIGN.md 3.8)
    emit_rtf_head(out);
    push_str(out, r"\f0\fs24 \cf0 ".to_string());
    let ncap: usize = p.caption.len();
    let mut i: usize = 0;
    while i + 1 < ncap {
        push_units(out, &p.caption[i]);
        out.push(0x5C);
        out.push(0x0A);
        i += 1;
    }
    let last: U = arr_at(&p.caption, ncap as i64 - 1)?;
    let mut tln: U = last;
    push_str(&mut tln, "}</string>".to_string());
    push_line(out, &tln);

    // Player.java:292-295
    push_line(out, &ul_a("</dict>".to_string()));
    push_line(out, &ul_a("</dict>".to_string()));

    // lifeline, Player.java:298-307
    push_line(out, &ul_a("<dict>".to_string()));
    push_line(out, &ul_a("<key>Class</key>".to_string()));
    push_line(out, &ul_a("<string>LineGraphic</string>".to_string()));
    push_line(out, &ul_a("<key>ID</key>".to_string()));
    ctx.player_id += 1; // Player.java:304: ++id
    let mut idl2: U = ul_a("<integer>".to_string());
    push_units(&mut idl2, &fmt_i(ctx.player_id));
    push_units(&mut idl2, &ul_a("</integer>".to_string()));
    push_line(out, &idl2);
    push_line(out, &ul_a("<key>Points</key>".to_string()));
    push_line(out, &ul_a("<array>".to_string()));
    let mut pt: U = ul_a("<string>{".to_string());
    push_str(&mut pt, fmt_d(f64_of_i64(center)));
    push_str(&mut pt, ", ".to_string());
    push_str(&mut pt, fmt_d(ycenter - ht / 2.0 + ht));
    push_str(&mut pt, "}</string>".to_string());
    push_line(out, &pt);
    let mut pt2: U = ul_a("<string>{".to_string());
    push_str(&mut pt2, fmt_d(f64_of_i64(center)));
    push_str(&mut pt2, ", ".to_string());
    push_units(&mut pt2, &fmt_i(GRAFFLE_TOP_MARG + get_vml_y_size(f) * GRAFFLE_MULT));
    push_str(&mut pt2, "}</string>".to_string());
    push_line(out, &pt2);
    push_line(out, &ul_a("</array>".to_string()));

    // Player.java:314-346
    emit_player_line_tail(out);
    let mut tidl: U = ul_a("<integer>".to_string());
    push_units(&mut tidl, &fmt_i(text_id));
    push_units(&mut tidl, &ul_a("</integer>".to_string()));
    push_line(out, &tidl);
    push_line(out, &ul_a("</dict>".to_string()));
    push_line(out, &ul_a("</dict>".to_string())); // Player.java:350
    return Ok(());
}

// Message.java:282-678
fn draw_message_graffle(f: &mut Flow, idx: usize, out: &mut U, ctx: &mut RunContext) -> Result<(), Ex> {
    let mm: Message = f.messages[idx].clone();
    let line_pos: i64 = mm.line * GRAFFLE_MULT + GRAFFLE_TOP_MARG;
    let fcol: i64 = column_of(f, &mm.from_)?;
    let tcol: i64 = column_of(f, &mm.to)?;
    let fx: i64 = fcol * GRAFFLE_MULT;
    let tx: i64 = tcol * GRAFFLE_MULT;
    let startx: i64 = if fx < tx { fx } else { tx };

    let mut num_columns: i64 = 1;
    if mm.repeat_label {
        let d: i64 = fcol - tcol;
        let ad: i64 = if d < 0 { -d } else { d };
        num_columns = idiv(ad, get_column_pitch(f))?;
        ctx.err = format!("{}startx {} numColumns {}\n", ctx.err, from_units(&fmt_i(startx)), from_units(&fmt_i(num_columns)));
    }

    // honor message numbering (Message.java:297-307); the caption mutation
    // (DESIGN.md 3.8 #6) prepends "N. " to line 0 and survives to the comments
    if !is_note(&f.messages[idx]) || get_do_note_numbering(f) {
        let num: i64 = next_message_number(f);
        if num != -1 {
            f.messages[idx].my_number = num;
            let mut newcap: U = fmt_i(num);
            push_units(&mut newcap, &ul_a(". ".to_string()));
            let old0: U = arr_at(&f.messages[idx].caption, 0)?;
            push_units(&mut newcap, &old0);
            f.messages[idx].caption[0] = newcap;
        }
    }

    // group wrapper (Message.java:310-314)
    emit_group_head(out);

    let text_id: i64 = ctx.message_id; // Message.java:317: int textId = id
    let mut cur_x: i64 = startx; // Message.java:318 (int curX)
    let mut textx: f64 = f64_of_i64(cur_x); // Message.java:320 (double textx = curX)
    let ncap: i64 = f.messages[idx].caption.len() as i64;
    let texty: i64 = -1 * GRAFFLE_MULT * ncap + GRAFFLE_LINE_TXT_OFST + line_pos;

    let mut col: i64 = 0;
    while col < num_columns {
        textx = f64_of_i64(cur_x) + 10.0; // Message.java:326 (offset for arrow)

        push_line(out, &ul_a("<dict>".to_string()));
        push_line(out, &ul_a("<key>Bounds</key>".to_string()));
        let mut ln: U = ul_a("<string>{{".to_string());
        push_str(&mut ln, fmt_d(textx));
        push_str(&mut ln, ", ".to_string());
        push_str(&mut ln, fmt_d(f64_of_i64(texty)));
        push_str(&mut ln, " }, {0, 0}}</string>".to_string());
        push_line(out, &ln);
        push_line(out, &ul_a("<key>Class</key>".to_string()));
        push_line(out, &ul_a("<string>ShapedGraphic</string>".to_string()));
        push_line(out, &ul_a("<key>FitText</key>".to_string()));
        push_line(out, &ul_a("<string>YES</string>".to_string()));
        push_line(out, &ul_a("<key>ID</key>".to_string()));
        let mut idl: U = ul_a("<integer>".to_string());
        push_units(&mut idl, &fmt_i(ctx.message_id));
        push_units(&mut idl, &ul_a("</integer>".to_string()));
        push_line(out, &idl);
        ctx.message_id += 1; // Message.java:341
        push_line(out, &ul_a("<key>Shape</key>".to_string()));
        push_line(out, &ul_a("<string>Rectangle</string>".to_string()));

        if is_note(&f.messages[idx]) {
            emit_note_style(out); // Message.java:349-426
        } else {
            emit_text_style(out); // Message.java:431-473
        }

        // RTF label body (Message.java:478-499; empty caption throws, 3.8)
        push_line(out, &ul_a("<key>Text</key>".to_string()));
        push_line(out, &ul_a("<dict>".to_string()));
        push_line(out, &ul_a("<key>Align</key>".to_string()));
        push_line(out, &ul_a("<integer>0</integer>".to_string()));
        push_line(out, &ul_a("<key>Text</key>".to_string()));
        emit_rtf_head(out);
        push_str(out, r"\f0\fs24 \cf0 ".to_string());
        let clen: i64 = f.messages[idx].caption.len() as i64;
        let mut j: i64 = 0;
        while j < clen - 1 {
            push_units(out, &f.messages[idx].caption[j as usize]);
            out.push(0x5C);
            out.push(0x0A);
            j += 1;
        }
        let fin: U = arr_at(&f.messages[idx].caption, j)?;
        let mut tln: U = fin;
        push_str(&mut tln, "}</string>".to_string());
        push_line(out, &tln);
        push_line(out, &ul_a("<key>VerticalPad</key>".to_string()));
        push_line(out, &ul_a("<integer>1</integer>".to_string()));
        push_line(out, &ul_a("</dict>".to_string()));
        push_line(out, &ul_a("</dict>".to_string())); // Message.java:501

        cur_x += get_column_pitch(f) * GRAFFLE_MULT; // Message.java:503
        col += 1;
    }

    // the arrow line (Message.java:511-542)
    push_line(out, &ul_a("<dict>".to_string()));
    push_line(out, &ul_a("<key>Class</key>".to_string()));
    push_line(out, &ul_a("<string>LineGraphic</string>".to_string()));
    push_line(out, &ul_a("<key>ID</key>".to_string()));
    let mut idl3: U = ul_a("<integer>".to_string());
    push_units(&mut idl3, &fmt_i(ctx.message_id));
    push_units(&mut idl3, &ul_a("</integer>".to_string()));
    push_line(out, &idl3);
    ctx.message_id += 1;
    push_line(out, &ul_a("<key>Points</key>".to_string()));
    push_line(out, &ul_a("<array>".to_string()));
    if is_self(&f.messages[idx]) && !is_undirected(&f.messages[idx]) {
        // self-directed: five-point hook
        let mut p1: U = ul_a("<string>{".to_string());
        push_units(&mut p1, &fmt_i(fx));
        push_str(&mut p1, ", ".to_string());
        push_str(&mut p1, fmt_d(f64_of_i64(texty - 5)));
        push_str(&mut p1, "}</string>".to_string());
        push_line(out, &p1);
        let mut p2: U = ul_a("<string>{".to_string());
        push_units(&mut p2, &fmt_i(fx + 20));
        push_str(&mut p2, ", ".to_string());
        push_str(&mut p2, fmt_d(f64_of_i64(texty - 10)));
        push_str(&mut p2, "}</string>".to_string());
        push_line(out, &p2);
        let mut p3: U = ul_a("<string>{".to_string());
        push_units(&mut p3, &fmt_i(fx + 30));
        push_str(&mut p3, ", ".to_string());
        push_str(&mut p3, fmt_d((f64_of_i64(texty + line_pos)) / 2.0));
        push_str(&mut p3, "}</string>".to_string());
        push_line(out, &p3);
        let mut p4: U = ul_a("<string>{".to_string());
        push_units(&mut p4, &fmt_i(fx + 20));
        push_str(&mut p4, ", ".to_string());
        push_units(&mut p4, &fmt_i(line_pos + 10));
        push_str(&mut p4, "}</string>".to_string());
        push_line(out, &p4);
        let mut p5: U = ul_a("<string>{".to_string());
        push_units(&mut p5, &fmt_i(fx));
        push_str(&mut p5, ", ".to_string());
        push_units(&mut p5, &fmt_i(line_pos + 5));
        push_str(&mut p5, "}</string>".to_string());
        push_line(out, &p5);
    } else if is_note(&f.messages[idx]) {
        let mut p1: U = ul_a("<string>{".to_string());
        push_str(&mut p1, fmt_d(textx));
        push_str(&mut p1, ", ".to_string());
        push_str(&mut p1, fmt_d(f64_of_i64(texty)));
        push_str(&mut p1, "}</string>".to_string());
        push_line(out, &p1);
        let mut p2: U = ul_a("<string>{".to_string());
        push_units(&mut p2, &fmt_i(tx));
        push_str(&mut p2, ", ".to_string());
        push_units(&mut p2, &fmt_i(line_pos));
        push_str(&mut p2, "}</string>".to_string());
        push_line(out, &p2);
    } else {
        let mut p1: U = ul_a("<string>{".to_string());
        push_units(&mut p1, &fmt_i(fx));
        push_str(&mut p1, ", ".to_string());
        push_units(&mut p1, &fmt_i(line_pos));
        push_str(&mut p1, "}</string>".to_string());
        push_line(out, &p1);
        let mut p2: U = ul_a("<string>{".to_string());
        push_units(&mut p2, &fmt_i(tx));
        push_str(&mut p2, ", ".to_string());
        push_units(&mut p2, &fmt_i(line_pos));
        push_str(&mut p2, "}</string>".to_string());
        push_line(out, &p2);
    }
    push_line(out, &ul_a("</array>".to_string()));

    // line style (Message.java:544-654)
    push_line(out, &ul_a("<key>Style</key>".to_string()));
    push_line(out, &ul_a("<dict>".to_string()));
    if is_note(&f.messages[idx]) {
        emit_line_note_shadow(out); // Message.java:550-576
    }
    push_line(out, &ul_a("<key>stroke</key>".to_string()));
    push_line(out, &ul_a("<dict>".to_string()));
    push_line(out, &ul_a("<key>HeadArrow</key>".to_string()));
    if mm.arrow_to && !mm.arrow_from {
        push_line(out, &ul_a("<string>FilledArrow</string>".to_string()));
        push_line(out, &ul_a("<key>HeadScale</key>".to_string()));
        push_line(out, &ul_a("<real>0.75</real>".to_string()));
    } else {
        push_line(out, &ul_a("<string>0</string>".to_string()));
        push_line(out, &ul_a("<key>HeadScale</key>".to_string()));
        push_line(out, &ul_a("<real>0.5</real>".to_string()));
    }
    if !is_self(&f.messages[idx]) && is_undirected(&f.messages[idx]) {
        // ".." long dash vs "--" short dash (Message.java:598/602)
        push_line(out, &ul_a("<key>Pattern</key>".to_string()));
        if mm.repeat_label {
            push_line(out, &ul_a("<integer>2</integer>".to_string()));
        } else {
            push_line(out, &ul_a("<integer>1</integer>".to_string()));
        }
    }
    if is_note(&f.messages[idx]) {
        emit_line_note_style(out); // Message.java:611-628
    }
    push_line(out, &ul_a("<key>TailArrow</key>".to_string()));
    if mm.arrow_from && !mm.arrow_to {
        push_line(out, &ul_a("<string>FilledArrow</string>".to_string()));
        push_line(out, &ul_a("<key>TailScale</key>".to_string()));
        push_line(out, &ul_a("<real>0.75</real>".to_string()));
    } else {
        push_line(out, &ul_a("<string>0</string>".to_string()));
        push_line(out, &ul_a("<key>TailScale</key>".to_string()));
        push_line(out, &ul_a("<real>0.5</real>".to_string()));
    }
    if is_self(&f.messages[idx]) {
        push_line(out, &ul_a("<key>LineType</key>".to_string()));
        push_line(out, &ul_a("<integer>1</integer>".to_string())); // Message.java:648-649
    }
    push_line(out, &ul_a("</dict>".to_string()));
    push_line(out, &ul_a("</dict>".to_string()));

    // a note links its line to the text box (Message.java:657-668)
    if is_note(&f.messages[idx]) {
        push_line(out, &ul_a("<key>Tail</key>".to_string()));
        push_line(out, &ul_a("<dict>".to_string()));
        push_line(out, &ul_a("<key>ID</key>".to_string()));
        let mut tidl: U = ul_a("<integer>".to_string());
        push_units(&mut tidl, &fmt_i(text_id));
        push_units(&mut tidl, &ul_a("</integer>".to_string()));
        push_line(out, &tidl);
        push_line(out, &ul_a("</dict>".to_string()));
    }
    push_line(out, &ul_a("</dict>".to_string())); // Message.java:670 (close LineGraphic)

    // end group wrapper (Message.java:674-677)
    push_line(out, &ul_a("</array>".to_string()));
    push_line(out, &ul_a("<key>ID</key>".to_string()));
    let mut gidl: U = ul_a("<integer>".to_string());
    push_units(&mut gidl, &fmt_i(ctx.message_id));
    push_units(&mut gidl, &ul_a("</integer>".to_string()));
    push_line(out, &gidl);
    ctx.message_id += 1;
    push_line(out, &ul_a("</dict>".to_string()));
    return Ok(());
}

fn f64_of_i64(v: i64) -> f64 {
    return v as f64;
}

fn f64_of_usize(v: usize) -> f64 {
    return v as f64;
}
