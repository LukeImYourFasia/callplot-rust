// Section 3.10: Mermaid sequenceDiagram from the IR (mermaid.py).
// Every line except the front matter is indented by 4 spaces; the output
// ends with one \n.  No Mermaid autonumber: it cannot number notes.

use crate::escape::esc_mermaid;
use crate::ir::Event;
use crate::ir::Kind;
use crate::ir::Model;
use crate::ir::Participant;
use crate::jcompat::U;
use crate::jcompat::fmt_i;
use crate::jcompat::ul_a;

// {text}: numbering prefix, then escaped caption lines joined with <br/>.
// An empty caption becomes #nbsp; (section 3.10).
pub fn event_text(e: &Event) -> U {
    let mut prefix: U = vec![];
    match e.num {
        None => {
        }
        Some(n) => {
            prefix = ul_a("(".to_string());
            push_units(&mut prefix, &fmt_i(n));
            push_units(&mut prefix, &ul_a(") ".to_string()));
        }
    }
    if e.caption.len() == 0 {
        push_units(&mut prefix, &ul_a("#nbsp;".to_string()));
        return prefix;
    }
    let mut parts: U = vec![];
    let n: usize = e.caption.len();
    let mut i: usize = 0;
    while i < n {
        if i > 0 {
            push_units(&mut parts, &ul_a("<br/>".to_string()));
        }
        push_units(&mut parts, &esc_mermaid(&e.caption[i]));
        i += 1;
    }
    push_units(&mut prefix, &parts);
    return prefix;
}

pub fn mermaid_on(model: &Model) -> U {
    let mut out: U = vec![];
    let has_title: bool;
    match model.title {
        None => {
            has_title = false;
        }
        Some(_) => {
            has_title = true;
        }
    }
    if has_title {
        write_line(&mut out, &ul_a("---".to_string()));
        let mut t: U = ul_a("title: ".to_string());
        let title_copy: Option<U> = model.title.clone();
        match title_copy {
            Some(v) => {
                push_units(&mut t, &esc_mermaid(&v));
            }
            None => {
            }
        }
        write_line(&mut out, &t);
        write_line(&mut out, &ul_a("---".to_string()));
    }
    write_line(&mut out, &ul_a("sequenceDiagram".to_string()));

    let np: usize = model.participants.len();
    let mut i: usize = 0;
    while i < np {
        let mut labels: U = vec![];
        let n: usize = model.participants[i].label.len();
        let mut k: usize = 0;
        while k < n {
            if k > 0 {
                push_units(&mut labels, &ul_a("<br/>".to_string()));
            }
            push_units(&mut labels, &esc_mermaid(&model.participants[i].label[k]));
            k += 1;
        }
        let mut line: U = ul_a("    participant p".to_string());
        push_units(&mut line, &fmt_i((i as i64) + 1));
        push_units(&mut line, &ul_a(" as ".to_string()));
        push_units(&mut line, &labels);
        write_line(&mut out, &line);
        i += 1;
    }

    let ne: usize = model.events.len();
    let mut j: usize = 0;
    while j < ne {
        let e: Event = model.events[j].clone();
        let mut a: U = ul_a("p".to_string());
        push_units(&mut a, &fmt_i(e.from_ + 1));
        let mut b: U = ul_a("p".to_string());
        push_units(&mut b, &fmt_i(e.to + 1));

        let mut line: U = vec![];
        if e.kind == Kind::NOTE {
            line = ul_a("    Note right of ".to_string());
            push_units(&mut line, &a);
            push_units(&mut line, &ul_a(": ".to_string()));
            push_units(&mut line, &event_text(&e));
        } else {
            let arrow: U;
            if e.kind == Kind::REPLY {
                arrow = ul_a("-->>".to_string());
            } else if e.kind == Kind::MEDIA || e.kind == Kind::MEDIA_REPEAT {
                arrow = ul_a("-->".to_string());
            } else {
                arrow = ul_a("->>".to_string());
            }
            line = ul_a("    ".to_string());
            push_units(&mut line, &a);
            push_units(&mut line, &arrow);
            push_units(&mut line, &b);
            push_units(&mut line, &ul_a(": ".to_string()));
            push_units(&mut line, &event_text(&e));
        }
        write_line(&mut out, &line);

        let nn: usize = e.notes.len();
        let mut x: usize = 0;
        while x < nn {
            let mut note: U = ul_a("    %% ".to_string());
            push_units(&mut note, &ul_a(e.id.clone()));
            push_units(&mut note, &ul_a(" note: ".to_string()));
            push_units(&mut note, &esc_mermaid(&e.notes[x]));
            write_line(&mut out, &note);
            x += 1;
        }
        let nc: usize = e.comments.len();
        let mut y: usize = 0;
        while y < nc {
            let mut com: U = ul_a("    %% ".to_string());
            push_units(&mut com, &ul_a(e.id.clone()));
            push_units(&mut com, &ul_a(" comment: ".to_string()));
            push_units(&mut com, &esc_mermaid(&e.comments[y]));
            write_line(&mut out, &com);
            y += 1;
        }
        j += 1;
    }
    return out;
}

pub fn push_units(out: &mut U, t: &U) {
    for c in t {
        out.push(*c);
    }
}

fn write_line(out: &mut U, t: &U) {
    push_units(out, t);
    out.push(0x0A);
}
