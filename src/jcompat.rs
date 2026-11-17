// Java-semantics helpers (DESIGN.md section 3.5; jcompat.py).
// Text inside the port is U = Vec<u16>: UTF-16 code units, like Java char[].

use crate::errors::Ex;
use crate::errors::ErrorKind;
use crate::errors::exc;

pub type U = Vec<u16>;

pub fn exc_index() -> Ex {
    return exc(ErrorKind::INDEX, "Index out of range".to_string());
}

pub fn exc_arith(msg: String) -> Ex {
    return exc(ErrorKind::ARITHMETIC, msg);
}

pub fn exc_neg(msg: String) -> Ex {
    return exc(ErrorKind::NEGATIVE_ARRAY_SIZE, msg);
}

pub fn exc_num(msg: String) -> Ex {
    return exc(ErrorKind::NUMBER_FORMAT, msg);
}

pub fn exc_null() -> Ex {
    return exc(ErrorKind::NULL_POINTER, "".to_string());
}

pub fn exc_runtime(msg: String) -> Ex {
    return exc(ErrorKind::RUNTIME, msg);
}

// ---- unit-string building and I/O conversion ----

// Build a unit string from an ASCII Rust literal (the common case).
pub fn ul_a(lit: String) -> U {
    let mut out: U = vec![];
    for b in lit.as_bytes() {
        out.push(*b as u16);
    }
    return out;
}

// Rust String -> units, surrogate-expanding above U+FFFF (jcompat to_units).
pub fn to_units(text: String) -> U {
    let mut out: U = vec![];
    for c in text.chars() {
        let cp = c as u32;
        if cp > 0xFFFF {
            let k = cp - 0x10000;
            out.push((0xD800 + (k >> 10)) as u16);
            out.push((0xDC00 + (k & 0x3FF)) as u16);
        } else {
            out.push(cp as u16);
        }
    }
    return out;
}

// Units -> Rust String, joining valid surrogate pairs (jcompat from_units).
// A lone surrogate cannot exist in a Rust String, so it becomes the byte the
// Java/Python writer emits for it: '?' (encode utf-8 errors=replace). Real U+FFFD
// units are untouched.
pub fn from_units(s: &U) -> String {
    let mut out: String = "".to_string();
    let n = s.len();
    let mut i: usize = 0;
    while i < n {
        let c = s[i];
        if c >= 0xD800 && c <= 0xDBFF && i + 1 < n && s[i + 1] >= 0xDC00 && s[i + 1] <= 0xDFFF {
            let hi = (c as u32) - 0xD800;
            let lo = (s[i + 1] as u32) - 0xDC00;
            let cp = 0x10000 + (hi << 10) + lo;
            out.push(char::from_u32(cp).unwrap());
            i += 2;
        } else if c >= 0xD800 && c <= 0xDFFF {
            out.push('?');
            i += 1;
        } else {
            out.push(char::from_u32(c as u32).unwrap());
            i += 1;
        }
    }
    return out;
}

// ---- unit-string primitives (Java char[] semantics) ----

pub fn u_len(s: &U) -> i64 { return s.len() as i64; }

pub fn u_at(s: &U, i: i64) -> u16 { return s[i as usize]; }

pub fn u_sub(s: &U, at: i64, len: i64) -> U {
    let mut out: U = vec![];
    let mut k: i64 = 0;
    while k < len {
        out.push(u_at(s, at + k));
        k += 1;
    }
    return out;
}

pub fn u_eq(a: &U, b: &U) -> bool { return a == b; }

pub fn u_startsWith(s: &U, pre: &U) -> bool {
    let n = u_len(pre);
    if u_len(s) < n {
        return false;
    }
    let mut i: i64 = 0;
    while i < n {
        if u_at(s, i) != u_at(pre, i) {
            return false;
        }
        i += 1;
    }
    return true;
}

// Index of unit c, or -1 (Java String.indexOf).
pub fn u_index(s: &U, c: u16) -> i64 {
    let n = u_len(s);
    let mut i: i64 = 0;
    while i < n {
        if u_at(s, i) == c {
            return i;
        }
        i += 1;
    }
    return -1;
}

