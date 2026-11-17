# callplot-rust — work state (persisted)

Date: 2026-11-17. Keep this file updated as work completes; delete finished
items from "Next moves". Read-only toward callplot-python/callplot-js test data.

## Objective
Port the `callplot` sequence-diagram utility (design docs at repo root) to Rust
in `/home/edgarvil/develop/callplot-rust`, producing byte-identical outputs vs
the Java oracle / JS / Python ports.

## Requirements (invariants)
- DESIGN.md = full behavior contract; DESIGN-PYTHON.md = companion-port pattern;
  DESIGN-RUST.md (Version 2) = the Rust-port design authority.
- Outputs byte-for-byte vs goldens; new outputs must equal the JS-approved
  snapshots (ports compare to existing snapshots, never write their own).
- UTF-16 code-unit string semantics internally (`U = Vec<u16>`); prep is BYTES
  (`Vec<u8>`); pure core, zero runtime deps (stdlib only), no `unsafe`.
- Validation protocol (from `callplot-python/tests/conftest.py` + `test_cli.py`):
  if `<name>.exit` is 0 compare stdout bytes; if non-0, render() must just raise
  (stdout NOT compared); stderr is NEVER compared; exit codes 0/1/255
  (255 = TableInputError cases - port only needs to raise).
- Snapshot protocol (`tests/golden/test_snapshots.py`): ext `.html-topbottom.html`
  / `.html-sidebyside.html` forces `Mode.HTML` + `RenderOptions(html_layout=...)`;
  a throw must match an EMPTY snapshot (`got = b""`).
- Nothing committed/pushed (repo root is not a git workspace).

## Decisions (pinned)
- Port from the *finished* Python implementation `callplot-python/src/callplot/`
  as reference, cross-checked vs DESIGN.md/DESIGN-RUST.md; DESIGN-RUST.md governs
  Rust shape/rules (workspace `crates/callplot` + `crates/callplot-cli` 3 bins +
  xtask, `JInt`/`JDouble`, exhaustive `match Mode` - restructure still DEFERRED).
- On-disk core keeps `Result<T, Ex>` with `Ex{kind,msg}`, 8 `ErrorKind` UPPER_SNAKE
  variants, `U = Vec<u16}` text storage (single edition-2021 package).
- CLI shape (DESIGN-RUST 3.6): ONE crate, shared `src/cli.rs` with
  `run_callplot/run_plottable/run_prep(args: &[OsString], si/so/se: &mut dyn
  Read/Write) -> u8`, three 5-line `src/bin` mains, usage texts by
  `include_str!` from `src/usage.txt` (3866 B, tabs kept, generated from the
  Python `USAGE`) and `src/plottable_usage.txt` (293 B, byte-equal to
  `test/cli/plottable_usage.expected`).
- Toolchain: the `rust-1.97/env.sh` shim is gone. Use the rustup GNU libc toolchain:
  `export HOME=/home/edgarvil; export T=$HOME/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/bin; PATH="$T:$PATH" RUSTC="$T/rustc" cargo build|test`.
  Every other rustup toolchain here is a musl build and cannot exec (`/lib/ld-musl-x86_64.so.1`
  is absent). NEVER run `cargo fmt`/`rustfmt` from that toolchain: it re-sorts the `use`
  blocks and churns all 376 files; the tree is formatted as-is.
- rustc quirks (pinned): `&str` literals need `.to_string()` for `String` params;
  no `+` between `&str` and `String` (use `format!`); index access yields VALUES
  (no `*` on `bytes[k]`), iterator elements of `Vec<u8>` need `*b`; move-out-of-Vec
  by index needs `.clone()`; `match` arms that produce values drop the trailing
  `;`; casts `as usize`/`as i64`; unicode escapes `\u{...}` (no bare `\uXXXX`).
- std APIs pinned by probes: `std::env::args_os`, `std::ffi::OsString`
  (`OsString(String)` ctor, `to_str()` -> `Option<&str>`), `std::io::stdin/stdout/stderr`
  + `.lock()`, `std::io::Read`/`std::io::Write` traits (for `read_to_end`/`write_all`/`flush`),
  `std::fs::read`/`std::fs::write` (accept `&OsString` and `&String`), `std::process::id`,
  `std::process::ExitCode::from(code)`, `std::panic::catch_unwind` with `|| {...}`,
  `std::io::ErrorKind as IoKind` + `IoKind::BrokenPipe`, `std::time::Instant` +
  `elapsed().as_millis()`, `std::fs::read_dir` (DirEntry.path() -> PathBuf,
  `p.to_str()` -> `Option<&str>`; no `name()`; PathBuf is not `Display`).
