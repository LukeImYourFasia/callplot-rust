// Section 3.14: callplot-prep, a port of lib/prep.awk (mawk 1.3.4 oracle);
// DESIGN-RUST.md section 3.8.  BYTES in, BYTES out: awk works on bytes, so
// this never decodes, and a trace with bad UTF-8 still converts exactly.
// One `if` block per awk rule in file order (DESIGN-RUST 3.8 rule 2): rules
// 8-15 print and FALL THROUGH to rule 16 (quirk 4 of DESIGN.md 3.14.3), so
// they are separate `if`s, never an `else if` chain.

use crate::jcompat::str_bytes;
use crate::jcompat::bytes_str;

pub struct PrepOptions {
    pub links: bool,
}

pub fn prep_new() -> PrepOptions {
    return PrepOptions { links: false };
}

// an ASCII literal as bytes
pub fn bb(lit: &str) -> Vec<u8> {
    return str_bytes(lit.to_string());
}

pub struct PBytes {
    pub k: Vec<u8>,
    pub v: Vec<u8>,
}

pub fn pget(list: &Vec<PBytes>, key: &Vec<u8>) -> Option<Vec<u8>> {
    let n: usize = list.len();
    let mut i: usize = 0;
    while i < n {
        if list[i].k == *key {
            return Some(list[i].v.clone());
        }
        i += 1;
    }
    return None;
}

pub fn phas(list: &Vec<PBytes>, key: &Vec<u8>) -> bool {
    return pget(list, key) != None;
}

pub fn pput(list: &mut Vec<PBytes>, key: &Vec<u8>, value: &Vec<u8>) {
    let n: usize = list.len();
    let mut i: usize = 0;
    while i < n {
        if list[i].k == *key {
            list[i].v = value.clone();
            return;
        }
        i += 1;
    }
    list.push(PBytes { k: key.clone(), v: value.clone() });
}

// the pieces of one output line, concatenated
pub fn cat_all(parts: &Vec<Vec<u8>>) -> Vec<u8> {
    let out: Vec<u8> = vec![];
    let mut out: Vec<u8> = out;
    let n: usize = parts.len();
    let mut i: usize = 0;
    while i < n {
        let one: &Vec<u8> = &parts[i];
        let m: usize = one.len();
        let mut j: usize = 0;
        while j < m {
            out.push(one[j]);
            j += 1;
        }
        i += 1;
    }
    return out;
}

pub fn push_line(out: &mut Vec<Vec<u8>>, parts: Vec<Vec<u8>>) {
    out.push(cat_all(&parts));
}

// ---- awk number printing (DESIGN-RUST 3.8 rule 7) --------------------------

pub fn find_byte(b: &Vec<u8>, c: u8) -> usize {
    let n: usize = b.len();
    let mut i: usize = 0;
    while i < n {
        if b[i] == c {
            return i;
        }
        i += 1;
    }
    return b.len();
}

pub fn has_byte(b: &Vec<u8>, c: u8) -> bool {
    return find_byte(b, c) < b.len();
}

// whole numbers print without a decimal point; OFMT %.6g otherwise
pub fn awk_num(x: f64) -> Vec<u8> {
    if x.fract() == 0.0 {
        if x == 0.0 {
            return bb("0");
        }
        return str_bytes(format!("{:.0}", x));
    }
    let sci: String = format!("{:.5e}", x);
    let sb: Vec<u8> = str_bytes(sci);
    let epos: usize = find_byte(&sb, 0x65);
    let mut mant_b: Vec<u8> = vec![];
    let mut i: usize = 0;
    while i < epos {
        mant_b.push(sb[i]);
        i += 1;
    }
    let mut esign: i64 = 1;
    let mut j: usize = epos + 1;
    if j < sb.len() && sb[j] == 0x2D {
        esign = -1;
        j += 1;
    } else if j < sb.len() && sb[j] == 0x2B {
        j += 1;
    }
    let mut eexp: i64 = 0;
    while j < sb.len() {
        eexp = eexp * 10 + (sb[j] as i64 - 0x30);
        j += 1;
    }
    let e: i64 = esign * eexp;
    if e >= -4 && e < 6 {
        let s2: String = format!("{:.1$}", x, (5 - e) as usize);
        let mut out: Vec<u8> = str_bytes(s2);
        if has_byte(&out, 0x2E) {
            while out.len() > 0 && out[out.len() - 1] == 0x30 {
                out.pop();
            }
            if out.len() > 0 && out[out.len() - 1] == 0x2E {
                out.pop();
            }
        }
        return out;
    }
    let mut man: Vec<u8> = mant_b;
    if has_byte(&man, 0x2E) {
        while man.len() > 0 && man[man.len() - 1] == 0x30 {
            man.pop();
        }
        if man.len() > 0 && man[man.len() - 1] == 0x2E {
            man.pop();
        }
    }
    let mut out: Vec<u8> = man;
    out.push(0x65);
    if e < 0 {
        out.push(0x2D);
    } else {
        out.push(0x2B);
    }
    let ee: i64 = if e < 0 { 0 - e } else { e };
    if ee < 10 {
        out.push(0x30);
    }
    let digits_b: Vec<u8> = str_bytes(format!("{}", ee));
    let dn: usize = digits_b.len();
    let mut q: usize = 0;
    while q < dn {
        out.push(digits_b[q]);
        q += 1;
    }
    return out;
}

