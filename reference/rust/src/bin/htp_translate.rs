// SPDX-License-Identifier: Apache-2.0

use htp_reference::{HumanAiTransaction, TranslationLevel};
use std::env;
use std::fs;
use std::io::{self, Read};
use std::process;

#[derive(Debug)]
struct Cli {
    input: String,
    previous: Option<String>,
    level: TranslationLevel,
    strict: bool,
}

fn main() {
    if let Err(error) = run() {
        eprintln!("htp-translate: {error}");
        process::exit(2);
    }
}

fn run() -> Result<(), String> {
    let cli = parse_args(env::args().skip(1))?;
    let current = read_transaction(&cli.input)?;
    let previous = cli.previous.as_deref().map(read_transaction).transpose()?;

    if let Some(previous) = &previous {
        if previous.conversation_id != current.conversation_id {
            return Err("previous and current transactions use different conversation_id values".into());
        }
        let previous_hash = previous.witness_hash();
        if current.predecessor_witness.as_deref() != Some(previous_hash.as_str()) {
            eprintln!("warning: predecessor_witness does not match the supplied previous transaction");
        }
    }

    let validation = current.validate();
    if cli.strict && !validation.valid {
        let encoded = serde_json::to_string_pretty(&validation)
            .map_err(|error| format!("cannot encode validation report: {error}"))?;
        println!("{encoded}");
        process::exit(1);
    }

    let mut projection = current.project(cli.level, previous.as_ref());
    if cli.level == TranslationLevel::Expert {
        let transaction = serde_json::to_value(&current)
            .map_err(|error| format!("cannot encode canonical transaction: {error}"))?;
        if let Some(object) = projection.as_object_mut() {
            object.insert("transaction".to_string(), transaction);
        }
    }

    let encoded = serde_json::to_string_pretty(&projection)
        .map_err(|error| format!("cannot encode translation: {error}"))?;
    println!("{encoded}");
    Ok(())
}

fn parse_args(args: impl Iterator<Item = String>) -> Result<Cli, String> {
    let mut input = None;
    let mut previous = None;
    let mut level = TranslationLevel::Plain;
    let mut strict = false;
    let mut args = args.peekable();

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--previous" => {
                previous = Some(args.next().ok_or_else(|| "--previous requires a path".to_string())?);
            }
            "--level" => {
                let value = args.next().ok_or_else(|| "--level requires a value".to_string())?;
                level = parse_level(&value)?;
            }
            "--strict" => strict = true,
            "-h" | "--help" => {
                print_help();
                process::exit(0);
            }
            value if value.starts_with('-') && value != "-" => {
                return Err(format!("unknown option {value}"));
            }
            value => {
                if input.replace(value.to_string()).is_some() {
                    return Err("only one current transaction path may be supplied".into());
                }
            }
        }
    }

    Ok(Cli {
        input: input.ok_or_else(|| "missing transaction path; use - for stdin".to_string())?,
        previous,
        level,
        strict,
    })
}

fn parse_level(value: &str) -> Result<TranslationLevel, String> {
    match value {
        "plain" => Ok(TranslationLevel::Plain),
        "informed" => Ok(TranslationLevel::Informed),
        "expert" => Ok(TranslationLevel::Expert),
        "machine" => Ok(TranslationLevel::Machine),
        _ => Err(format!("unknown translation level {value}; expected plain, informed, expert, or machine")),
    }
}

fn read_transaction(path: &str) -> Result<HumanAiTransaction, String> {
    let content = if path == "-" {
        let mut input = String::new();
        io::stdin().read_to_string(&mut input).map_err(|error| format!("cannot read stdin: {error}"))?;
        input
    } else {
        fs::read_to_string(path).map_err(|error| format!("cannot read {path}: {error}"))?
    };
    serde_json::from_str(&content).map_err(|error| format!("invalid transaction JSON: {error}"))
}

fn print_help() {
    println!(
        "Human Translation Protocol projector\n\n\
Usage:\n  htp-translate [--level LEVEL] [--previous PATH] [--strict] <TRANSACTION.json|->\n\n\
LEVEL:\n  plain      Human-readable public summary (default)\n  informed   Evidence, authority, actions, fractures, and repairs\n  expert     Validation, portable DDC-dimensional change metadata, and change set\n  machine    Canonical protocol transaction JSON\n\n\
--strict exits with status 1 when protocol validation contains errors."
    );
}
