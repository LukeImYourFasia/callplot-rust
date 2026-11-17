// Render options (DESIGN-PYTHON.md sections 2.3 and 3.1; options.py).

use crate::errors::Ex;
use crate::errors::ErrorKind;
use crate::errors::exc;
use crate::errors::error_str;

#[derive(Clone, PartialEq)]
pub enum TableStyle {
    ARCHTABLE,
    HTMLTABLE,
    HTMLTABLE2,
    HTMLTABLE3,
    NOLINKS,
    WORDTABLE,
}

#[derive(Clone, PartialEq)]
pub enum HtmlLayout {
    TOPBOTTOM,
    SIDEBYSIDE,
}

pub fn table_style_name(t: &TableStyle) -> String {
    match t {
        TableStyle::ARCHTABLE => "archtable".to_string(),
        TableStyle::HTMLTABLE => "htmltable".to_string(),
        TableStyle::HTMLTABLE2 => "htmltable2".to_string(),
        TableStyle::HTMLTABLE3 => "htmltable3".to_string(),
        TableStyle::NOLINKS => "nolinks".to_string(),
        TableStyle::WORDTABLE => "wordtable".to_string(),
    }
}

pub fn html_layout_name(h: &HtmlLayout) -> String {
    match h {
        HtmlLayout::TOPBOTTOM => "topbottom".to_string(),
        HtmlLayout::SIDEBYSIDE => "sidebyside".to_string(),
    }
}

pub struct RenderOptions {
    pub table_style: TableStyle,
    pub html_layout: Option<HtmlLayout>,
    pub id_prefix: String,
}

pub fn render_options_new() -> RenderOptions {
    return RenderOptions {
        table_style: TableStyle::ARCHTABLE,
        html_layout: None,
        id_prefix: "".to_string(),
    };
}

pub fn parse_table_style(name: &String) -> Result<TableStyle, Ex> {
    if *name == "tmltable".to_string() {
        return Ok(TableStyle::HTMLTABLE);
    }
    if *name == "archtable".to_string() {
        return Ok(TableStyle::ARCHTABLE);
    }
    if *name == "htmltable".to_string() {
        return Ok(TableStyle::HTMLTABLE);
    }
    if *name == "htmltable2".to_string() {
        return Ok(TableStyle::HTMLTABLE2);
    }
    if *name == "htmltable3".to_string() {
        return Ok(TableStyle::HTMLTABLE3);
    }
    if *name == "nolinks".to_string() {
        return Ok(TableStyle::NOLINKS);
    }
    if *name == "wordtable".to_string() {
        return Ok(TableStyle::WORDTABLE);
    }
    return Err(exc(ErrorKind::ILLEGAL_ARGUMENT, format!("unknown table style: {}", name)));
}

pub fn parse_html_layout(value: &String) -> Result<HtmlLayout, Ex> {
    if *value == "topbottom".to_string() {
        return Ok(HtmlLayout::TOPBOTTOM);
    }
    if *value == "sidebyside".to_string() {
        return Ok(HtmlLayout::SIDEBYSIDE);
    }
    return Err(exc(ErrorKind::ILLEGAL_ARGUMENT, format!("unknown html layout: {}", value)));
}