// awk string -> number: the longest leading number, 0 when there is none.
// A bare sign or no digits gives 0 (the awk quirk).
pub fn awk_strtod(b: &Vec<u8>) -> f64 {
    let n: usize = b.len();
    let mut i: usize = 0;
    while i < n && (b[i] == 0x20 || b[i] == 0x09) {
        i += 1;
    }
    let start: usize = i;
    if i < n && (b[i] == 0x2B || b[i] == 0x2D) {
        i += 1;
    }
    let digits_start: usize = i;
    while i < n && b[i] >= 0x30 && b[i] <= 0x39 {
        i += 1;
    }
    if i < n && b[i] == 0x2E {
        i += 1;
        while i < n && b[i] >= 0x30 && b[i] <= 0x39 {
            i += 1;
        }
    }
    if i > digits_start && i < n && (b[i] == 0x65 || b[i] == 0x45) {
        let save: usize = i;
        let mut k: usize = i + 1;
        if k < n && (b[k] == 0x2B || b[k] == 0x2D) {
            k += 1;
        }
        let exp_start: usize = k;
        while k < n && b[k] >= 0x30 && b[k] <= 0x39 {
            k += 1;
        }
        if k > exp_start {
            i = k;
        } else {
            i = save;
        }
    }
    if i <= digits_start {
        return 0.0;
    }
    let mut num: Vec<u8> = vec![];
    let mut m: usize = start;
    while m < i {
        num.push(b[m]);
        m += 1;
    }
    if num.len() > 0 && num[num.len() - 1] == 0x2E {
        num.push(0x30); // "1." is a float in awk/Python, not in Rust
    }
    let s: String = bytes_str(num);
    let parsed = s.parse();
    match parsed {
        Ok(v) => { return v; }
        Err(_) => { return 0.0; }
    }
}

// ---- toTime (DESIGN.md 3.14.2 time lines) ---------------------------------

// replace every '/' ',' ' ' '.' with ':', split on ':', then
// arr[n]/1000 + arr[n-1] + 60*arr[n-2] + 3600*arr[n-3]
pub fn to_time(ts: &Vec<u8>) -> f64 {
    let mut rep: Vec<u8> = vec![];
    let n: usize = ts.len();
    let mut i: usize = 0;
    while i < n {
        let c: u8 = ts[i];
        if c == 0x2F || c == 0x2C || c == 0x20 || c == 0x2E {
            rep.push(0x3A);
        } else {
            rep.push(c);
        }
        i += 1;
    }
    let parts: Vec<Vec<u8>> = split_byte(&rep, 0x3A);
    let ln: i64 = parts.len() as i64;
    return at_num(&parts, ln) / 1000.0 + at_num(&parts, ln - 1)
        + 60.0 * at_num(&parts, ln - 2) + 3600.0 * at_num(&parts, ln - 3);
}

// awk arr[i], 1-based; out of range and non-numeric parts give 0
pub fn at_num(parts: &Vec<Vec<u8>>, i: i64) -> f64 {
    if i < 1 || i > parts.len() as i64 {
        return 0.0;
    }
    return awk_strtod(&parts[(i - 1) as usize]);
}

// ---- awk fields and line surgery ------------------------------------------

