// SPDX-License-Identifier: Apache-2.0

const HTP_V02_SIGNATURE_VERSION = "ddc-htp-signature/0.2";
const HTP_V02_ALGORITHM = "ed25519";
const HTP_V02_HASH_PREFIX = "ddc-htp-security-v0.2\0json\0";

const normalizeV01Payload = normalizePayload;
const renderV01Expert = renderExpert;
const renderV01 = render;

function htpV02Canonicalize(value) {
  if (Array.isArray(value)) return value.map(htpV02Canonicalize);
  if (value && typeof value === "object") {
    const output = {};
    for (const key of Object.keys(value).sort()) output[key] = htpV02Canonicalize(value[key]);
    return output;
  }
  return value;
}

function htpV02HexToBytes(value) {
  if (typeof value !== "string" || value.length % 2) throw new Error("Invalid hexadecimal value");
  const output = new Uint8Array(value.length / 2);
  for (let index = 0; index < output.length; index += 1) {
    const byte = Number.parseInt(value.slice(index * 2, index * 2 + 2), 16);
    if (!Number.isFinite(byte)) throw new Error("Invalid hexadecimal value");
    output[index] = byte;
  }
  return output;
}

function htpV02BytesToHex(bytes) {
  return [...new Uint8Array(bytes)].map((byte) => byte.toString(16).padStart(2, "0")).join("");
}

async function htpV02TaggedContentHash(content) {
  const canonical = JSON.stringify(htpV02Canonicalize(content));
  const bytes = new TextEncoder().encode(HTP_V02_HASH_PREFIX + canonical);
  return htpV02BytesToHex(await crypto.subtle.digest("SHA-256", bytes));
}

function htpV02CheckContentBinding(envelope, issues) {
  const content = envelope.content || {};
  if (envelope.profile === "public") {
    if (content.profile !== "public") issues.push("Public content does not declare public profile");
    if (content.protocol_version !== "ddc-htp/0.2") issues.push("Public content has wrong protocol version");
    if (content.witness_hash !== envelope.witness_hash) issues.push("Public content witness hash does not match envelope");
    if (content.stream_root !== envelope.stream_root) issues.push("Public content stream root does not match envelope");
  } else if (envelope.profile === "private") {
    if (content.protocol_version !== "ddc-htp/0.2") issues.push("Private content has wrong protocol version");
    if (content.stream_root !== envelope.stream_root) issues.push("Private content stream root does not match envelope");
    if (!content.transaction || content.transaction.protocol_version !== "ddc-htp/0.1") {
      issues.push("Private content does not contain a canonical v0.1 transaction");
    }
  } else {
    issues.push("Unknown publication profile");
  }
}

async function htpV02VerifyEnvelope(envelope) {
  const issues = [];
  if (envelope.signature_version !== HTP_V02_SIGNATURE_VERSION) issues.push("Unsupported signature version");
  if (envelope.algorithm !== HTP_V02_ALGORITHM) issues.push("Unsupported signature algorithm");
  htpV02CheckContentBinding(envelope, issues);

  try {
    const contentHash = await htpV02TaggedContentHash(envelope.content);
    if (contentHash !== envelope.content_hash) issues.push("Content hash mismatch");
  } catch (error) {
    issues.push(`Content hash unavailable: ${error.message}`);
  }

  let signedPayload = null;
  let payloadBytes = null;
  try {
    payloadBytes = htpV02HexToBytes(envelope.signed_payload_hex);
    signedPayload = JSON.parse(new TextDecoder().decode(payloadBytes));
    const expected = {
      signature_version: envelope.signature_version,
      algorithm: envelope.algorithm,
      profile: envelope.profile,
      key_id: envelope.key_id,
      public_key_hex: envelope.public_key_hex,
      witness_hash: envelope.witness_hash,
      stream_root: envelope.stream_root,
      content_hash: envelope.content_hash
    };
    for (const [key, value] of Object.entries(expected)) {
      if (signedPayload[key] !== value) issues.push(`Signed payload mismatch: ${key}`);
    }
  } catch (error) {
    issues.push(`Signed payload invalid: ${error.message}`);
  }

  if (issues.length) return { status: "failed", issues };
  if (typeof crypto === "undefined" || !crypto.subtle || !payloadBytes) {
    return { status: "unavailable", issues: ["WebCrypto unavailable"] };
  }

  try {
    const publicKey = await crypto.subtle.importKey(
      "raw",
      htpV02HexToBytes(envelope.public_key_hex),
      { name: "Ed25519" },
      false,
      ["verify"]
    );
    const valid = await crypto.subtle.verify(
      { name: "Ed25519" },
      publicKey,
      htpV02HexToBytes(envelope.signature_hex),
      payloadBytes
    );
    return valid
      ? { status: "verified", issues: [] }
      : { status: "failed", issues: ["Ed25519 signature verification failed"] };
  } catch (error) {
    return { status: "unavailable", issues: [`Browser Ed25519 verification unavailable: ${error.message}`] };
  }
}