- This std has NO unwind-safe wrapper for closures that capture streams (no
  `Uncategorized`-style helper): `catch_unwind` wraps ONLY the pure render
  computes; stream writes stay outside and map `BrokenPipe` -> exit 0, quiet.
- A SECOND writer (parallel `opencode serve` agent) edits the same `src/*.rs`
  files - re-read before editing, never overwrite wholesale.

## Work State
### Completed
- Foundation (errors/constants/context/jcompat) compiles; jcompat pins verified
  against the Python oracle (`/tmp/opencode/pin3.out` == `oracle_pins.py`).
- Library compiles GREEN (warnings only). Modules: errors, constants, context,
  jcompat, escape, model, options, flow, parser, ir, text, pic, mermaid, plantuml,
  svg, vml, graffle, api, table, jsnum, html_layout, html_style, html_svg,
  html_page, prep, cli.
- Golden validation FULLY GREEN: 702/702 (9 core modes 378 + 6 table styles x
  54 = 324 incl raise cases) via `/tmp/opencode/golden_run.py` — the oracle tree
  was bumped to DESIGN.md V12 / DESIGN-PYTHON V2 on 2026-11-15 (fixture
  `utf8_surrogate_bytes` + its 13 goldens, generated in callplot-js branch v12
  by `tools/gen-golden.sh` on this machine's java 1.8.0_512; JDK_VERSION pins
  _504 and all 697 pre-existing goldens reverified byte-identical, only Xalan
  .err stderr drifted and was restored; 4 new snapshots via snapshot:update).
  The golden/snapshot drivers now decode fixtures with the same
  ed-surrogate rule as the CLI (java_utf8 helper in the driver).
- SNAPSHOT layer FULLY GREEN: 216/216 (196 byte-equal + 20 throw-matches) via
  `/tmp/opencode/snapshot_run.py`.
- PREP FULLY GREEN: 8/8 (`/tmp/opencode/prep_run.py`); `src/prep.rs` bytes in/out,
  one `if` per awk rule, `awk_fields` on 0x20/0x09/0x0A only, `awk_num` per table.
- Diagram smoke 3/3: MODE_TEXT renders of `api`/`internal`/`journey` `.msd` equal
  `docs/diagrams/*.txt` byte for byte.
- CLI layer COMPLETE and validated 16/16 through the BUILT binaries
  (`/tmp/opencode/cli_run.py`, mirrors `tests/cli/test_cli.py` incl `typed_path`
  fixture mapping, `plottable_debug`/`prep_stdin` in a temp dir, `epipe` through
  `/bin/bash -c "... | head -n 1; exit ${PIPESTATUS[0]}"`):
  8 golden-CLI cases byte-equal + exit codes, `table_default` ==
  `test/golden/s_fred_annot.table-archtable.out`, `plottable_usage` byte-equal,
  `plottable_debug` side file `_x_<pid>.xml` == `s_sip_annot.xml.out`,
  `epipe` exit 0 with empty stderr.
  Root cause of the 3 CLI failures fixed this session: `-table`, `-table=` and
  `-html`/`-html=` must also SET THE MODE (cli.rs now sets MODE_TABLE / MODE_HTML,
  as cli.py does), otherwise the run stayed in TEXT mode.
- Trap tests COMPLETE: `tests/traps.rs` (the only tests/ file) 28/28 GREEN via
  `cargo test --test traps` in debug (5.3 s), named per DESIGN-RUST 4.4
  (`trap01_jtrim_nbsp_kept` ... `trap28_decode_java_utf8_three_byte_leads`).
- REFERENCE layer FULLY GREEN: 85/85 (5 samples x 17 outputs, incl the `trace_sip`
  input read from `samples/reference/trace_sip/trace_sip.msd` and the 6 `table-*.html`
  styles + 2 `html-*.html` layouts) via `/tmp/opencode/reference_run.py`
  (generates `src/bin/genref.rs`), mirroring `tests/golden/test_reference.py`.