// DESIGN-RUST 3.8 rule 3: runs of 0x20/0x09/0x0A only.  str::split_whitespace
// is banned (it also cuts 0x0B and NBSP).  Empty items drop out; awk makes
// missing fields the empty string, so pad to four.
pub fn awk_fields(line: &Vec<u8>) -> Vec<Vec<u8>> {
    let mut out: Vec<Vec<u8>> = vec![];
    let n: usize = line.len();
    let mut i: usize = 0;
    while i < n {
        if line[i] == 0x20 || line[i] == 0x09 || line[i] == 0x0A {
            i += 1;
        } else {
            let start: usize = i;
            while i < n && line[i] != 0x20 && line[i] != 0x09 && line[i] != 0x0A {
                i += 1;
            }
            let mut one: Vec<u8> = vec![];
            let mut j: usize = start;
            while j < i {
                one.push(line[j]);
                j += 1;
            }
            out.push(one);
        }
    }
    while out.len() < 4 {
        out.push(vec![]);
    }
    return out;
}

// awk record split on one byte; a trailing separator adds no empty record
pub fn split_byte(b: &Vec<u8>, sep: u8) -> Vec<Vec<u8>> {
    let mut out: Vec<Vec<u8>> = vec![];
    let n: usize = b.len();
    let mut i: usize = 0;
    while i < n {
        let start: usize = i;
        while i < n && b[i] != sep {
            i += 1;
        }
        let mut one: Vec<u8> = vec![];
        let mut j: usize = start;
        while j < i {
            one.push(b[j]);
            j += 1;
        }
        out.push(one);
        i = i + 1;
    }
    return out;
}

// awk sub("\r",""): the FIRST match only (DESIGN-RUST 3.8 rule 6)
pub fn sub_first_cr(line: &Vec<u8>) -> Vec<u8> {
    let mut out: Vec<u8> = vec![];
    let n: usize = line.len();
    let cr: usize = find_byte(line, 0x0D);
    let mut i: usize = 0;
    while i < n {
        if i != cr {
            out.push(line[i]);
        }
        i += 1;
    }
    return out;
}

// the awk gsub triple: & first, so the entities for < and > are not re-escaped
pub fn esc0(cur: &Vec<u8>) -> Vec<u8> {
    let mut out: Vec<u8> = vec![];
    let n: usize = cur.len();
    let mut i: usize = 0;
    while i < n {
        let c: u8 = cur[i];
        if c == 0x26 {
            let amp: Vec<u8> = bb("&amp;");
            let m: usize = amp.len();
            let mut j: usize = 0;
            while j < m {
                out.push(amp[j]);
                j += 1;
            }
        } else if c == 0x3C {
            let lt2: Vec<u8> = bb("&lt;");
            let m: usize = lt2.len();
            let mut j: usize = 0;
            while j < m {
                out.push(lt2[j]);
                j += 1;
            }
        } else if c == 0x3E {
            let gt2: Vec<u8> = bb("&gt;");
            let m: usize = gt2.len();
            let mut j: usize = 0;
            while j < m {
                out.push(gt2[j]);
                j += 1;
            }
        } else {
            out.push(c);
        }
        i += 1;
    }
    return out;
}

pub fn starts_with(line: &Vec<u8>, pre: &Vec<u8>) -> bool {
    if line.len() < pre.len() {
        return false;
    }
    let n: usize = pre.len();
    let mut i: usize = 0;
    while i < n {
        if line[i] != pre[i] {
            return false;
        }
        i += 1;
    }
    return true;
}

pub fn byte_at(cur: &Vec<u8>, i: i64) -> u8 {
    if i < 0 || i as usize >= cur.len() {
        return 0;
    }
    return cur[i as usize];
}

pub fn is_digit(cur: &Vec<u8>, i: i64) -> bool {
    let c: u8 = byte_at(cur, i);
    return c >= 0x30 && c <= 0x39;
}

// ^Date dd/dd/dddd
pub fn date4(cur: &Vec<u8>) -> bool {
    return starts_with(cur, &bb("Date ")) && is_digit(cur, 5) && is_digit(cur, 6)
        && byte_at(cur, 7) == 0x2F && is_digit(cur, 8) && is_digit(cur, 9)
        && byte_at(cur, 10) == 0x2F && is_digit(cur, 11) && is_digit(cur, 12)
        && is_digit(cur, 13) && is_digit(cur, 14);
}