function htpV02PublicTransaction(content) {
  return {
    protocol_version: content.transaction_protocol_version || "ddc-htp/0.1",
    conversation_id: content.conversation_id,
    turn_id: content.turn_id,
    predecessor_witness: content.predecessor_witness ?? null,
    need: {
      statement: "[redacted by public publication profile]",
      objective: content.need?.objective || "",
      constraints: []
    },
    projection: {
      interpretation: content.projection?.interpretation || "",
      assumptions: [],
      ambiguities: content.projection?.ambiguities || []
    },
    authority: content.authority || { required: [], granted: [], denied: [] },
    evidence: values(content.evidence).map((item) => ({
      id: item.id,
      kind: item.kind,
      statement: item.statement,
      source: item.source_digest ? `source digest ${item.source_digest}` : "[redacted source]",
      status: item.status
    })),
    actions: values(content.actions).map((item) => ({
      id: item.id,
      description: item.description,
      requires: item.requires || [],
      status: item.status,
      effects: values(item.affected_dimensions).map((dimension) => ({ dimension, boundary: "[redacted]" })),
      resource_delta: {},
      reversible: item.reversible ?? null
    })),
    witness: content.witness || { conclusion: "", recommendation: null, evidence_ids: [], action_ids: [] },
    fractures: content.fractures || [],
    repairs: content.repairs || []
  };
}

function htpV02NormalizeLiveContent(payload, signed) {
  const content = payload.content;
  const transaction = payload.profile === "private" && content.transaction?.protocol_version
    ? content.transaction
    : htpV02PublicTransaction(content);
  return {
    transaction,
    expert: {
      witness_hash: payload.witness_hash,
      change: content.change || null,
      dimensional_boundary: {
        vocabulary: "ddc-eight-dimensions",
        mode: "portable-htp",
        private_crystalline_closure_included: false
      },
      __live: content,
      __signatureEnvelope: signed ? payload : null
    },
    signature: signed
      ? { status: "checking", issues: [] }
      : { status: "unsigned", issues: ["Development witness is not cryptographically signed"] }
  };
}

normalizePayload = function normalizePayloadV02(payload) {
  if (payload?.signature_version === HTP_V02_SIGNATURE_VERSION && payload?.content) {
    return htpV02NormalizeLiveContent(payload, true);
  }
  if (payload?.unsigned === true && payload?.content) {
    return htpV02NormalizeLiveContent(payload, false);
  }
  return normalizeV01Payload(payload);
};

function htpV02ParseCapture(text) {
  try {
    return JSON.parse(text);
  } catch {
    const lines = text
      .split(/\r?\n/)
      .map((line) => line.trim())
      .filter((line) => line && !line.startsWith("#"));
    if (!lines.length) throw new Error("The selected capture contains no JSON witnesses.");
    return JSON.parse(lines.at(-1));
  }
}

loadJson = async function loadJsonV02(file, target) {
  const payload = htpV02ParseCapture(await file.text());
  const normalized = normalizePayload(payload);
  if (normalized.expert?.__signatureEnvelope) {
    normalized.signature = await htpV02VerifyEnvelope(normalized.expert.__signatureEnvelope);
  }
  if (target === "current") currentRecord = normalized;
  else previousRecord = normalized;
  render();
};

renderExpert = function renderExpertV02(t, previous, expert) {
  const base = renderV01Expert(t, previous, expert);
  const live = expert?.__live;
  if (!live) return base;
  const envelope = expert.__signatureEnvelope;
  const signature = currentRecord.expert === expert ? currentRecord.signature : previousRecord?.signature;
  const status = signature?.status || "checking";
  const issueText = values(signature?.issues).join(" · ") || "Envelope metadata, content hash and signature agree.";
  return `${base}
    <h3 class="section-title">${envelope ? "Signed live witness" : "Live development witness"}</h3>
    <div class="grid-3">
      <article class="card"><p class="eyebrow">Signature</p><h2>${esc(status)}</h2><p>${esc(issueText)}</p></article>
      <article class="card"><p class="eyebrow">Publication</p><h2>${esc(envelope?.key_id || "unsigned")}</h2><p>${esc(envelope?.algorithm || "no signature")} · ${esc(envelope?.profile || "development")} profile</p></article>
      <article class="card"><p class="eyebrow">Event provenance</p><h2>${values(live.event_receipts).length} receipts</h2><p>Stream root ${esc(envelope?.stream_root || live.stream_root || "unavailable")}</p></article>
    </div>`;
};

render = function renderV02() {
  renderV01();
  if (currentRecord.signature) {
    const meta = document.querySelector("#witnessMeta");
    const status = currentRecord.signature.status;
    meta.innerHTML += ` · <strong>signature ${esc(status)}</strong>`;
  }
};

render();
