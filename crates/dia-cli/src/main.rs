use std::{env, fs, process};

const USAGE: &str = "usage: dia [-f FROM] [-t TO] [-o OUTPUT] INPUT
formats: cetz (.typ), tikz (.tex), drawio (.drawio)";

fn main() {
    if let Err(e) = run(env::args().skip(1).collect()) {
        eprintln!("dia: {e}");
        process::exit(1);
    }
}

fn run(args: Vec<String>) -> Result<(), String> {
    let (mut from, mut to, mut output, mut input) = (None, None, None, None);
    let mut it = args.into_iter();
    while let Some(a) = it.next() {
        match a.as_str() {
            "-f" | "--from" => from = it.next(),
            "-t" | "--to" => to = it.next(),
            "-o" | "--output" => output = it.next(),
            "-h" | "--help" => return Err(USAGE.into()),
            _ => input = Some(a),
        }
    }
    let input = input.ok_or(USAGE)?;
    let from = match from {
        Some(f) => f,
        None => dia_core::format_for_path(&input)
            .map_err(|e| e.to_string())?
            .name
            .to_string(),
    };
    let to = match (to, &output) {
        (Some(t), _) => t,
        (None, Some(o)) => dia_core::format_for_path(o)
            .map_err(|e| e.to_string())?
            .name
            .to_string(),
        (None, None) => return Err("missing -t or -o".into()),
    };
    let src = fs::read_to_string(&input).map_err(|e| format!("{input}: {e}"))?;
    let mut warnings = vec![];
    let out = dia_core::convert(&src, &from, &to, &mut warnings).map_err(|e| e.to_string())?;
    for w in warnings {
        eprintln!("warning: {w}");
    }
    match output {
        Some(o) => fs::write(&o, out).map_err(|e| format!("{o}: {e}")),
        None => {
            print!("{out}");
            Ok(())
        }
    }
}
