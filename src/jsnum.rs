// JS number printing for the new outputs (DESIGN-RUST.md section 3.7; jsnum.rs).
// The new outputs have no Java oracle; their numbers print as JS prints them.

use crate::errors::Ex;
use crate::errors::ErrorKind;
use crate::errors::exc;

// String(x) in a JS template literal.  -0 gives "0".  Within
// 1e-6 <= |x| < 1e21 format!("{}", v) gives the same shortest digits as JS.
pub fn js_num(v: f64) -> Result<String, Ex> {
    if v.is_nan() {
        return Ok("NaN".to_string());
    }
    if v.is_infinite() {
        if v < 0.0 {
            return Ok("-Infinity".to_string());
        }
        return Ok("Infinity".to_string());
    }
    if v == 0.0 {
        return Ok("0".to_string());
    }
    let a: f64 = if v < 0.0 { 0.0 - v } else { v };
    if a < 1e-6 || a >= 1e21 {
        return Err(exc(ErrorKind::RUNTIME, "js_num: number outside the plain range".to_string()));
    }
    return Ok(format!("{}", v));
}

// Math.round: half toward +Infinity (JS Math.round(-2.5) is -2).
// NOT (x + 0.5).floor(): that gives 1 for 0.49999999999999994, JS gives 0.
pub fn js_round(v: f64) -> f64 {
    let f: f64 = v.floor();
    if v - f >= 0.5 {
        return f + 1.0;
    }
    return f;
}

// x.toFixed(k): round the EXACT binary value to k decimals, ties away from
// zero.  Get the exact digits with format!("{:.1$}", x, k + 30), then round
// that decimal text by hand.
pub fn js_to_fixed(x: f64, k: i64) -> String {
    if x.is_nan() {
        return "NaN".to_string();
    }
    if x.is_infinite() {
        if x < 0.0 {
            return "-Infinity".to_string();
        }
        return "Infinity".to_string();
    }
    let neg: bool = x < 0.0;
    let v: f64 = if neg { 0.0 - x } else { x };
    let kk: usize = if k < 0 { 0usize } else { k as usize };
    let s: String = format!("{:.1$}", v, kk + 30);
    let bs: Vec<u8> = crate::jcompat::str_bytes(s);
    let mut dot: usize = 0;
    let mut i: usize = 0;
    while i < bs.len() {
        if bs[i] == 0x2E {
            dot = i;
            i = bs.len();
        }
        i += 1;
    }
    let mut int_b: Vec<u8> = vec![];
    let mut i: usize = 0;
    while i < dot {
        int_b.push(bs[i]);
        i += 1;
    }
    let mut frac_all: Vec<u8> = vec![];
    let mut i2: usize = dot + 1;
    while i2 < bs.len() {
        frac_all.push(bs[i2]);
        i2 += 1;
    }
    // digits = integer part + first kk fraction digits
    let mut dig: Vec<u8> = int_b;
    let mut j: usize = 0;
    while j < kk {
        dig.push(frac_all[j]);
        j += 1;
    }
    // the first dropped digit decides: >= '5' rounds up (away from zero)
    if frac_all[kk] >= 0x35 {
        let mut c: i64 = 1;
        let mut i3: i64 = dig.len() as i64;
        while i3 > 0 && c > 0 {
            let p: usize = i3 as usize - 1;
            if dig[p] == 0x39 {
                dig[p] = 0x30;
            } else {
                dig[p] = dig[p] + 1;
                c = 0;
            }
            i3 -= 1;
        }
        if c > 0 {
            dig.insert(0, 0x31);
        }
    }
    let dl: usize = dig.len();
    let int_part: Vec<u8> = if kk < dl { vec_slice(&dig, 0, dl - kk) } else { vec![0x30] };
    let mut out: Vec<u8> = vec![];
    if neg {
        out.push(0x2D);
    }
    let mut i4: usize = 0;
    while i4 < int_part.len() {
        out.push(int_part[i4]);
        i4 += 1;
    }
    if kk > 0 {
        out.push(0x2E);
        let mut i5: usize = dl - kk;
        while i5 < dl {
            out.push(dig[i5]);
            i5 += 1;
        }
    }
    return bytes_to_string(out);
}

// Slice a byte vector [at, at+n).
pub fn vec_slice(v: &Vec<u8>, at: usize, n: usize) -> Vec<u8> {
    let mut out: Vec<u8> = vec![];
    let mut i: usize = at;
    while i < at + n {
        out.push(v[i]);
        i += 1;
    }
    return out;
}

pub fn bytes_to_string(v: Vec<u8>) -> String {
    return crate::jcompat::bytes_str(v);
}
