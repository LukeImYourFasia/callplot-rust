// Flow: options, layout and geometry (DESIGN.md sections 3.3/3.4.0; flow.py).
// Flow.java:84-88 ctor, 90-116 option helpers, 118-168 geometry getters,
// 170-212 VML/text getters, 214-247 honorOpts, 249-277 layout, 279-288
// columnOf, 289-301 numbering.  All arithmetic is Java int.

use crate::errors::Ex;
use crate::errors::ErrorKind;
use crate::errors::exc;
use crate::jcompat::U;
use crate::jcompat::ul_a;
use crate::jcompat::u_eq;
use crate::jcompat::parse_java_int;
use crate::jcompat::CharBuf;
use crate::jcompat::from_units;
use crate::model::Player;
use crate::model::Message;

#[derive(Clone)]
pub struct OPair {
    pub k: U,
    pub v: U,
}

#[derive(Clone)]
pub struct Flow {
    pub players: Vec<Player>,
    pub messages: Vec<Message>,
    pub opts: Vec<OPair>,
    pub vml_x_size: i64,
    pub vml_y_size: i64,
    pub stagger: bool,
    pub start_line: i64,
    pub start_column: i64,
    pub line_pitch: i64,
    pub column_pitch: i64,
    pub num_players: i64,
    pub num_messages: i64,
    pub crunch: bool,
    pub links: bool,
    pub top_label: bool,
    pub bottom_label: bool,
    pub do_numbering: bool,
    pub do_note_numbering: bool,
    pub pic_x_size: U,
    pub pic_y_size: U,
    pub vml_style: U,
    pub vml_font: U,
    pub vml_text_size: U,
    pub vml_arrow_style: U,
    pub vml_dot_style: U,
    pub vml_box_style: U,
    pub vml_text_style: U,
    pub vml_box_text_style: U,
    pub vml_box_width_percentage: U,
    pub guy_caption_lines: i64,
    pub height: i64,
    pub msg_num: i64,
}

pub fn flow_new(players: Vec<Player>, messages: Vec<Message>, opts: Vec<OPair>) -> Flow {
    return Flow {
        players: players,
        messages: messages,
        opts: opts,
        vml_x_size: 0,
        vml_y_size: 0,
        stagger: false,
        start_line: 0,
        start_column: 0,
        line_pitch: 0,
        column_pitch: 0,
        num_players: 0,
        num_messages: 0,
        crunch: false,
        links: true,
        top_label: false,
        bottom_label: false,
        do_numbering: false,
        do_note_numbering: false,
        pic_x_size: ul_a(" ".to_string()),
        pic_y_size: ul_a(" ".to_string()),
        vml_style: ul_a(" ".to_string()),
        vml_font: ul_a("courier new".to_string()),
        vml_text_size: ul_a("9".to_string()),
        vml_arrow_style: ul_a("EndArrow='Open'".to_string()),
        vml_dot_style: ul_a("DashStyle='dot'".to_string()),
        vml_box_style: ul_a("fillcolor='red'".to_string()),
        vml_text_style: ul_a("-".to_string()),
        vml_box_text_style: ul_a("-".to_string()),
        vml_box_width_percentage: ul_a("90".to_string()),
        guy_caption_lines: 0,
        height: 0,
        msg_num: 0,
    };
}

// Map put: an existing key keeps its slot but takes the new value.
pub fn opt_put(opts: &mut Vec<OPair>, k: U, v: U) {
    let n = opts.len();
    let mut i: usize = 0;
    while i < n {
        if u_eq(&opts[i].k, &k) {
            opts[i].v = v;
            return;
        }
        i += 1;
    }
    opts.push(OPair { k: k, v: v });
}

pub fn opt_get(f: &Flow, name: &U) -> Option<U> {
    for e in &f.opts {
        if u_eq(&e.k, name) {
            return Some(e.v.clone());
        }
    }
    return None;
}

pub fn opts_has(f: &Flow, name: &U) -> bool {
    return opt_get(f, name) != None;
}

