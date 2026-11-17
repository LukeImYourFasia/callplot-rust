// The callplot-prep command (DESIGN-RUST.md section 3.6: the 5-line binary).
use std::env::args_os;
use std::io::stdin;
use std::io::stdout;
use std::io::stderr;
use std::process::ExitCode;

pub fn main() -> ExitCode {
    let args: Vec<_> = args_os().skip(1).collect();
    let code = callplot::cli::run_prep(&args, &mut stdin().lock(), &mut stdout().lock(), &mut stderr().lock());
    ExitCode::from(code)
}