pub fn u_repeat(c: u16, k: i64) -> U {
    let mut out: U = vec![];
    let mut i: i64 = 0;
    while i < k {
        out.push(c);
        i += 1;
    }
    return out;
}

pub fn u_ap(s: &U, t: &U) -> U {
    let mut out: U = vec![];
    for c in s {
        out.push(*c);
    }
    for c in t {
        out.push(*c);
    }
    return out;
}

pub fn u_ap_ch(s: &U, c: u16) -> U {
    let mut out: U = vec![];
    for x in s {
        out.push(*x);
    }
    out.push(c);
    return out;
}

pub fn u_zeros(k: i64) -> U { return u_repeat(0x30, k); }

// ---- BufferedReader.readLine (DESIGN-PYTHON.md section 3.2) ----

pub fn read_lines(text: &U) -> Vec<U> {
    let mut lines: Vec<U> = vec![];
    if u_len(text) == 0 {
        return lines;
    }
    let n = u_len(text);
    let mut start: i64 = 0;
    let mut i: i64 = 0;
    while i < n {
        let ch = u_at(text, i);
        if ch == 0x0A {
            lines.push(u_sub(text, start, i - start));
            i += 1;
            start = i;
        } else if ch == 0x0D {
            lines.push(u_sub(text, start, i - start));
            if i + 1 < n && u_at(text, i + 1) == 0x0A {
                i += 2;
            } else {
                i += 1;
            }
            start = i;
        } else {
            i += 1;
        }
    }
    if start < n {
        lines.push(u_sub(text, start, n - start));
    }
    return lines;
}

// ---- number formatting ----

// Java String concatenation of an int ("" + int).
pub fn fmt_i(n: i64) -> U {
    return to_units(format!("{}", n));
}

// Shortest round-trip digits + decimal-point index for a finite double,
// parsed from Rust's {:e} form (same shortest digits as Python repr).
// value == int(digits) * 10 ** (n - len(digits)); digits have no
// leading or trailing zeros.
pub struct DExp {
    pub digits: Vec<u8>,
    pub n: i64,
}

pub fn java_digits_exp(x: f64) -> DExp {
    let ax: f64 = x.abs();
    let s: String = format!("{:e}", ax);
    let bytes: Vec<u8> = str_bytes(s);
    let n = bytes.len();
    let mut epos: i64 = -1;
    let mut k: usize = 0;
    while k < n {
        if bytes[k] == 0x65 || bytes[k] == 0x45 {
            epos = k as i64;
            k = n;
        }
        k += 1;
    }
    let mut head: Vec<u8> = vec![];
    if epos >= 0 {
        let mut j: usize = 0;
        while (j as i64) < epos {
            head.push(bytes[j]);
            j += 1;
        }
    } else {
        let hn2 = bytes.len();
        let mut j2: usize = 0;
        while j2 < hn2 {
            head.push(bytes[j2]);
            j2 += 1;
        }
    }
    let mut dot: i64 = -1;
    let hn = head.len();
    let mut jj: usize = 0;
    while jj < hn {
        if head[jj] == 0x2E {
            dot = jj as i64;
            jj = hn;
        }
        jj += 1;
    }
    let mut digits: Vec<u8> = vec![];
    if dot >= 0 {
        let mut kk: usize = 0;
        while (kk as i64) < dot {
            digits.push(head[kk]);
            kk += 1;
        }
        kk = dot as usize + 1;
        while kk < hn {
            digits.push(head[kk]);
            kk += 1;
        }
    } else {
        let hn3 = head.len();
        let mut kk: usize = 0;
        while kk < hn3 {
            digits.push(head[kk]);
            kk += 1;
        }
    }
    let mut expo: i64 = 0;
    if epos >= 0 {
        let mut esign: i64 = 1;
        let mut kk: usize = epos as usize + 1;
        if kk < n && (bytes[kk] == 0x2B || bytes[kk] == 0x2D) {
            if bytes[kk] == 0x2D {
                esign = -1;
            }
            kk += 1;
        }
        while kk < n {
            expo = expo * 10 + ((bytes[kk] as i64) - 0x30);
            kk += 1;
        }
        expo = esign * expo;
    }
    let int_len: i64;
    if dot >= 0 {
        int_len = dot;
    } else {
        int_len = head.len() as i64;
    }
    let mut nd: i64 = int_len + expo;
    while digits.len() > 1 && digits[0] == 0x30 {
        let mut rest: Vec<u8> = vec![];
        let dn = digits.len();
        let mut kk: usize = 1;
        while kk < dn {
            rest.push(digits[kk]);
            kk += 1;
        }
        digits = rest;
        nd -= 1;
    }
    while digits.len() > 1 && digits[digits.len() - 1] == 0x30 {
        let dn: usize = digits.len();
        let mut rest: Vec<u8> = vec![];
        let mut kk: usize = 0;
        while kk < dn - 1 {
            rest.push(digits[kk]);
            kk += 1;
        }
        digits = rest;
    }
    if digits.is_empty() {
        let mut one: Vec<u8> = vec![];
        one.push(0x30);
        return DExp { digits: one, n: 1 };
    }
    return DExp { digits: digits, n: nd };
}