// Flow.java:90-97. A bad value throws NumberFormatError.
pub fn flow_get_int_value_or(f: &Flow, name: &U, dflt: i64) -> Result<i64, Ex> {
    match opt_get(f, name) {
        Some(val) => {
            return parse_java_int(&val);
        }
        None => {
            return Ok(dflt);
        }
    }
}

// Flow.java:99-116. Case-insensitive yes/true/on and no/false/off.
pub fn ascii_lower(s: U) -> U {
    let mut out: U = vec![];
    for c in s {
        if c >= 0x41 && c <= 0x5A {
            out.push(c + 0x20);
        } else {
            out.push(c);
        }
    }
    return out;
}

pub fn flow_get_boolean_value_or(f: &Flow, name: &U, dflt: bool) -> Result<bool, Ex> {
    match opt_get(f, name) {
        Some(val) => {
            let lc: U = ascii_lower(val);
            if u_eq(&lc, &ul_a("yes".to_string())) || u_eq(&lc, &ul_a("true".to_string())) || u_eq(&lc, &ul_a("on".to_string())) {
                return Ok(true);
            }
            if u_eq(&lc, &ul_a("no".to_string())) || u_eq(&lc, &ul_a("false".to_string())) || u_eq(&lc, &ul_a("off".to_string())) {
                return Ok(false);
            }
            return Err(exc(ErrorKind::RUNTIME, "Bad value for boolean option".to_string()));
        }
        None => {
            return Ok(dflt);
        }
    }
}

// Flow.java:214-247. stagger first: other defaults depend on it.
pub fn flow_honor_opts(f: &mut Flow) -> Result<(), Ex> {
    f.stagger = flow_get_boolean_value_or(f, &ul_a("stagger".to_string()), false)?;
    if f.stagger {
        f.start_line = flow_get_int_value_or(f, &ul_a("startLine".to_string()), 7)?;
    } else {
        f.start_line = flow_get_int_value_or(f, &ul_a("startLine".to_string()), 4)?;
    }
    f.start_column = flow_get_int_value_or(f, &ul_a("startColumn".to_string()), 10)?;
    f.line_pitch = flow_get_int_value_or(f, &ul_a("linePitch".to_string()), 4)?;
    if f.line_pitch < 2 {
        f.line_pitch = 2;
    }
    f.column_pitch = flow_get_int_value_or(f, &ul_a("columnPitch".to_string()), 10)?;
    f.num_players = f.players.len() as i64;
    f.num_messages = f.messages.len() as i64;
    f.crunch = flow_get_boolean_value_or(f, &ul_a("crunch".to_string()), false)?;
    f.links = flow_get_boolean_value_or(f, &ul_a("links".to_string()), true)?;
    f.top_label = flow_get_boolean_value_or(f, &ul_a("topLabel".to_string()), true)?;
    f.bottom_label = flow_get_boolean_value_or(f, &ul_a("bottomLabel".to_string()), false)?;
    f.do_numbering = flow_get_boolean_value_or(f, &ul_a("messageNumbering".to_string()), false)?;
    f.do_note_numbering = flow_get_boolean_value_or(f, &ul_a("noteNumbering".to_string()), false)?;

    // An option present with an empty value stays empty (a Java Map returns
    // "", not the default).
    match opt_get(f, &ul_a("picXSize".to_string())) {
        Some(v) => {
            f.pic_x_size = v;
        }
        None => {
            f.pic_x_size = ul_a(" ".to_string());
        }
    }
    match opt_get(f, &ul_a("picYSize".to_string())) {
        Some(v) => {
            f.pic_y_size = v;
        }
        None => {
            f.pic_y_size = ul_a(" ".to_string());
        }
    }
    match opt_get(f, &ul_a("vmlStyle".to_string())) {
        Some(v) => {
            f.vml_style = v;
        }
        None => {
            f.vml_style = ul_a(" ".to_string());
        }
    }
    match opt_get(f, &ul_a("vmlFont".to_string())) {
        Some(v) => {
            f.vml_font = v;
        }
        None => {
            f.vml_font = ul_a("courier new".to_string());
        }
    }
    match opt_get(f, &ul_a("vmlTextSize".to_string())) {
        Some(v) => {
            f.vml_text_size = v;
        }
        None => {
            f.vml_text_size = ul_a("9".to_string());
        }
    }
    match opt_get(f, &ul_a("vmlArrowStyle".to_string())) {
        Some(v) => {
            f.vml_arrow_style = v;
        }
        None => {
            f.vml_arrow_style = ul_a("EndArrow='Open'".to_string());
        }
    }
    match opt_get(f, &ul_a("vmlDottedStyle".to_string())) {
        Some(v) => {
            f.vml_dot_style = v;
        }
        None => {
            f.vml_dot_style = ul_a("DashStyle='dot'".to_string());
        }
    }
    match opt_get(f, &ul_a("vmlBoxStyle".to_string())) {
        Some(v) => {
            f.vml_box_style = v;
        }
        None => {
            f.vml_box_style = ul_a("fillcolor='red'".to_string());
        }
    }
    match opt_get(f, &ul_a("vmlTextStyle".to_string())) {
        Some(v) => {
            f.vml_text_style = v;
        }
        None => {
            f.vml_text_style = ul_a("-".to_string());
        }
    }
    match opt_get(f, &ul_a("vmlBoxTextStyle".to_string())) {
        Some(v) => {
            f.vml_box_text_style = v;
        }
        None => {
            f.vml_box_text_style = ul_a("-".to_string());
        }
    }
    match opt_get(f, &ul_a("vmlBoxWidthPercentage".to_string())) {
        Some(v) => {
            f.vml_box_width_percentage = v;
        }
        None => {
            if f.stagger {
                f.vml_box_width_percentage = ul_a("160".to_string());
            } else {
                f.vml_box_width_percentage = ul_a("90".to_string());
            }
        }
    }
    return Ok(());
}

