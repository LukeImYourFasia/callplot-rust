// Diagram geometry for the -html SVG (DESIGN.md 3.12.4; render/html/layout.js
// + renderers/html/layout.py).  Text width is ESTIMATED (never measured), so
// the output is machine-independent.  Geometry svgOn draws and layout widths
// must agree on; the constants below are the decisions where 3.12.4 is open.

use crate::ir::Model;
use crate::ir::Kind;
use crate::jcompat::U;
use crate::jcompat::u_len;
use crate::jsnum::js_round;

pub const CW: f64 = 7.5; // one caption character (12.5px mono font)
pub const LABW: f64 = 7.6; // one label character (13px UI font)
pub const LH: f64 = 16.0; // one text line
pub const M: f64 = 28.0; // left and right margin
pub const TOP: f64 = 16.0; // space above the participant boxes
pub const PITCH_MIN: f64 = 150.0;
pub const PITCH_MAX: f64 = 300.0;

pub const NOTE_DX: f64 = 12.0; // note box left edge, right of the lifeline
pub const NOTE_PAD: f64 = 16.0; // note box width = caption width + padding
pub const NOTE_DY: f64 = 18.0; // note box top inside its event band
pub const SELF_W: f64 = 34.0; // self loop width
pub const SELF_H: f64 = 18.0; // self loop height
pub const SELF_DX: f64 = 40.0; // self caption left edge, right of the loop
pub const HL_DY: f64 = 8.0; // .hl rect starts 8px above the band top
pub const HL_BOT: f64 = 12.0; // SVG height = last event bottom + 12

pub struct Band {
    pub top: f64,
    pub bh: f64,
}

pub struct HtmlLayoutData {
    pub pitch: f64,
    pub box_h: f64,
    pub top0: f64,
    pub x: Vec<f64>,
    pub n: i64,
    pub w: f64,
    pub h: f64,
    pub bands: Vec<Band>,
    pub cap_lens: Vec<i64>,
}

// longest line length in UTF-16 units (like JS length)
pub fn longest_line(lines: &Vec<U>) -> i64 {
    let mut length: i64 = 0;
    let n: usize = lines.len();
    let mut i: usize = 0;
    while i < n {
        let l: i64 = u_len(&lines[i]);
        if l > length {
            length = l;
        }
        i += 1;
    }
    return length;
}

// estimated caption width: longest line times CW, plus badge room
pub fn cap_w(lines: &Vec<U>, numbered: bool) -> f64 {
    return longest_line(lines) as f64 * CW + (if numbered { 28.0 } else { 6.0 });
}

pub fn compute_layout(model: &Model) -> HtmlLayoutData {
    // steps 2-3: pitch starts at 150, raised by the widest label and by
    // every event spanning 2 or more lifelines
    let mut pitch: f64 = PITCH_MIN;
    let mut label_lines: i64 = 1;
    let np: usize = model.participants.len();
    let mut i: usize = 0;
    while i < np {
        let length: i64 = longest_line(&model.participants[i].label);
        if model.participants[i].label.len() as i64 > label_lines {
            label_lines = model.participants[i].label.len() as i64;
        }
        if length as f64 * LABW + 36.0 > pitch {
            pitch = length as f64 * LABW + 36.0;
        }
        i += 1;
    }
    let ne: usize = model.events.len();
    let mut i2: usize = 0;
    while i2 < ne {
        let mut span: i64 = model.events[i2].from_ - model.events[i2].to;
        if span < 0 {
            span = 0 - span;
        }
        if span >= 2 {
            let c: f64 = cap_w(&model.events[i2].caption, model.events[i2].num.is_some());
            let cand: f64 = c / (span as f64) + 24.0;
            if cand > pitch {
                pitch = cand;
            }
        }
        i2 += 1;
    }

    // step 4: round (JS Math.round), then cap at 300
    pitch = js_round(pitch);
    if pitch > PITCH_MAX {
        pitch = PITCH_MAX;
    }

    // step 5: lifelines are centered in their pitch slots
    let n: i64 = model.participants.len() as i64;
    let mut x: Vec<f64> = vec![];
    let mut i3: i64 = 0;
    while i3 < n {
        x.push(M + pitch / 2.0 + (i3 as f64) * pitch);
        i3 += 1;
    }

    // steps 6-8: band stacking; the first event starts below the boxes
    let box_h: f64 = 14.0 + (label_lines as f64) * LH;
    let top0: f64 = TOP + box_h + 30.0;
    let mut bands: Vec<Band> = vec![];
    let mut cap_lens: Vec<i64> = vec![];
    let mut top: f64 = top0;
    let mut hmax: f64 = top0;
    let mut right_edge: f64 = 2.0 * M + (n as f64) * pitch;
    let ne2: usize = model.events.len();
    let mut k: usize = 0;
    while k < ne2 {
        let length: i64 = longest_line(&model.events[k].caption);
        cap_lens.push(length);
        let lines: i64 = model.events[k].caption.len() as i64;
        let mut h: f64 = (lines as f64) * LH + 30.0;
        if model.events[k].kind == Kind::SELF {
            h = (lines as f64) * LH + 38.0;
        }
        bands.push(Band { top: top, bh: h });
        hmax = top + h;
        top = top + h;
        if model.events[k].kind == Kind::NOTE || model.events[k].kind == Kind::SELF {
            let mut edge: f64 = 0.0;
            if model.events[k].kind == Kind::NOTE {
                edge = x[model.events[k].from_ as usize] + NOTE_DX + (length as f64) * CW + NOTE_PAD;
            } else {
                edge = x[model.events[k].from_ as usize] + SELF_DX + (length as f64) * CW;
            }
            if edge + M > right_edge {
                right_edge = edge + M;
            }
        }
        k += 1;
    }

    return HtmlLayoutData {
        pitch: pitch,
        box_h: box_h,
        top0: top0,
        x: x,
        n: n,
        w: right_edge,
        h: hmax + HL_BOT,
        bands: bands,
        cap_lens: cap_lens,
    };
}
