// DESIGN.md section 3.9: the IR model for the new outputs (ir.py).
// build_model(flow) runs after flow.layout() and never changes Flow.

use crate::errors::Ex;
use crate::errors::ErrorKind;
use crate::errors::exc;
use crate::jcompat::U;
use crate::jcompat::u_eq;
use crate::jcompat::ul_a;
use crate::jcompat::from_units;
use crate::flow::Flow;
use crate::flow::opt_get;
use crate::model::Message;
use crate::model::is_note;
use crate::model::is_self;
use crate::model::is_undirected;

#[derive(Clone, PartialEq)]
pub enum Kind {
    NOTE,
    SELF,
    MEDIA_REPEAT,
    MEDIA,
    REPLY,
    MESSAGE,
}

#[derive(Clone)]
pub struct Participant {
    pub alias: U,
    pub label: Vec<U>,
    pub index: i64,
}

#[derive(Clone)]
pub struct Event {
    pub id: String,
    pub kind: Kind,
    pub from_: i64,
    pub to: i64,
    pub caption: Vec<U>,
    pub num: Option<i64>,
    pub notes: Vec<U>,
    pub comments: Vec<U>,
    pub src_line: i64,
}

#[derive(Clone)]
pub struct IROptions {
    pub message_numbering: bool,
    pub note_numbering: bool,
    pub html_layout: Option<U>,
}

#[derive(Clone)]
pub struct Model {
    pub title: Option<U>,
    pub participants: Vec<Participant>,
    pub events: Vec<Event>,
    pub options: IROptions,
}

pub fn build_model(f: &Flow) -> Result<Model, Ex> {
    let mut participants: Vec<Participant> = vec![];
    // alias -> index of the FIRST player with that name (step 3)
    let mut alias_index: Vec<OPair2> = vec![];

    let np: usize = f.players.len();
    let mut i: usize = 0;
    while i < np {
        let p: Participant = Participant {
            alias: f.players[i].name.clone(),
            label: f.players[i].caption.clone(),
            index: i as i64,
        };
        participants.push(p);
        let nm: usize = alias_index.len();
        let mut found: bool = false;
        let mut k: usize = 0;
        while k < nm {
            if u_eq(&alias_index[k].k, &f.players[i].name) {
                found = true;
                k = nm;
            }
            k += 1;
        }
        if !found {
            alias_index.push(OPair2 { k: f.players[i].name.clone(), v: i as i64 });
        }
        i += 1;
    }

    let message_numbering: bool = f.do_numbering;
    let note_numbering: bool = f.do_note_numbering;

    let mut events: Vec<Event> = vec![];
    let mut n: i64 = 1;
    let nm2: usize = f.messages.len();
    let mut j: usize = 0;
    while j < nm2 {
        let m: &Message = &f.messages[j];
        let from_: i64 = index_of(&alias_index, &m.from_);
        let to: i64 = index_of(&alias_index, &m.to);
        if from_ < 0 {
            return Err(exc(ErrorKind::ILLEGAL_ARGUMENT, format!("no player named {}", from_units(&m.from_))));
        }
        if to < 0 {
            return Err(exc(ErrorKind::ILLEGAL_ARGUMENT, format!("no player named {}", from_units(&m.to))));
        }

        // Step 4: kind, first matching rule wins.
        let kind: Kind;
        if is_note(m) {
            kind = Kind::NOTE;
        } else if is_self(m) {
            kind = Kind::SELF;
        } else if is_undirected(m) && m.repeat_label {
            kind = Kind::MEDIA_REPEAT;
        } else if is_undirected(m) {
            kind = Kind::MEDIA;
        } else if m.is_reply {
            kind = Kind::REPLY;
        } else {
            kind = Kind::MESSAGE;
        }

        // Steps 5 and 6: num and id.
        let mut num: Option<i64> = None;
        if message_numbering && (from_ != to || note_numbering) {
            num = Some(n);
            n = n + 1;
        }

        events.push(Event {
            id: format!("e{}", j + 1),
            kind: kind,
            from_: from_,
            to: to,
            caption: m.caption.clone(),
            num: num,
            notes: m.notes.clone(),
            comments: m.comments.clone(),
            src_line: m.src_line,
        });
        j += 1;
    }

    // Step 7: title and htmlLayout come from the input options (section 3.3).
    let title: Option<U> = opt_get(f, &ul_a("title".to_string()));
    let html_layout: Option<U> = opt_get(f, &ul_a("htmlLayout".to_string()));
    return Ok(Model {
        title: title,
        participants: participants,
        events: events,
        options: IROptions {
            message_numbering: message_numbering,
            note_numbering: note_numbering,
            html_layout: html_layout,
        },
    });
}

#[derive(Clone)]
pub struct OPair2 {
    pub k: U,
    pub v: i64,
}

pub fn index_of(alias_index: &Vec<OPair2>, alias: &U) -> i64 {
    for e in alias_index {
        if u_eq(&e.k, alias) {
            return e.v;
        }
    }
    return -1;
}
