// SPDX-License-Identifier: Apache-2.0

use htp_reference::{
    publish_snapshot, ConversationEvent, IngressTrustPolicy, LiveConversationAssembler,
    PublicationProfile, SignedWitnessEnvelope,
};
use serde_json::json;
use std::env;
use std::fs::File;
use std::io::{self, BufRead, BufReader};
use std::process;

const SIGNING_KEY_ENV: &str = "HTP_SIGNING_KEY_HEX";
const LEGACY_SIGNING_KEY_ENV: &str = "DDC_HTP_SIGNING_KEY_HEX";

#[derive(Debug)]
struct Cli {
    input: String,
    trust_policy: String,
    profile: PublicationProfile,
    key_id: String,
    unsigned: bool,
}

fn main() {
    if let Err(error) = run() {
        eprintln!("htp-live: {error}");
        process::exit(2);
    }
}

fn run() -> Result<(), String> {
    let cli = parse_args(env::args().skip(1))?;
    if cli.input == cli.trust_policy && cli.input == "-" {
        return Err("event stream and trust policy cannot both use stdin".to_string());
    }

    let policy_file = File::open(&cli.trust_policy)
        .map_err(|error| format!("cannot open trust policy {}: {error}", cli.trust_policy))?;
    let policy: IngressTrustPolicy = serde_json::from_reader(BufReader::new(policy_file))
        .map_err(|error| format!("invalid trust policy JSON: {error}"))?;
    if policy.sources.is_empty() {
        return Err("trust policy contains no trusted sources".to_string());
    }

    let signing_key = if cli.unsigned {
        None
    } else {
        Some(read_signing_key()?)
    };

    let mut assembler = LiveConversationAssembler::new(policy);
    if cli.input == "-" {
        let stdin = io::stdin();
        process_stream(
            stdin.lock(),
            &mut assembler,
            cli.profile,
            &cli.key_id,
            signing_key.as_deref(),
        )?;
    } else {
        let file = File::open(&cli.input)
            .map_err(|error| format!("cannot open event stream {}: {error}", cli.input))?;
        process_stream(
            BufReader::new(file),
            &mut assembler,
            cli.profile,
            &cli.key_id,
            signing_key.as_deref(),
        )?;
    }

    if assembler.has_active_turn() {
        return Err("event stream ended with an incomplete active turn".to_string());
    }
    Ok(())
}

fn read_signing_key() -> Result<String, String> {
    match env::var(SIGNING_KEY_ENV) {
        Ok(value) => Ok(value),
        Err(_) => env::var(LEGACY_SIGNING_KEY_ENV).map_err(|_| {
            format!(
                "{SIGNING_KEY_ENV} is required unless --unsigned is used (legacy {LEGACY_SIGNING_KEY_ENV} is also accepted)"
            )
        }),
    }
}

fn process_stream<R: BufRead>(
    reader: R,
    assembler: &mut LiveConversationAssembler,
    profile: PublicationProfile,
    key_id: &str,
    signing_key: Option<&str>,
) -> Result<(), String> {
    for (index, line) in reader.lines().enumerate() {
        let line_number = index + 1;
        let line =
            line.map_err(|error| format!("cannot read event line {line_number}: {error}"))?;
        if line.trim().is_empty() || line.trim_start().starts_with('#') {
            continue;
        }
        let event: ConversationEvent = serde_json::from_str(&line)
            .map_err(|error| format!("invalid event JSON on line {line_number}: {error}"))?;
        match assembler.ingest(event) {
            Ok(Some(snapshot)) => {
                let output = if let Some(secret) = signing_key {
                    serde_json::to_value(
                        SignedWitnessEnvelope::sign(&snapshot, profile, key_id, secret)
                            .map_err(|error| format!("cannot sign witness: {error}"))?,
                    )
                    .map_err(|error| format!("cannot serialize signed witness: {error}"))?
                } else {
                    json!({
                        "unsigned": true,
                        "profile": profile,
                        "witness_hash": snapshot.witness_hash(),
                        "stream_root": snapshot.stream_root,
                        "content": publish_snapshot(&snapshot, profile),
                    })
                };
                println!(
                    "{}",
                    serde_json::to_string(&output)
                        .map_err(|error| format!("cannot serialize witness output: {error}"))?
                );
            }
            Ok(None) => {}
            Err(error) => return Err(format!("event line {line_number} rejected: {error}")),
        }
    }
    Ok(())
}

fn parse_args(args: impl Iterator<Item = String>) -> Result<Cli, String> {
    let mut input = None;
    let mut trust_policy = None;
    let mut profile = PublicationProfile::Public;
    let mut key_id = "local-htp-signer".to_string();
    let mut unsigned = false;
    let mut args = args.peekable();

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--trust-policy" => {
                trust_policy = Some(
                    args.next()
                        .ok_or_else(|| "--trust-policy requires a JSON path".to_string())?,
                );
            }
            "--profile" => {
                let value = args
                    .next()
                    .ok_or_else(|| "--profile requires public or private".to_string())?;
                profile = match value.as_str() {
                    "public" => PublicationProfile::Public,
                    "private" => PublicationProfile::Private,
                    _ => return Err(format!("unknown publication profile {value}")),
                };
            }
            "--key-id" => {
                key_id = args
                    .next()
                    .ok_or_else(|| "--key-id requires a value".to_string())?;
            }
            "--unsigned" => unsigned = true,
            "-h" | "--help" => {
                print_help();
                process::exit(0);
            }
            value if value.starts_with('-') && value != "-" => {
                return Err(format!("unknown option {value}"))
            }
            value => {
                if input.replace(value.to_string()).is_some() {
                    return Err("only one event stream path may be supplied".to_string());
                }
            }
        }
    }

    Ok(Cli {
        input: input.unwrap_or_else(|| "-".to_string()),
        trust_policy: trust_policy.ok_or_else(|| "--trust-policy is required".to_string())?,
        profile,
        key_id,
        unsigned,
    })
}

fn print_help() {
    println!(
        "Human Translation Protocol v0.2 live ingestor\n\n\
Usage:\n  htp-live --trust-policy POLICY.json [OPTIONS] [EVENTS.jsonl|-]\n\n\
OPTIONS:\n  --profile public|private   Publication/redaction profile (default: public)\n  --key-id ID                Signer key identifier (default: local-htp-signer)\n  --unsigned                 Emit unsigned development witnesses\n\n\
Signed mode reads the 32-byte Ed25519 private seed as 64 hexadecimal characters\nfrom HTP_SIGNING_KEY_HEX. For migration compatibility DDC_HTP_SIGNING_KEY_HEX is\nalso accepted. The key is never accepted as a command-line argument.\n\n\
The input is JSON Lines. One event is processed immediately per line, and one witness\nis emitted immediately whenever an ai_response event completes a turn."
    );
}