// ^dd/dd/dddd
pub fn date5(cur: &Vec<u8>) -> bool {
    return is_digit(cur, 0) && is_digit(cur, 1) && byte_at(cur, 2) == 0x2F
        && is_digit(cur, 3) && is_digit(cur, 4) && byte_at(cur, 5) == 0x2F
        && is_digit(cur, 6) && is_digit(cur, 7) && is_digit(cur, 8) && is_digit(cur, 9);
}

// HTTP/1.. at `at_`: the two dots match any byte except a newline (awk)
pub fn http_at(cur: &Vec<u8>, at_: i64) -> bool {
    if (cur.len() as i64) < at_ + 9 {
        return false;
    }
    let pre: Vec<u8> = bb("HTTP/1");
    let mut i: i64 = 0;
    while i < 6 {
        if byte_at(cur, at_ + i) != pre[i as usize] {
            return false;
        }
        i += 1;
    }
    return byte_at(cur, at_ + 6) != 0x0A && byte_at(cur, at_ + 7) != 0x0A
        && byte_at(cur, at_ + 8) == 0x20;
}

pub fn method_start(word: &Vec<u8>) -> bool {
    let ms: Vec<String> = vec!["MESSAGE".to_string(), "INVITE".to_string(), "BYE".to_string(),
        "CANCEL".to_string(), "REGISTER".to_string(), "INFO".to_string(), "NOTIFY".to_string(),
        "SUBSCRIBE".to_string(), "PUBLISH".to_string()];
    let n: usize = ms.len();
    let mut i: usize = 0;
    while i < n {
        let mb: Vec<u8> = str_bytes(ms[i].clone());
        if starts_with(word, &mb) {
            return true;
        }
        i += 1;
    }
    return false;
}

pub fn verb_start(word: &Vec<u8>) -> bool {
    let vs: Vec<String> = vec!["GET".to_string(), "POST".to_string()];
    let n: usize = vs.len();
    let mut i: usize = 0;
    while i < n {
        let vb: Vec<u8> = str_bytes(vs[i].clone());
        if starts_with(word, &vb) {
            return true;
        }
        i += 1;
    }
    return false;
}

// the letter labels a..zA..z; past 52 the label is empty (quirk 2 of 3.14.3)
pub fn letter_at(letidx: i64) -> Vec<u8> {
    if letidx >= 0 && letidx < 26 {
        return vec![0x61 + letidx as u8];
    }
    if letidx >= 26 && letidx < 52 {
        return vec![0x41 + (letidx - 26) as u8];
    }
    return vec![];
}

// the state the rules carry between lines
pub struct PrepState {
    pub letidx: i64,
    pub alias: Vec<PBytes>,
    pub guys: Vec<PBytes>,
    pub t0: Vec<u8>,
    pub t: Vec<u8>,
    pub lt: Vec<u8>,
    pub s: Vec<u8>,
    pub d: Vec<u8>,
    pub m_date: Vec<u8>,
    pub m_src: Vec<u8>,
    pub m_dest: Vec<u8>,
}

pub fn prep_state_new() -> PrepState {
    return PrepState {
        letidx: 0,
        alias: vec![],
        guys: vec![],
        t0: vec![],
        t: vec![],
        lt: vec![],
        s: vec![],
        d: vec![],
        m_date: vec![],
        m_src: vec![],
        m_dest: vec![],
    };
}

// rules 4/5 action: the t0/lt/t ladder, then the gsub triple; t takes $0
// BEFORE it is escaped, as the awk action order does
pub fn set_date(st: &mut PrepState, cur: &Vec<u8>) {
    st.lt = st.t.clone();
    st.t = cur.clone();
    if st.t0.len() == 0 {
        st.t0 = st.t.clone();
    }
    let esc: Vec<u8> = esc0(cur);
    st.m_date = cat_all(&vec![bb("!!"), esc]);
}

// rules 6/7 action: resolve $3 through ALIAS/GUYS, minting a letter label
pub fn set_addr(out: &mut Vec<Vec<u8>>, debug: &mut Vec<Vec<u8>>, st: &mut PrepState,
                field: &Vec<u8>, cur: &Vec<u8>) -> (Vec<u8>, Vec<u8>) {
    let raw: Vec<u8> = sub_first_cr(field);
    let got: Option<Vec<u8>> = pget(&st.alias, &raw);
    let v: Vec<u8> = match got {
        Some(x) => { x.clone() }
        None => { raw.clone() }
    };
    return finish_addr(out, debug, st, &v, cur);
}

