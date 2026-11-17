// The public render entry point (DESIGN.md section 4.1 API; api.py).
// render() does to_units, parse, layout, the renderer pick, from_units.

use crate::constants::MODE_TEXT;
use crate::constants::MODE_XML;
use crate::constants::MODE_PIC;
use crate::constants::MODE_MERMAID;
use crate::constants::MODE_PLANTUML;
use crate::constants::MODE_VML;
use crate::constants::MODE_VXML;
use crate::constants::MODE_SVG;
use crate::constants::MODE_GRAFFLE;
use crate::constants::MODE_HTML;
use crate::constants::MODE_TABLE;
use crate::errors::Ex;
use crate::errors::ErrorKind;
use crate::errors::exc;
use crate::jcompat::U;
use crate::jcompat::to_units;
use crate::parser::parse;
use crate::flow::Flow;
use crate::flow::flow_layout;
use crate::ir::build_model;
use crate::ir::Model;
use crate::text::text_on;
use crate::pic::pic_on;
use crate::mermaid::mermaid_on;
use crate::plantuml::plantuml_on;
use crate::table::table_on;
use crate::options::RenderOptions;
use crate::options::TableStyle;
use crate::options::parse_table_style;
use crate::options::render_options_new;
use crate::svg::svg_on;
use crate::vml::vml_on;
use crate::graffle::graffle_on;
use crate::context::RunContext;
use crate::context::run_context_fresh;
use crate::html_page::page_on;
use crate::options::HtmlLayout;
use crate::options::parse_html_layout;

// The parse+layout prefix shared by render() and render_opts().  Callers own
// the flow value; renderers that need it mutable get it via &mut.
pub fn render_flow(f: &mut Flow, mode: usize) -> Result<U, Ex> {
    if mode == MODE_TEXT || mode == MODE_XML {
        return text_on(f, mode);
    }
    if mode == MODE_PIC {
        return pic_on(f, mode);
    }
    if mode == MODE_SVG {
        let mut ctx: RunContext = run_context_fresh();
        return svg_on(f, mode, &mut ctx);
    }
    if mode == MODE_VML || mode == MODE_VXML {
        let mut ctx: RunContext = run_context_fresh();
        return vml_on(f, mode, &mut ctx);
    }
    if mode == MODE_GRAFFLE {
        let mut ctx: RunContext = run_context_fresh();
        return graffle_on(f, mode, &mut ctx);
    }
    if mode == MODE_TABLE {
        let opts: RenderOptions = crate::options::render_options_new();
        return table_on(f, &opts);
    }
    if mode == MODE_MERMAID {
        let model: Model = build_model(f)?;
        return Ok(mermaid_on(&model));
    }
    if mode == MODE_PLANTUML {
        let model: Model = build_model(f)?;
        return Ok(plantuml_on(&model));
    }
    if mode == MODE_HTML {
        // the default RenderOptions: layout comes from the raw htmlLayout opt
        let opts: RenderOptions = crate::options::render_options_new();
        let model: Model = build_model(f)?;
        return page_on(&model, &opts);
    }
    return Err(exc(ErrorKind::RUNTIME, format!("unknown output mode: {}", mode)));
}

pub fn render(text: String, mode: usize) -> Result<U, Ex> {
    let f: Flow = parse(&to_units(text))?;
    let mut f2: Flow = f;
    flow_layout(&mut f2)?;
    return render_flow(&mut f2, mode);
}

// render() with an explicit table style (the golden harness for table-* modes
// passes RenderOptions(table_style=parse_table_style(style))); any other mode
// ignores the style.
pub fn render_opts(text: String, mode: usize, table_style: String) -> Result<U, Ex> {
    let f: Flow = parse(&to_units(text))?;
    let mut f2: Flow = f;
    flow_layout(&mut f2)?;
    if mode == MODE_TABLE {
        let style: TableStyle = crate::options::parse_table_style(&table_style)?;
        let opts: RenderOptions = crate::options::render_options_new();
        let mut opts2: RenderOptions = opts;
        opts2.table_style = style;
        return table_on(&mut f2, &opts2);
    }
    return render_flow(&mut f2, mode);
}

// render() with an explicit html layout (the snapshot harness for the html-topbottom
// and html-sidebyside modes passes RenderOptions(html_layout=...)).  A bad
// layout name raises IllegalArgument, like parse_html_layout in options.
pub fn render_html(text: String, mode: usize, layout_name: String) -> Result<U, Ex> {
    let f: Flow = parse(&to_units(text))?;
    let mut f2: Flow = f;
    flow_layout(&mut f2)?;
    if mode == MODE_HTML {
        let layout: HtmlLayout = parse_html_layout(&layout_name)?;
        let opts: RenderOptions = crate::options::render_options_new();
        let mut opts2: RenderOptions = opts;
        opts2.html_layout = Some(layout);
        let model: Model = build_model(&f2)?;
        return page_on(&model, &opts2);
    }
    return render_flow(&mut f2, mode);
}

// The CLI dispatcher (DESIGN-RUST.md section 3.6 rule 3): one shared RunContext
// across the files of a run, and explicit RenderOptions (table_style for -table=,
// html_layout for -html=). api.py's render(text, mode, ctx=, options=) is this.
pub fn render_flow_ctx(f: &mut Flow, mode: usize, opts: &RenderOptions, ctx: &mut RunContext) -> Result<U, Ex> {
    if mode == MODE_TEXT || mode == MODE_XML {
        return text_on(f, mode);
    }
    if mode == MODE_PIC {
        return pic_on(f, mode);
    }
    if mode == MODE_SVG {
        return svg_on(f, mode, ctx);
    }
    if mode == MODE_VML || mode == MODE_VXML {
        return vml_on(f, mode, ctx);
    }
    if mode == MODE_GRAFFLE {
        return graffle_on(f, mode, ctx);
    }
    if mode == MODE_TABLE {
        return table_on(f, opts);
    }
    if mode == MODE_MERMAID {
        let model: Model = build_model(f)?;
        return Ok(mermaid_on(&model));
    }
    if mode == MODE_PLANTUML {
        let model: Model = build_model(f)?;
        return Ok(plantuml_on(&model));
    }
    if mode == MODE_HTML {
        let model: Model = build_model(f)?;
        return page_on(&model, opts);
    }
    return Err(exc(ErrorKind::RUNTIME, format!("unknown output mode: {}", mode)));
}

pub fn render_cli(text: String, mode: usize, opts: &RenderOptions, ctx: &mut RunContext) -> Result<U, Ex> {
    let f: Flow = parse(&to_units(text))?;
    let mut f2: Flow = f;
    flow_layout(&mut f2)?;
    return render_flow_ctx(&mut f2, mode, opts, ctx);
}
