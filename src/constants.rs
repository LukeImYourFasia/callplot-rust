// Mode bits and renderer constants (DESIGN.md sections 1.4/2.2; constants.py).
// Bit values are internal: no golden or CLI output depends on them.

pub const MODE_TEXT: usize = 0x01;
pub const MODE_PIC: usize = 0x02;
pub const MODE_XML: usize = 0x04;
pub const MODE_VML: usize = 0x08;
pub const MODE_SVG: usize = 0x10;
pub const MODE_GRAFFLE: usize = 0x20;
pub const MODE_MERMAID: usize = 0x40;
pub const MODE_PLANTUML: usize = 0x80;
pub const MODE_HTML: usize = 0x100;
pub const MODE_TABLE: usize = 0x200;
pub const MODE_VXML: usize = MODE_VML | MODE_XML;

pub const GRAFFLE_MULT: i64 = 14;
pub const GRAFFLE_TOP_MARG: i64 = 15;
pub const GRAFFLE_LINE_TXT_OFST: i64 = -7;

pub const VML_TEXT_XOFF: i64 = 1;
pub const VML_TEXT_YOFF: i64 = 2;
