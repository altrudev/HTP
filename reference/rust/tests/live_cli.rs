// SPDX-License-Identifier: Apache-2.0

use htp_reference::SignedWitnessEnvelope;
use std::fs;
use std::process::Command;
use tempfile::tempdir;

const SECRET: &str = "000102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f";

fn fixture() -> (tempfile::TempDir, String, String) {
    let dir = tempdir().unwrap();
    let policy = dir.path().join("policy.json");
    let events = dir.path().join("events.jsonl");
    fs::write(
        &policy,
        r#"{
  "sources": {
    "human-ui": "human",
    "assistant": "assistant",
    "system": "system"
  }
}"#,
    )
    .unwrap();
    fs::write(
        &events,
        concat!(
            "{\"event_id\":\"e1\",\"source_id\":\"human-ui\",\"conversation_id\":\"cli-conv\",\"turn_id\":\"turn-1\",\"actor\":\"human\",\"payload\":{\"type\":\"human_message\",\"statement\":\"Can I install this update?\",\"objective\":\"Decide safely\",\"constraints\":[\"Do not modify without permission\"]}}\n",
            "{\"event_id\":\"e2\",\"source_id\":\"assistant\",\"conversation_id\":\"cli-conv\",\"turn_id\":\"turn-1\",\"actor\":\"assistant\",\"payload\":{\"type\":\"ai_projection\",\"interpretation\":\"Check the observed system first\",\"assumptions\":[],\"ambiguities\":[]}}\n",
            "{\"event_id\":\"e3\",\"source_id\":\"system\",\"conversation_id\":\"cli-conv\",\"turn_id\":\"turn-1\",\"actor\":\"system\",\"payload\":{\"type\":\"evidence\",\"id\":\"sys\",\"kind\":\"system_observation\",\"statement\":\"Ubuntu 24.04 is installed\",\"source\":\"local inventory\",\"status\":\"established\"}}\n",
            "{\"event_id\":\"e4\",\"source_id\":\"assistant\",\"conversation_id\":\"cli-conv\",\"turn_id\":\"turn-1\",\"actor\":\"assistant\",\"payload\":{\"type\":\"ai_response\",\"conclusion\":\"The observed OS is compatible\",\"recommendation\":\"Request install authority before changing anything\",\"evidence_ids\":[\"sys\"],\"action_ids\":[]}}\n"
        ),
    )
    .unwrap();
    (
        dir,
        policy.to_string_lossy().into_owned(),
        events.to_string_lossy().into_owned(),
    )
}

#[test]
fn unsigned_cli_emits_one_public_live_witness() {
    let (_dir, policy, events) = fixture();
    let output = Command::new(env!("CARGO_BIN_EXE_htp-live"))
        .args(["--trust-policy", &policy, "--profile", "public", "--unsigned", &events])
        .output()
        .unwrap();
    assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
    let stdout = String::from_utf8(output.stdout).unwrap();
    let lines: Vec<_> = stdout.lines().collect();
    assert_eq!(lines.len(), 1);
    let value: serde_json::Value = serde_json::from_str(lines[0]).unwrap();
    assert_eq!(value["unsigned"], true);
    assert_eq!(value["profile"], "public");
    assert_eq!(value["content"]["protocol_version"], "ddc-htp/0.2");
    assert_eq!(value["content"]["witness"]["conclusion"], "The observed OS is compatible");
}

#[test]
fn signed_cli_emits_a_verifiable_envelope() {
    let (_dir, policy, events) = fixture();
    let output = Command::new(env!("CARGO_BIN_EXE_htp-live"))
        .args(["--trust-policy", &policy, "--profile", "public", "--key-id", "cli-test", &events])
        .env("HTP_SIGNING_KEY_HEX", SECRET)
        .output()
        .unwrap();
    assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
    let first_line = output.stdout.split(|byte| *byte == b'\n').next().unwrap();
    let envelope: SignedWitnessEnvelope = serde_json::from_slice(first_line).unwrap();
    assert_eq!(envelope.key_id, "cli-test");
    assert!(envelope.verify().valid);
}
