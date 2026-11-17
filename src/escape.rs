// Escapes for the new outputs (DESIGN.md sections 3.10, 3.11, 3.12; escape.py).
// One character table applied in ONE pass, so emitted escapes are never
// re-escaped. html.escape is NOT used: it writes ' as &#x27;, not &#39;.

use crate::jcompat::U;
use crate::jcompat::ul_a;
use crate::jcompat::u_ap_ch;

pub fn esc_html(s: &U) -> U {
    let mut out: U = vec![];
    for c in s {
        match c {
            0x26 => {
                for d in ul_a("&amp;".to_string()) {
                    out.push(d);
                }
            }
            0x3C => {
                for d in ul_a("&lt;".to_string()) {
                    out.push(d);
                }
            }
            0x3E => {
                for d in ul_a("&gt;".to_string()) {
                    out.push(d);
                }
            }
            0x22 => {
                for d in ul_a("&quot;".to_string()) {
                    out.push(d);
                }
            }
            0x27 => {
                for d in ul_a("&#39;".to_string()) {
                    out.push(d);
                }
            }
            _ => {
                out.push(*c);
            }
        }
    }
    return out;
}

pub fn esc_mermaid(s: &U) -> U {
    let mut out: U = vec![];
    for c in s {
        match c {
            0x23 => {
                for d in ul_a("#35;".to_string()) {
                    out.push(d);
                }
            }
            0x3B => {
                for d in ul_a("#59;".to_string()) {
                    out.push(d);
                }
            }
            0x3C => {
                for d in ul_a("#lt;".to_string()) {
                    out.push(d);
                }
            }
            0x3E => {
                for d in ul_a("#gt;".to_string()) {
                    out.push(d);
                }
            }
            0x22 => {
                for d in ul_a("#quot;".to_string()) {
                    out.push(d);
                }
            }
            _ => {
                out.push(*c);
            }
        }
    }
    return out;
}

// Deviation logged at W13: the pinned plantuml.jar rejects ~" inside a
// quoted participant string, so " becomes &#34; (the jar renders &#34;
// back as a literal ").
pub fn esc_plantuml(s: &U) -> U {
    let mut out: U = vec![];
    for c in s {
        match c {
            0x22 => {
                for d in ul_a("&#34;".to_string()) {
                    out.push(d);
                }
            }
            0x7E => {
                for d in ul_a("~~".to_string()) {
                    out.push(d);
                }
            }
            0x2A => {
                for d in ul_a("~*".to_string()) {
                    out.push(d);
                }
            }
            0x5F => {
                for d in ul_a("~_".to_string()) {
                    out.push(d);
                }
            }
            0x2F => {
                for d in ul_a("~/".to_string()) {
                    out.push(d);
                }
            }
            0x2D => {
                for d in ul_a("~-".to_string()) {
                    out.push(d);
                }
            }
            0x3C => {
                for d in ul_a("~<".to_string()) {
                    out.push(d);
                }
            }
            0x5B => {
                for d in ul_a("~[".to_string()) {
                    out.push(d);
                }
            }
            0x5C => {
                for d in ul_a("~\\".to_string()) {
                    out.push(d);
                }
            }
            _ => {
                out.push(*c);
            }
        }
    }
    return out;
}