// Java Double.toString on OpenJDK 1.8.0_504 (DESIGN.md section 3.5 table).
pub fn fmt_d(x: f64) -> String {
    if x.is_nan() {
        return "NaN".to_string();
    }
    if x.is_infinite() {
        if x < 0.0 {
            return "-Infinity".to_string();
        }
        return "Infinity".to_string();
    }
    if x == 0.0 {
        // Java prints "0.0" and "-0.0"; distinguish via the sign bit.
        if fmt_has_minus(x) {
            return "-0.0".to_string();
        }
        return "0.0".to_string();
    }
    let dexp: DExp = java_digits_exp(x);
    let dlen: i64 = dexp.digits.len() as i64;
    let digits: String = bytes_str(dexp.digits);
    let n: i64 = dexp.n;
    let mut sign: String = "".to_string();
    if x < 0.0 {
        sign = "-".to_string();
    }
    if n >= -2 && n <= 7 {
        if n <= 0 {
            return format!("{}0.{}{}", sign, u_zeros_s(-n), digits);
        }
        if n >= dlen {
            return format!("{}{}{}.0", sign, digits, u_zeros_s(n - dlen));
        }
        return format!("{}{}.{}", sign, digit_slice(&digits, 0, n), digit_slice(&digits, n, dlen - n));
    }
    let head: String = digit_slice(&digits, 0, 1);
    let tail: String = digit_slice(&digits, 1, dlen - 1);
    let mut mant: String = head.to_string();
    if tail.is_empty() {
        mant = format!("{}.0", head);
    } else {
        mant = format!("{}.{}", head, tail);
    }
    return format!("{}{}E{}", sign, mant, format!("{}", n - 1));
}

pub fn fmt_has_minus(x: f64) -> bool {
    for b in format!("{}", x).as_bytes() {
        return *b == 0x2D;
    }
    return false;
}

pub fn digit_slice(s: &String, at: i64, len: i64) -> String {
    let b: Vec<u8> = str_bytes(s.to_string());
    let mut out: Vec<u8> = vec![];
    let mut k: i64 = 0;
    while k < len {
        out.push(b[(at + k) as usize]);
        k += 1;
    }
    return bytes_str(out);
}

pub fn str_bytes(s: String) -> Vec<u8> {
    let mut out: Vec<u8> = vec![];
    for b in s.as_bytes() {
        out.push(*b);
    }
    return out;
}

pub fn bytes_str(v: Vec<u8>) -> String {
    return String::from_utf8(v).unwrap();
}

pub fn u_zeros_s(k: i64) -> String {
    let mut out: String = "".to_string();
    let mut i: i64 = 0;
    while i < k {
        out = format!("{}0", out);
        i += 1;
    }
    return out;
}

// ---- Java int and double arithmetic ----