pub fn finish_addr(out: &mut Vec<Vec<u8>>, debug: &mut Vec<Vec<u8>>, st: &mut PrepState,
                   v: &Vec<u8>, cur: &Vec<u8>) -> (Vec<u8>, Vec<u8>) {
    if phas(&st.guys, &v) == false {
        let dd: Vec<u8> = cat_all(&vec![bb("["), v.clone(), bb("] not in guys")]);
        debug.push(dd);
        let letr: Vec<u8> = letter_at(st.letidx);
        st.letidx = st.letidx + 1;
        push_line(out, vec![bb("guy/"), letr.clone(), bb("/"), v.clone()]);
        pput(&mut st.guys, &v, &letr);
    }
    let label: Vec<u8> = match pget(&st.guys, &v) {
        Some(x) => { x.clone() }
        None => { vec![] }
    };
    let esc: Vec<u8> = esc0(cur);
    return (label, cat_all(&vec![bb("!!"), esc]));
}

// the two time lines; the awk bt != "" guard never fires (quirk 1: a number
// never string-equals "", so with lt empty the "from last" value is the
// whole time of day)
pub fn tdiffs(out: &mut Vec<Vec<u8>>, st: &PrepState) {
    let at: f64 = to_time(&st.t);
    let b_init: f64 = to_time(&st.t0);
    let b_last: f64 = to_time(&st.lt);
    push_line(out, vec![bb("!!    "), awk_num(at - b_init), bb(" from initial")]);
    push_line(out, vec![bb("!!    "), awk_num(at - b_last), bb(" from last")]);
}

// rules 8-11: the request line and its four follow-up lines
pub fn emit_full(out: &mut Vec<Vec<u8>>, st: &PrepState, word: &Vec<u8>) {
    push_line(out, vec![st.s.clone(), bb("->"), st.d.clone(), bb("/"), word.clone()]);
    push_line(out, vec![st.m_date.clone()]);
    tdiffs(out, st);
    if st.s.len() > 1 {
        push_line(out, vec![st.m_src.clone(), bb(" ("), st.s.clone(), bb(")")]);
    } else {
        push_line(out, vec![st.m_src.clone()]);
    }
    push_line(out, vec![st.m_dest.clone(), bb(" ("), st.d.clone(), bb(")")]);
}

// rules 12-15: the hidden-message variant prints one line only
pub fn emit_dash(out: &mut Vec<Vec<u8>>, st: &PrepState, word: &Vec<u8>) {
    push_line(out, vec![bb("-"), st.s.clone(), bb("->"), st.d.clone(), bb("/"), word.clone()]);
}

// cur[len(f0) + 1:]: replace(f0 " ", "") with the rule test guaranteeing a
// match at 0, so this removes exactly the first field plus one blank
pub fn tail_from(cur: &Vec<u8>, at: usize) -> Vec<u8> {
    let mut out: Vec<u8> = vec![];
    let n: usize = cur.len();
    let mut i: usize = at;
    while i < n {
        out.push(cur[i]);
        i += 1;
    }
    return out;
}