// Flow.java:249-277
pub fn flow_layout(f: &mut Flow) -> Result<(), Ex> {
    flow_honor_opts(f)?;
    f.guy_caption_lines = 0;

    let mut col: i64 = f.start_column;
    let np: usize = f.players.len();
    let mut i: usize = 0;
    while i < np {
        f.players[i].column = col;
        col = col + f.column_pitch;
        if f.players[i].caption.len() as i64 > f.guy_caption_lines {
            f.guy_caption_lines = f.players[i].caption.len() as i64;
        }
        i += 1;
    }

    if f.top_label {
        f.start_line = f.start_line + f.guy_caption_lines;
    }

    let mut line: i64 = f.start_line;
    let nm: usize = f.messages.len();
    let mut j: usize = 0;
    while j < nm {
        let length: i64 = f.messages[j].caption.len() as i64;
        if length >= f.line_pitch {
            line = line + (length + 1 - f.line_pitch);
        }
        f.messages[j].line = line;
        line = line + f.line_pitch;
        j += 1;
    }
    f.height = line;
    return Ok(());
}

// Flow.java:118-124
pub fn get_start_line(f: &Flow) -> i64 {
    return f.start_line;
}

pub fn get_line_pitch(f: &Flow) -> i64 {
    return f.line_pitch;
}

// Flow.java:126-128
pub fn get_start_column(f: &Flow) -> i64 {
    return f.start_column;
}

// Flow.java:130-132
pub fn get_column_pitch(f: &Flow) -> i64 {
    return f.column_pitch;
}

// Flow.java:134-136
pub fn get_total_width(f: &Flow) -> i64 {
    return f.num_players * f.column_pitch + f.start_column + f.column_pitch;
}

// Flow.java:138-142
pub fn get_total_height(f: &Flow) -> i64 {
    return f.height;
}

// Flow.java:144-146
pub fn get_header_height(f: &Flow) -> i64 {
    return f.guy_caption_lines;
}

// Flow.java:148-156
pub fn label_line(f: &Flow, line: i64) -> i64 {
    if f.top_label && line < f.guy_caption_lines {
        return line;
    }
    if f.bottom_label && f.height - line < f.guy_caption_lines {
        return f.guy_caption_lines - 1 - (f.height - line);
    }
    return -1;
}

pub fn get_do_numbering(f: &Flow) -> bool {
    return f.do_numbering;
}

pub fn get_stagger(f: &Flow) -> bool {
    return f.stagger;
}