// Java int division (section 3.3): rounds toward zero, and the result wraps
// back into the int range, so INT_MIN / -1 gives INT_MIN instead of throwing.
pub fn idiv(a: i64, b: i64) -> Result<i64, Ex> {
    if b == 0 {
        return Err(exc_arith("/ by zero".to_string()));
    }
    let q: i64 = wrap_i32(a) / wrap_i32(b);
    return Ok(wrap_i32(q));
}

// the Java int of an i64 (two's-complement wrap into -2^31..2^31-1)
pub fn wrap_i32(v: i64) -> i64 {
    let m: i64 = v & 0xFFFFFFFF;
    if m >= 2147483648 {
        return m - 4294967296;
    }
    return m;
}

pub fn fdiv(a: f64, b: f64) -> f64 {
    if b == 0.0 {
        if a.is_nan() || a == 0.0 {
            let z: f64 = 0.0;
            return z / z;
        }
        let neg: bool = (a < 0.0) != fmt_has_minus(b);
        if neg {
            return 0.0 - 1.0 / 0.0;
        }
        return 1.0 / 0.0;
    }
    return a / b;
}

// Integer.parseInt (strict): Java 8 accepts a leading + or -.
pub fn parse_java_int(s: &U) -> Result<i64, Ex> {
    let n = u_len(s);
    let mut pos: i64 = 0;
    let mut neg: bool = false;
    if n > 0 && (u_at(s, 0) == 0x2B || u_at(s, 0) == 0x2D) {
        neg = u_at(s, 0) == 0x2D;
        pos = 1;
    }
    if pos >= n {
        return Err(exc_num(format!("For input string: \"{}\"", from_units(s))));
    }
    let mut k: i64 = pos;
    while k < n {
        let c = u_at(s, k);
        if c < 0x30 || c > 0x39 {
            return Err(exc_num(format!("For input string: \"{}\"", from_units(s))));
        }
        k += 1;
    }
    let mut v: i64 = 0;
    k = pos;
    while k < n {
        let c = u_at(s, k);
        v = v * 10 + (c as i64 - 0x30);
        if v > 2147483648 {
            return Err(exc_num(format!("value out of int range: \"{}\"", from_units(s))));
        }
        k += 1;
    }
    let mut r: i64 = v;
    if neg {
        r = 0 - v;
    }
    if r < -2147483648 || r > 2147483647 {
        return Err(exc_num(format!("value out of int range: \"{}\"", from_units(s))));
    }
    return Ok(r);
}

// ---- JS number printing (the -html geometry is JS arithmetic) ----

pub fn js_round(v: f64) -> i64 {
    return (v + 0.5).floor() as i64;
}

// An integral double prints without a decimal point; a non-integral one
// prints with the shortest round-trip decimal (same digits as repr).
pub fn js_num(v: f64) -> String {
    if v.is_nan() {
        return "NaN".to_string();
    }
    if v.is_infinite() {
        if v < 0.0 {
            return "-Infinity".to_string();
        }
        return "Infinity".to_string();
    }
    if v.floor() == v {
        return format!("{}", v as i64);
    }
    let dexp: DExp = java_digits_exp(v);
    let dlen: i64 = dexp.digits.len() as i64;
    let digits: String = bytes_str(dexp.digits);
    let n: i64 = dexp.n;
    let mut sign: String = "".to_string();
    if v < 0.0 {
        sign = "-".to_string();
    }
    if n <= 0 {
        return format!("{}0.{}{}", sign, u_zeros_s(-n), digits);
    }
    return format!("{}{}.{}", sign, digit_slice(&digits, 0, n), digit_slice(&digits, n, dlen - n));
}

// ---- Java String.trim (strips only codes <= U+0020) ----

pub fn java_trim(s: U) -> U {
    let n = u_len(&s);
    let mut i: i64 = 0;
    let mut j: i64 = n;
    while i < j && (u_at(&s, i) as i64) <= 0x20 {
        i += 1;
    }
    while j > i && (u_at(&s, j - 1) as i64) <= 0x20 {
        j -= 1;
    }
    return u_sub(&s, i, j - i);
}