// one awk pass over the record lines
pub fn run_lines(out: &mut Vec<Vec<u8>>, debug: &mut Vec<Vec<u8>>, st: &mut PrepState,
                 lines: &Vec<Vec<u8>>) {
    let n: usize = lines.len();
    let mut k: usize = 0;
    while k < n {
        // remove the first \r before the rules run (DESIGN-RUST 3.8 rule 5)
        let cur0: Vec<u8> = sub_first_cr(&lines[k]);
        let cur: &Vec<u8> = &cur0;
        let f: Vec<Vec<u8>> = awk_fields(cur);

        // rule 1: $1 == "@"
        if f[0].len() == 1 && f[0][0] == 0x40 {
            let a2: Vec<u8> = f[1].clone();
            let a3: Vec<u8> = sub_first_cr(&f[2]);
            if phas(&st.guys, &a3) == false {
                push_line(out, vec![bb("guy/"), a3.clone(), bb("/"), a3.clone()]);
                pput(&mut st.guys, &a3, &a3);
                debug.push(cat_all(&vec![bb("["), a3.clone(), bb("] going into guys")]));
            }
            pput(&mut st.alias, &a2, &a3);
            k = k + 1;
            continue;
        }

        // rule 2: starts with @ (callplot commands pass through)
        if byte_at(cur, 0) == 0x40 {
            out.push(tail_from(cur, 1));
            k = k + 1;
            continue;
        }

        // rule 3: starts with ----
        if starts_with(cur, &bb("----")) {
            k = k + 1;
            continue;
        }

        // rules 4-5
        if date4(cur) {
            set_date(st, cur);
            k = k + 1;
            continue;
        }
        if date5(cur) {
            set_date(st, cur);
            k = k + 1;
            continue;
        }

        // rules 6-7
        if starts_with(cur, &bb("Source Address")) {
            let pair: (Vec<u8>, Vec<u8>) = set_addr(out, debug, st, &f[2], cur);
            st.s = pair.0;
            st.m_src = pair.1;
            k = k + 1;
            continue;
        }
        if starts_with(cur, &bb("Destination Address")) {
            let pair: (Vec<u8>, Vec<u8>) = set_addr(out, debug, st, &f[2], cur);
            st.d = pair.0;
            st.m_dest = pair.1;
            k = k + 1;
            continue;
        }

        // rules 8-11 (fall through to 16)
        if method_start(&f[0]) || verb_start(&f[0]) {
            emit_full(out, st, &f[0]);
        } else if starts_with(cur, &bb("SIP/2.0 ")) || http_at(cur, 0) {
            emit_full(out, st, &tail_from(cur, f[0].len() + 1));
        }

        // rules 12-15 (fall through to 16)
        if byte_at(&f[0], 0) == 0x2D {
            let tailf: Vec<u8> = tail_from(&f[0], 1);
            if method_start(&tailf) || verb_start(&tailf) {
                emit_dash(out, st, &tailf);
            }
        }
        if starts_with(cur, &bb("-SIP/2.0 ")) || http_at(cur, 1) {
            emit_dash(out, st, &tail_from(cur, f[0].len() + 1));
        }

        // rule 16: every line that got here
        let esc: Vec<u8> = esc0(cur);
        out.push(cat_all(&vec![bb("!"), esc]));
        k = k + 1;
    }
}

// join the lines with newlines; a non-empty output ends with one
pub fn finish_out(out: &Vec<Vec<u8>>) -> Vec<u8> {
    let mut res: Vec<u8> = vec![];
    let n: usize = out.len();
    let mut i: usize = 0;
    while i < n {
        if i > 0 {
            res.push(0x0A);
        }
        let one: &Vec<u8> = &out[i];
        let m: usize = one.len();
        let mut j: usize = 0;
        while j < m {
            res.push(one[j]);
            j += 1;
        }
        i += 1;
    }
    if n > 0 {
        res.push(0x0A);
    }
    return res;
}

// the public core calls: bytes in, bytes out, no I/O (DESIGN-RUST 3.8 rule 9:
// the debug lines are returned; the command writes them only to --debug-file)
pub fn trace_lines(text: &Vec<u8>, links: bool) -> (Vec<u8>, Vec<Vec<u8>>) {
    let out: Vec<Vec<u8>> = vec![];
    let mut out: Vec<Vec<u8>> = out;
    let debug: Vec<Vec<u8>> = vec![];
    let mut debug: Vec<Vec<u8>> = debug;
    let mut st: PrepState = prep_state_new();
    let raw_lines: Vec<Vec<u8>> = split_byte(text, 0x0A);
    push_line(&mut out, vec![bb("opt/messageNumbering/on")]);
    push_line(&mut out, vec![bb("opt/xml/on")]);
    push_line(&mut out, vec![bb("opt/crunch/on")]);
    if links {
        push_line(&mut out, vec![bb("opt/links/on")]);
    } else {
        push_line(&mut out, vec![bb("opt/links/off")]);
    }
    run_lines(&mut out, &mut debug, &mut st, &raw_lines);
    return (finish_out(&out), debug);
}

pub fn prep(input: &Vec<u8>, options: &PrepOptions) -> Vec<u8> {
    let pair: (Vec<u8>, Vec<Vec<u8>>) = trace_lines(input, options.links);
    return pair.0;
}

pub fn prep_debug(input: &Vec<u8>, options: &PrepOptions) -> (Vec<u8>, Vec<Vec<u8>>) {
    return trace_lines(input, options.links);
}
