// Error transport for the whole port: fallible functions return Result<T, Ex>.
//
// Copyright (C) 2004 dynamicsoft, Inc.
//
// This program is free software; you can redistribute it and/or modify it
// under the terms of the GNU Lesser General Public License as published by
// the Free Software Foundation; either version 2.1 of the License, or (at
// your option) any later version.  A copy of the license is in LICENSE.

// The CLI turns an Ex into the "callplot: NAME: message" line and exits 1
// (DESIGN.md section 3.7).

#[derive(PartialEq, Clone)]
pub enum ErrorKind {
    NUMBER_FORMAT,
    ILLEGAL_ARGUMENT,
    INDEX,
    NULL_POINTER,
    ARITHMETIC,
    NEGATIVE_ARRAY_SIZE,
    TABLE_INPUT,
    RUNTIME,
}

pub struct Ex {
    pub kind: ErrorKind,
    pub msg: Option<String>,
}

pub fn error_name(kind: ErrorKind) -> String {
    match kind {
        ErrorKind::NUMBER_FORMAT => "NumberFormatError".to_string(),
        ErrorKind::ILLEGAL_ARGUMENT => "IllegalArgumentError".to_string(),
        ErrorKind::INDEX => "IndexError".to_string(),
        ErrorKind::NULL_POINTER => "NullPointerError".to_string(),
        ErrorKind::ARITHMETIC => "ArithmeticError".to_string(),
        ErrorKind::NEGATIVE_ARRAY_SIZE => "NegativeArraySizeError".to_string(),
        ErrorKind::TABLE_INPUT => "TableInputError".to_string(),
        _ => "RuntimeException".to_string(),
    }
}

pub fn exc(kind: ErrorKind, msg: String) -> Ex {
    return Ex { kind: kind, msg: Some(msg) };
}

pub fn exc_plain(kind: ErrorKind, msg: String) -> Ex {
    return Ex { kind: kind, msg: Some(msg) };
}

pub fn error_str(e: Ex) -> String {
    let name: String = error_name(e.kind);
    let msg: String = match e.msg {
        Some(m) => m,
        None => "".to_string(),
    };
    if msg.is_empty() {
        return name;
    }
    return format!("{}: {}", name, msg);
}