// ---- Parser.java:75 split: every '/' not preceded by '\' ----

pub fn java_split_slash(line: U) -> Vec<U> {
    let n = u_len(&line);
    let mut marks: Vec<i64> = vec![];
    let mut i: i64 = 1;
    while i < n {
        if u_at(&line, i) == 0x2F && u_at(&line, i - 1) != 0x5C {
            marks.push(i);
        }
        i += 1;
    }
    if marks.is_empty() {
        return vec![line];
    }
    let mut segs: Vec<U> = vec![];
    let mut prev: i64 = 0;
    for m in marks {
        segs.push(u_sub(&line, prev, m - prev));
        prev = m + 1;
    }
    segs.push(u_sub(&line, prev, n - prev));
    while !segs.is_empty() {
        let last = &segs[segs.len() - 1];
        if last.is_empty() {
            segs.pop();
        } else {
            break;
        }
    }
    return segs;
}

pub fn tokenize(s: U, delim: u16) -> Vec<U> {
    // java.util.StringTokenizer: split on each delim char, drop empties.
    let mut out: Vec<U> = vec![];
    let n = u_len(&s);
    let mut start: i64 = 0;
    let mut i: i64 = 0;
    while i < n {
        if u_at(&s, i) == delim {
            if i > start {
                out.push(u_sub(&s, start, i - start));
            }
            start = i + 1;
        }
        i += 1;
    }
    if n > start {
        out.push(u_sub(&s, start, n - start));
    }
    return out;
}

// Unescape "\/" -> "/" for all fields except the command word.
pub fn replace_slash(p: U) -> U {
    let mut out: U = vec![];
    let n = u_len(&p);
    let mut i: i64 = 0;
    while i < n {
        if i + 1 < n && u_at(&p, i) == 0x5C && u_at(&p, i + 1) == 0x2F {
            out.push(0x2F);
            i += 2;
        } else {
            out.push(u_at(&p, i));
            i += 1;
        }
    }
    return out;
}

// ---- char[] buffer (CharBuf.java) ----

pub struct CharBuf {
    pub a: U,
}

pub fn char_buf_new(width: i64) -> Result<CharBuf, Ex> {
    if width < 0 {
        return Err(exc_neg(format!("{}", width)));
    }
    let mut cells: U = vec![];
    let mut i: i64 = 0;
    while i < width {
        cells.push(0);
        i += 1;
    }
    return Ok(CharBuf { a: cells });
}

pub fn buf_get(buf: &CharBuf, i: i64) -> Result<u16, Ex> {
    if i < 0 || i >= buf.a.len() as i64 {
        return Err(exc_index());
    }
    return Ok(buf.a[i as usize]);
}

pub fn buf_set(buf: &CharBuf, i: i64, ch: u16) -> Result<CharBuf, Ex> {
    if i < 0 || i >= buf.a.len() as i64 {
        return Err(exc_index());
    }
    let mut out: CharBuf = CharBuf { a: vec![] };
    for c in &buf.a {
        out.a.push(*c);
    }
    out.a[i as usize] = ch;
    return Ok(out);
}

pub fn buf_copy_in(buf: &CharBuf, src: &U, at: i64, n: i64) -> Result<CharBuf, Ex> {
    // System.arraycopy bounds: the whole range is checked before any write.
    if n < 0 || at < 0 || at + n > buf.a.len() as i64 {
        return Err(exc_index());
    }
    let mut out: CharBuf = CharBuf { a: vec![] };
    for c in &buf.a {
        out.a.push(*c);
    }
    let mut k: i64 = 0;
    while k < n {
        out.a[(at + k) as usize] = u_at(src, k);
        k += 1;
    }
    return Ok(out);
}

pub fn buf_slice(buf: &CharBuf, at: i64, n: i64) -> Result<U, Ex> {
    // new String(char[], offset, length)
    let mut out: U = vec![];
    let mut k: i64 = 0;
    while k < n {
        out.push(buf_get(buf, at + k)?);
        k += 1;
    }
    return Ok(out);
}