pub fn get_do_note_numbering(f: &Flow) -> bool {
    return f.do_note_numbering;
}

// Flow.java:170-184
pub fn get_vml_style(f: &Flow) -> U {
    return f.vml_style.clone();
}

pub fn get_vml_arrow_style(f: &Flow) -> U {
    return f.vml_arrow_style.clone();
}

pub fn get_vml_dotted_style(f: &Flow) -> U {
    return f.vml_dot_style.clone();
}

pub fn get_vml_box_style(f: &Flow) -> U {
    return f.vml_box_style.clone();
}

// Flow.java:186-196. Sentinel "-": build from vml_font/vml_text_size.
pub fn get_vml_text_style(f: &Flow) -> U {
    if u_eq(&f.vml_text_style, &ul_a("-".to_string())) {
        let mut out: U = ul_a("style='FONT-FAMILY:\"".to_string());
        let a: U = f.vml_font.clone();
        for c in &a {
            out.push(*c);
        }
        let b: U = ul_a("\";FONT-SIZE:".to_string());
        for c in &b {
            out.push(*c);
        }
        let c2: U = f.vml_text_size.clone();
        for c in &c2 {
            out.push(*c);
        }
        let d: U = ul_a("pt;V-TEXT-ALIGN:left'".to_string());
        for c in &d {
            out.push(*c);
        }
        return out;
    }
    return f.vml_text_style.clone();
}

// Flow.java:198-208 (same, center-aligned)
pub fn get_vml_box_text_style(f: &Flow) -> U {
    if u_eq(&f.vml_box_text_style, &ul_a("-".to_string())) {
        let mut out: U = ul_a("style='FONT-FAMILY:\"".to_string());
        let a: U = f.vml_font.clone();
        for c in &a {
            out.push(*c);
        }
        let b: U = ul_a("\";FONT-SIZE:".to_string());
        for c in &b {
            out.push(*c);
        }
        let c2: U = f.vml_text_size.clone();
        for c in &c2 {
            out.push(*c);
        }
        let d: U = ul_a("pt;V-TEXT-ALIGN:center'".to_string());
        for c in &d {
            out.push(*c);
        }
        return out;
    }
    return f.vml_box_text_style.clone();
}

// Flow.java:210-212; parsed with parseJavaInt at draw time
pub fn get_vml_box_width_percentage(f: &Flow) -> U {
    return f.vml_box_width_percentage.clone();
}

// Flow.java:424-430
pub fn get_vml_x_size(f: &Flow) -> i64 {
    return f.vml_x_size;
}

pub fn get_vml_y_size(f: &Flow) -> i64 {
    return f.vml_y_size;
}

pub fn set_vml_x_size(f: &mut Flow, v: i64) {
    f.vml_x_size = v;
}

pub fn set_vml_y_size(f: &mut Flow, v: i64) {
    f.vml_y_size = v;
}

// Flow.java:432-447: crunch drops lines holding only spaces and bars.
pub fn should_print(f: &Flow, buf: &CharBuf) -> bool {
    if !f.crunch {
        return true;
    }
    for ch in &buf.a {
        if *ch != 0x20 && *ch != 0x7C {
            return true;
        }
    }
    return false;
}

// Flow.java:303-305: the method exists but does nothing (section 3.8 #2).
pub fn xml_escape(s: &U) -> U {
    return s.clone();
}

// Flow.java:279-287: first match wins.
pub fn column_of(f: &Flow, name: &U) -> Result<i64, Ex> {
    for p in &f.players {
        if u_eq(&p.name, name) {
            return Ok(p.column);
        }
    }
    return Err(exc(ErrorKind::ILLEGAL_ARGUMENT, format!("no player named {}", from_units(name))));
}

// Flow.java:289-301: msg_num resets per render, not per run (section 3.6).
pub fn start_message_numbers(f: &mut Flow) {
    f.msg_num = 1;
}

pub fn next_message_number(f: &mut Flow) -> i64 {
    if f.do_numbering {
        let num: i64 = f.msg_num;
        f.msg_num = f.msg_num + 1;
        return num;
    }
    return -1;
}
