// Section 3.11: PlantUML from the IR (plantuml.py).
// Caption lines join with a literal \n (backslash + n); media arrows are
// teal with two heads; notes and comments land as ' source comments.

use crate::escape::esc_plantuml;
use crate::ir::Event;
use crate::ir::Kind;
use crate::ir::Model;
use crate::jcompat::U;
use crate::jcompat::fmt_i;
use crate::jcompat::ul_a;

pub fn push_units(out: &mut U, t: &U) {
    for c in t {
        out.push(*c);
    }
}

fn write_line(out: &mut U, t: &U) {
    push_units(out, t);
    out.push(0x0A);
}

// {text}: numbering prefix + escaped caption lines joined with \n.
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
    let mut parts: U = vec![];
    let n: usize = e.caption.len();
    let mut i: usize = 0;
    while i < n {
        if i > 0 {
            push_units(&mut parts, &ul_a("\\n".to_string()));
        }
        push_units(&mut parts, &esc_plantuml(&e.caption[i]));
        i += 1;
    }
    push_units(&mut prefix, &parts);
    return prefix;
}

pub fn plantuml_on(model: &Model) -> U {
    let mut out: U = vec![];
    write_line(&mut out, &ul_a("@startuml".to_string()));

    let title_copy: Option<U> = model.title.clone();
    match title_copy {
        Some(v) => {
            let mut t: U = ul_a("title ".to_string());
            push_units(&mut t, &esc_plantuml(&v));
            write_line(&mut out, &t);
        }
        None => {
        }
    }

    write_line(&mut out, &ul_a("hide footbox".to_string()));

    let np: usize = model.participants.len();
    let mut i: usize = 0;
    while i < np {
        let mut labels: U = vec![];
        let n: usize = model.participants[i].label.len();
        let mut k: usize = 0;
        while k < n {
            if k > 0 {
                push_units(&mut labels, &ul_a("\\n".to_string()));
            }
            push_units(&mut labels, &esc_plantuml(&model.participants[i].label[k]));
            k += 1;
        }
        let mut line: U;
        if u_len2(&labels) == 0 {
            // the pinned jar rejects participant "" as pN; declare it bare
            line = ul_a("participant p".to_string());
            push_units(&mut line, &fmt_i((i as i64) + 1));
        } else {
            line = ul_a("participant \"".to_string());
            push_units(&mut line, &labels);
            push_units(&mut line, &ul_a("\" as p".to_string()));
            push_units(&mut line, &fmt_i((i as i64) + 1));
        }
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
            line = ul_a("note right of ".to_string());
            push_units(&mut line, &a);
            push_units(&mut line, &ul_a(" : ".to_string()));
            push_units(&mut line, &event_text(&e));
        } else {
            let arrow: U;
            if e.kind == Kind::REPLY {
                arrow = ul_a("-->".to_string());
            } else if e.kind == Kind::MEDIA || e.kind == Kind::MEDIA_REPEAT {
                arrow = ul_a("<-[#008080]->".to_string());
            } else {
                arrow = ul_a("->".to_string());
            }
            line = ul_a("".to_string());
            push_units(&mut line, &a);
            push_units(&mut line, &ul_a(" ".to_string()));
            push_units(&mut line, &arrow);
            push_units(&mut line, &ul_a(" ".to_string()));
            push_units(&mut line, &b);
            push_units(&mut line, &ul_a(" : ".to_string()));
            push_units(&mut line, &event_text(&e));
        }
        write_line(&mut out, &line);

        let nn: usize = e.notes.len();
        let mut x: usize = 0;
        while x < nn {
            let mut note: U = ul_a("' ".to_string());
            push_units(&mut note, &ul_a(e.id.clone()));
            push_units(&mut note, &ul_a(" note: ".to_string()));
            push_units(&mut note, &esc_plantuml(&e.notes[x]));
            write_line(&mut out, &note);
            x += 1;
        }
        let nc: usize = e.comments.len();
        let mut y: usize = 0;
        while y < nc {
            let mut com: U = ul_a("' ".to_string());
            push_units(&mut com, &ul_a(e.id.clone()));
            push_units(&mut com, &ul_a(" comment: ".to_string()));
            push_units(&mut com, &esc_plantuml(&e.comments[y]));
            write_line(&mut out, &com);
            y += 1;
        }
        j += 1;
    }

    write_line(&mut out, &ul_a("@enduml".to_string()));
    return out;
}

fn u_len2(s: &U) -> i64 {
    return s.len() as i64;
}