- BUG FIX found by trap 8: `jcompat::idiv` returned `INT_MIN / -1 == 2147483648`
  (outside Java int) - now `wrap_i32(a) / wrap_i32(b)` wrapped back into the int
  range via the new `wrap_i32` helper, so `INT_MIN / -1 == INT_MIN` (DESIGN-RUST
  3.3 `wrapping_div` rule). Re-verified: no golden contains `2147483648`, and all
  689 goldens, 212 snapshots, 8 prep, 3 diagrams, 16 CLI cases stayed green.

- ASTRAL/UTF-16 decode fix (2026-11-17, exposed by the V12 fuzz corpus while the
  js port was fuzzing): `cli.rs::decode_java_utf8` had TWO wrong branches, both
  pinned by the corpus and by the oracle. (a) 4-byte leads `F0..F4` went through
  the 3-byte branch, so `f0 9f 98 80` became U+107D8 + U+FFFD (cp from 3 bytes,
  `0xE0` offset, `k += 3`); now its own branch: cp from all four bytes, `F0` wants
  `90..BF` and `F4` wants `80..8F`, a bad continuation restarts that byte fresh,
  a truncated good prefix gives one U+FFFD (trap27). (b) that branch also demanded
  a THIRD continuation byte, so the complete U+260E in the `unicode` fixture became
  U+FFFD; now exactly two continuations (trap28). The writer side already matched
  the JDK: `jcompat::from_units` maps a lone surrogate to '?' (0x3F), which is what
  a java 1.8 PrintStream writes when a label banner splits an astral pair. The 41
  astral corpus cases (of 2000 seed-1 cases) now agree byte-for-byte in all 7 CLI
  modes, and the CLI-driven golden/prep/snapshot/CLI layers stayed green (702/8/216/16).

### Active
- (nothing in flight).

### Blocked
- (none).

## Deviations and design errata (record honestly)
- `--debug-file` OVERWRITES the file: this std has no append API
  (`fs::append*` absent). DESIGN-RUST 3.8 rule 9 / DESIGN.md Appendix B say the
  awk wrote with `>> debug`; tests never re-run the same debug file, so the
  overwrite is unobservable in the oracles. Logged as an Appendix-B erratum.
- DESIGN-RUST 4.4 row 22 writes `jsplit_unescaped_slash("///") == []`; BOTH oracles
  (Java `String.split("(?<=[^\\\\])/")` and Node) give `["/"]` after Java's trailing-
  empty drop, and the 689 goldens pin that; `trap22` pins the ORACLE (`["/"]`), and
  the `""` row pins `[""]`. Row 22's first cell looks like a doc typo.
- Trap 9 (JInt `MAX + 1` wraps) cannot be tested yet: the `JInt`/`JDouble` newtypes
  of DESIGN-RUST 3.3 are a deferred step of this port, so `trap09_parse_java_int_keeps_the_java_int_range`
  pins the Java int boundary the port actually enforces. Traps 6-11 are tested
  through the current helpers (`idiv`, `fdiv`, `fmt_d`), not the future newtypes.
- Trap 4: `awk_fields` pads NF to 4 (the port's own convention), so the test counts
  NON-EMPTY fields (1) instead of list length.

## Next Move
1. Optional restructure to the DESIGN-RUST workspace (`crates/callplot` +
   `crates/callplot-cli` + xtask dist-check/fuzz), `JInt`/`JDouble` newtypes, and
   libtest-mimic golden/snapshot/reference/CLI harnesses in `tests/` (needs dev deps ->
   verify network first). Until then `src/bin/{gencases,gensnap,genprep,genref,probe_diagrams}.rs`
   are the dev drivers, and `cargo install` would also install them - a reason to
   do the restructure before shipping.
2. Fuzz (DESIGN-RUST 4.6): needs the Java harness + `tools/jref.sh`; never write
   Java. (`crates/callplot` +
   `crates/callplot-cli` + xtask dist-check/fuzz), `JInt`/`JDouble` newtypes, and
   libtest-mimic golden/snapshot/CLI harnesses in `tests/` (needs dev deps ->
   verify network first). Until then `src/bin/{gencases,gensnap,genprep,probe_diagrams}.rs`
   are the dev drivers, and `cargo install` would also install them - a reason to
   do the restructure before shipping.
3. Fuzz (DESIGN-RUST 4.6): needs the Java harness + `tools/jref.sh`; never write
   Java.

## Relevant Files
- `/home/edgarvil/develop/callplot-rust/src/` + `src/lib.rs` (now 26 modules incl
  `cli`), `src/cli.rs` (run_callplot/run_plottable/run_prep + `decode_java_utf8`
  walker + `wbytes`/`drain_err`), `src/bin/{callplot,plottable,callplot-prep}.rs`
  (thin mains), `src/{usage.txt,plottable_usage.txt,style.css}`,
  `tests/traps.rs`, `src/bin/{gencases,gensnap,genprep,genref,probe_diagrams}.rs`
  (scratch probes deleted; shared with the parallel writer).
- `/home/edgarvil/develop/callplot-python/src/callplot/{cli,plottable,prep_cli}.py`:
  exact CLI oracles; `callplot-python/test/cli/` (16 `*.json` + `plottable_usage.expected`
  + `inputs/layout_topbottom`), `test/golden/cli/` (8 `{out,exit,err}`),
  `test/golden/s_fred_annot.table-archtable.out`, `s_sip_annot.table-htmltable3.out`,
  `s_sip_annot.xml.out`, `trace_sip.prep.out` - read-only.
- `/home/edgarvil/develop/DESIGN-RUST.md` (Version 2): authority. Sections used:
  3.2 (read_lines/decode tables), 3.3 (fmt_d 19 rows), 3.4 (built-ins ban),
  3.6 (CLI rules 1-12 + plottable/prep), 3.7 (jsnum), 3.8 (awk_num 12 rows),
  4.4 (its 26 required traps, plus 27 and 28 that the astral corpus added), 4.5 (CLI cases).
- Drivers (current, all green): `/tmp/opencode/rs_golden_run.py` drives the BUILT binaries
  over 702 goldens + 8 prep + 216 snapshots; `/tmp/opencode/rs_cli_run.py` the 16 CLI
  cases (incl the `_x_<pid>.xml` side file and the `| head -n 1` pipe case);
  `/tmp/opencode/decode_battery.py` compares `decode_java_utf8` against the oracle on 37
  hand-picked byte sequences. Trap runner: `cargo test --test traps` (28).
  The older `golden_run.py`/`snapshot_run.py`/`cli_run.py`/`reference_run.py` scripts were
  dropped from /tmp; `src/bin/{genref,gensnap,genprep,gencases,probe_diagrams}.rs` still
  generate the expectations the oracle layers pin.

## Important Context
- Shell starts fresh each call: `export HOME=/home/edgarvil; source /home/edgarvil/.local/opt/rust-1.97/env.sh`;
  forgetting source -> "cargo: command not found". No std docs offline; discover
  APIs from compiler errors (the `/usr/src/rustc-.../` paths they cite do not exist on disk).
- Harness transcript display mangles a few tokens (`||`, `UnwindSafe`, some trait
  names show as phrases); the write tool lands exactly what is emitted - verify
  disk truth with `python3 repr()` reads when in doubt.
- CLI raise path: `callplot: {ErrorName}: {msg}` (msg = `ex.msg` or `""`), exit 1;
  unreadable file: `callplot: {file}: {io error Display}` + `\n`, exit 1 (stderr is
  never compared, so Display text is free but must end with `\n` for the person).
- jcompat facts: `fmt_d` non-throwing ("NaN"/"Infinity"/"-Infinity"); `idiv` now
  wraps into the Java int range (trap 8), `/0` -> Arithmetic "/ by zero";
  `parse_java_int` strict `[+-]?[0-9]+` int32; `java_split_slash` never splits at
  position 0 and drops trailing empties; CharBuf cells start `' '`,
  NegativeArraySize for a negative width.
- Two js_rounds: `jcompat::js_round` (legacy `(v+0.5).floor()` as i64) and
  `jsnum::js_round` (JS Math.round, f64) - HTML geometry uses the jsnum one.
- Golden `.err` files contain JVM traces + "Picked up JAVA_TOOL_OPTIONS..." -
  never compared; port stderr format is `callplot: {Name}: {msg}`.
