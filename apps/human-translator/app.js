// SPDX-License-Identifier: Apache-2.0

const DEMO_PREVIOUS = {
  protocol_version: "ddc-htp/0.1",
  conversation_id: "demo-update",
  turn_id: "turn-1",
  predecessor_witness: null,
  need: {
    statement: "Can I safely install this update?",
    objective: "Decide whether installing the update creates unacceptable risk",
    constraints: ["Do not change the system without permission"]
  },
  projection: {
    interpretation: "Evaluate compatibility, security, and rollback risk",
    assumptions: ["The installed operating system is Ubuntu 24.04"],
    ambiguities: ["Plugin X compatibility has not been verified"]
  },
  authority: { required: [], granted: [], denied: [] },
  evidence: [
    {
      id: "e-os",
      kind: "system_observation",
      statement: "The system reports Ubuntu 24.04",
      source: "local system inventory",
      status: "established"
    },
    {
      id: "e-vendor",
      kind: "source",
      statement: "The update supports Ubuntu 24.04",
      source: "vendor compatibility documentation",
      status: "established"
    },
    {
      id: "e-plugin-open",
      kind: "source",
      statement: "Plugin X compatibility has not yet been verified",
      source: "compatibility check pending",
      status: "unknown"
    }
  ],
  actions: [],
  witness: {
    conclusion: "The base system is compatible, but Plugin X remains unverified",
    recommendation: "Verify Plugin X before installing",
    evidence_ids: ["e-os", "e-vendor", "e-plugin-open"],
    action_ids: []
  },
  fractures: [
    {
      id: "f-plugin",
      dimension: "semantic",
      summary: "Plugin X compatibility is unresolved",
      severity: "blocking",
      evidence_ids: ["e-plugin-open"]
    }
  ],
  repairs: []
};

const DEMO_CURRENT = {
  protocol_version: "ddc-htp/0.1",
  conversation_id: "demo-update",
  turn_id: "turn-2",
  predecessor_witness: "demo-linked-witness",
  need: {
    statement: "Can I safely install this update?",
    objective: "Decide whether installing the update creates unacceptable risk",
    constraints: ["Do not change the system without permission"]
  },
  projection: {
    interpretation: "Evaluate compatibility, security, and rollback risk",
    assumptions: ["The installed operating system is Ubuntu 24.04"],
    ambiguities: []
  },
  authority: { required: [], granted: [], denied: [] },
  evidence: [
    {
      id: "e-os",
      kind: "system_observation",
      statement: "The system reports Ubuntu 24.04",
      source: "local system inventory",
      status: "established"
    },
    {
      id: "e-vendor",
      kind: "source",
      statement: "The update supports Ubuntu 24.04",
      source: "vendor compatibility documentation",
      status: "established"
    },
    {
      id: "e-plugin-doc",
      kind: "source",
      statement: "Plugin X supports only versions through 3.7",
      source: "Plugin X compatibility documentation",
      status: "established"
    }
  ],
  actions: [],
  witness: {
    conclusion: "Do not install yet because Plugin X is not compatible with the new version",
    recommendation: "Wait for a compatible Plugin X release",
    evidence_ids: ["e-os", "e-vendor", "e-plugin-doc"],
    action_ids: []
  },
  fractures: [
    {
      id: "f-plugin",
      dimension: "semantic",
      summary: "Plugin X compatibility was unresolved",
      severity: "blocking",
      evidence_ids: ["e-plugin-doc"]
    }
  ],
  repairs: [
    {
      fracture_id: "f-plugin",
      summary: "Current Plugin X documentation resolved the compatibility question",
      evidence_ids: ["e-plugin-doc"]
    }
  ]
};

const DIMENSIONS = ["semantic", "authority", "state", "resource", "security", "physical", "frequency", "lineage"];
const $ = (selector) => document.querySelector(selector);
const $$ = (selector) => [...document.querySelectorAll(selector)];

let currentRecord = { transaction: structuredClone(DEMO_CURRENT), expert: null };
let previousRecord = { transaction: structuredClone(DEMO_PREVIOUS), expert: null };
let activeLevel = "plain";

function esc(value) {
  return String(value ?? "")
    .replaceAll("&", "&amp;")
    .replaceAll("<", "&lt;")
    .replaceAll(">", "&gt;")
    .replaceAll('"', "&quot;")
    .replaceAll("'", "&#039;");
}

function values(value) {
  return Array.isArray(value) ? value : value ? [...value] : [];
}

function repairedIds(t) {
  return new Set(values(t.repairs).map((repair) => repair.fracture_id));
}

function unresolvedFractures(t) {
  const repaired = repairedIds(t);
  return values(t.fractures).filter((fracture) => !repaired.has(fracture.id));
}

function missingAuthority(t) {
  const granted = new Set(values(t.authority?.granted));
  return values(t.authority?.required).filter((item) => !granted.has(item));
}

function protocolState(t) {
  if (!t.witness?.conclusion?.trim()) return "open";
  if (unresolvedFractures(t).some((item) => item.severity === "blocking")) return "fractured";
  const denied = new Set(values(t.authority?.denied));
  if (values(t.authority?.required).some((item) => denied.has(item))) return "blocked";
  if (missingAuthority(t).length) return "awaiting_authority";
  return "closed";
}

function evidenceCounts(t) {
  const counts = { established: 0, inferred: 0, unknown: 0, contradicted: 0 };
  for (const item of values(t.evidence)) counts[item.status] = (counts[item.status] || 0) + 1;
  return counts;
}

function transactionDiff(current, previous) {
  if (!previous) return null;
  const priorEvidence = new Set(values(previous.evidence).map((item) => item.id));
  const currentEvidence = new Set(values(current.evidence).map((item) => item.id));
  const priorFractures = new Set(values(previous.fractures).map((item) => item.id));
  const currentFractures = new Set(values(current.fractures).map((item) => item.id));
  const currentRepaired = repairedIds(current);
  const priorGrants = new Set(values(previous.authority?.granted));
  const currentGrants = new Set(values(current.authority?.granted));

  return {
    conclusionChanged: previous.witness?.conclusion !== current.witness?.conclusion,
    previousConclusion: previous.witness?.conclusion || "",
    currentConclusion: current.witness?.conclusion || "",
    newEvidence: [...currentEvidence].filter((id) => !priorEvidence.has(id)),
    newFractures: [...currentFractures].filter((id) => !priorFractures.has(id)),
    resolvedFractures: [...priorFractures].filter((id) => !currentFractures.has(id) || currentRepaired.has(id)),
    authorityAdded: [...currentGrants].filter((id) => !priorGrants.has(id)),
    authorityRemoved: [...priorGrants].filter((id) => !currentGrants.has(id))
  };
}

function stableJson(value) {
  return JSON.stringify(value ?? null);
}

function executedEffects(t, dimension) {
  return values(t.actions)
    .filter((action) => action.status === "executed")
    .flatMap((action) => values(action.effects).filter((effect) => effect.dimension === dimension));
}

function resourceSnapshot(t) {
  const total = {};
  for (const action of values(t.actions).filter((item) => item.status === "executed")) {
    for (const [key, delta] of Object.entries(action.resource_delta || {})) total[key] = (total[key] || 0) + delta;
  }
  return total;
}

function dimensionHints(current, previous) {
  if (!previous) return new Set();
  const changed = new Set();
  if (stableJson([current.need, current.projection, current.witness?.conclusion, current.witness?.recommendation]) !==
      stableJson([previous.need, previous.projection, previous.witness?.conclusion, previous.witness?.recommendation])) changed.add("semantic");
  if (stableJson(current.authority) !== stableJson(previous.authority)) changed.add("authority");
  if (protocolState(current) !== protocolState(previous)) changed.add("state");
  if (stableJson(resourceSnapshot(current)) !== stableJson(resourceSnapshot(previous))) changed.add("resource");
  if (stableJson(executedEffects(current, "security")) !== stableJson(executedEffects(previous, "security"))) changed.add("security");
  if (stableJson(executedEffects(current, "physical")) !== stableJson(executedEffects(previous, "physical"))) changed.add("physical");
  if (values(current.evidence).length !== values(previous.evidence).length || values(current.actions).length !== values(previous.actions).length ||
      values(current.fractures).length !== values(previous.fractures).length || values(current.repairs).length !== values(previous.repairs).length) changed.add("frequency");
  if (stableJson([current.predecessor_witness, current.evidence, current.repairs, current.witness?.evidence_ids, current.witness?.action_ids]) !==
      stableJson([previous.predecessor_witness, previous.evidence, previous.repairs, previous.witness?.evidence_ids, previous.witness?.action_ids])) changed.add("lineage");
  return changed;
}

function canonicalDimensions(expert) {
  const changed = expert?.change?.changed_dimensions || expert?.ddc_closure?.by_dimension;
  return changed ? new Set(Object.keys(changed)) : null;
}

function evidenceCard(item) {
  return `<div class="item">
    <div class="item-row"><strong>${esc(item.statement)}</strong><span class="status-pill ${esc(item.status)}">${esc(item.status)}</span></div>
    <p>${esc(item.source)} · ${esc(item.kind)}</p>
    <code>${esc(item.id)}</code>
  </div>`;
}

function listOrEmpty(items, renderer) {
  return items.length ? `<div class="list">${items.map(renderer).join("")}</div>` : `<p class="empty">None recorded.</p>`;
}

function renderPlain(t) {
  const counts = evidenceCounts(t);
  const unknowns = values(t.evidence).filter((item) => item.status === "unknown");
  const fractures = unresolvedFractures(t);
  const missing = missingAuthority(t);
  return `
    <div class="grid-2">
      <article class="card accent"><p class="eyebrow">Human need</p><h2>You want</h2><p>${esc(t.need?.objective)}</p></article>
      <article class="card accent"><p class="eyebrow">AI projection</p><h2>AI understood</h2><p>${esc(t.projection?.interpretation)}</p></article>
    </div>
    <article class="hero-conclusion">
      <p class="eyebrow">Public witness</p>
      <h2>${esc(t.witness?.conclusion)}</h2>
      <p>${esc(t.witness?.recommendation || "No recommendation recorded.")}</p>
    </article>
    <div class="status-row">
      <span class="status-pill established">${counts.established} established</span>
      <span class="status-pill inferred">${counts.inferred} inferred</span>
      <span class="status-pill unknown">${counts.unknown} unknown</span>
      <span class="status-pill contradicted">${counts.contradicted} contradicted</span>
      <span class="status-pill">${fractures.length} unresolved fracture${fractures.length === 1 ? "" : "s"}</span>
      <span class="status-pill">${missing.length} authority missing</span>
    </div>
    <div class="grid-2">
      <section><h3 class="section-title">Unknown / unresolved</h3>${listOrEmpty(unknowns, evidenceCard)}</section>
      <section><h3 class="section-title">Fractures</h3>${listOrEmpty(fractures, (item) => `<div class="item"><strong>${esc(item.summary)}</strong><p>${esc(item.dimension)} · ${esc(item.severity)}</p></div>`)}</section>
    </div>`;
}

function renderInformed(t) {
  return `
    <div class="grid-3">
      <article class="card"><p class="eyebrow">Need</p><h2>${esc(t.need?.objective)}</h2><p>${esc(t.need?.statement)}</p></article>
      <article class="card"><p class="eyebrow">Projection</p><h2>${esc(t.projection?.interpretation)}</h2><p>${values(t.projection?.assumptions).map(esc).join(" · ") || "No assumptions recorded."}</p></article>
      <article class="card"><p class="eyebrow">Authority</p><h2>${missingAuthority(t).length ? "Incomplete" : "Satisfied"}</h2><p>Required: ${values(t.authority?.required).map(esc).join(", ") || "none"}</p></article>
    </div>
    <article class="hero-conclusion"><p class="eyebrow">Witness</p><h2>${esc(t.witness?.conclusion)}</h2><p>${esc(t.witness?.recommendation || "No recommendation recorded.")}</p></article>
    <h3 class="section-title">Evidence</h3>${listOrEmpty(values(t.evidence), evidenceCard)}
    <div class="grid-2">
      <section><h3 class="section-title">Actions</h3>${listOrEmpty(values(t.actions), (item) => `<div class="item"><div class="item-row"><strong>${esc(item.description)}</strong><span class="mini-chip">${esc(item.status)}</span></div><p>Requires: ${values(item.requires).map(esc).join(", ") || "none"} · Reversible: ${item.reversible == null ? "unknown" : item.reversible}</p></div>`)}</section>
      <section><h3 class="section-title">Fracture → Repair</h3>${listOrEmpty(values(t.fractures), (item) => {
        const repair = values(t.repairs).find((candidate) => candidate.fracture_id === item.id);
        return `<div class="item"><strong>${esc(item.summary)}</strong><p>${repair ? `Repaired: ${esc(repair.summary)}` : "Unresolved"}</p></div>`;
      })}</section>
    </div>`;
}

function renderExpert(t, previous, expert) {
  const hints = dimensionHints(t, previous);
  const canonical = canonicalDimensions(expert);
  const dimensions = DIMENSIONS.map((dimension) => {
    const changed = canonical ? canonical.has(dimension) : hints.has(dimension);
    return `<div class="dimension ${changed ? "changed" : ""}">${esc(dimension)}</div>`;
  }).join("");
  const validation = expert?.validation;
  const boundary = expert?.dimensional_boundary;
  return `
    <div class="grid-3">
      <article class="card"><p class="eyebrow">Conversation</p><h2>${esc(t.conversation_id)}</h2><p>Turn ${esc(t.turn_id)}</p></article>
      <article class="card"><p class="eyebrow">Predecessor</p><h2>${t.predecessor_witness ? "Linked" : "Root"}</h2><p>${esc(t.predecessor_witness || "No predecessor witness")}</p></article>
      <article class="card"><p class="eyebrow">Validation</p><h2>${validation ? (validation.valid ? "Valid" : "Invalid") : "Browser projection"}</h2><p>${validation ? `${validation.issues?.length || 0} validation issue(s)` : "Load expert CLI output for canonical validation."}</p></article>
    </div>
    <article class="card accent" style="margin-top:12px"><p class="eyebrow">DDC dimensional vocabulary</p><h2>${canonical ? "Canonical HTP change summary" : "Non-authoritative browser hints"}</h2><div class="dimension-grid">${dimensions}</div>${boundary ? `<p style="margin-top:10px">${esc(boundary.mode)} · private Crystalline closure included: ${esc(boundary.private_crystalline_closure_included)}</p>` : ""}</article>
    <div class="grid-2">
      <section><h3 class="section-title">Authority boundary</h3><div class="item"><strong>Required</strong><p>${values(t.authority?.required).map(esc).join(", ") || "none"}</p></div><div class="item"><strong>Granted</strong><p>${values(t.authority?.granted).map(esc).join(", ") || "none"}</p></div></section>
      <section><h3 class="section-title">Witness lineage</h3><div class="item"><strong>Evidence references</strong><p>${values(t.witness?.evidence_ids).map(esc).join(", ") || "none"}</p></div><div class="item"><strong>Action references</strong><p>${values(t.witness?.action_ids).map(esc).join(", ") || "none"}</p></div></section>
    </div>
    ${expert ? `<h3 class="section-title">Canonical expert envelope</h3><pre class="machine">${esc(JSON.stringify({witness_hash: expert.witness_hash, change: expert.change, dimensional_boundary: expert.dimensional_boundary}, null, 2))}</pre>` : ""}`;
}

function renderMachine(t) {
  return `<pre class="machine">${esc(JSON.stringify(t, null, 2))}</pre>`;
}

function renderChange(current, previous, expert) {
  const panel = $("#changeView");
  const flag = $("#changeFlag");
  if (!previous) {
    flag.textContent = "root";
    flag.classList.add("stable");
    panel.innerHTML = `<p class="empty">No predecessor transaction loaded.</p>`;
    return;
  }
  const change = transactionDiff(current, previous);
  const changed = change.conclusionChanged || change.newEvidence.length || change.newFractures.length || change.resolvedFractures.length || change.authorityAdded.length || change.authorityRemoved.length;
  flag.textContent = changed ? "changed" : "stable";
  flag.classList.toggle("stable", !changed);
  const canonical = canonicalDimensions(expert);
  const canonicalList = canonical ? [...canonical] : null;
  const visualDimensions = [...dimensionHints(current, previous)];
  panel.innerHTML = `
    <div class="change-block"><h3>Before</h3><p>${esc(change.previousConclusion)}</p></div>
    <div class="change-block"><h3>Now</h3><p>${esc(change.currentConclusion)}</p></div>
    <div class="change-block"><h3>New evidence</h3>${change.newEvidence.length ? `<ul>${change.newEvidence.map((id) => `<li>${esc(id)}</li>`).join("")}</ul>` : `<p class="empty">No new evidence IDs.</p>`}</div>
    <div class="change-block"><h3>Fracture state</h3>${change.resolvedFractures.length ? `<ul>${change.resolvedFractures.map((id) => `<li>Resolved ${esc(id)}</li>`).join("")}</ul>` : `<p class="empty">No fracture was resolved.</p>`}</div>
    <div class="change-block"><h3>${canonicalList ? "HTP dimensional change" : "Dimension hints"}</h3><p>${esc((canonicalList || visualDimensions).join(" · ") || "No dimensional change detected")}</p></div>`;
}

function render() {
  const t = currentRecord.transaction;
  const previous = previousRecord?.transaction || null;
  const primary = $("#primaryView");
  if (activeLevel === "plain") primary.innerHTML = renderPlain(t);
  if (activeLevel === "informed") primary.innerHTML = renderInformed(t);
  if (activeLevel === "expert") primary.innerHTML = renderExpert(t, previous, currentRecord.expert);
  if (activeLevel === "machine") primary.innerHTML = renderMachine(t);

  const state = protocolState(t);
  const stateChip = $("#stateChip");
  stateChip.textContent = state.replaceAll("_", " ");
  stateChip.className = `state-chip ${state}`;
  $("#witnessMeta").innerHTML = `<strong>${esc(t.conversation_id)}</strong> · ${esc(t.turn_id)} · ${values(t.evidence).length} evidence records · ${values(t.actions).length} actions`;
  renderChange(t, previous, currentRecord.expert);
}

function normalizePayload(payload) {
  if (payload?.transaction?.protocol_version) return { transaction: payload.transaction, expert: payload };
  if (payload?.protocol_version && payload?.need && payload?.witness) return { transaction: payload, expert: null };
  throw new Error("Expected a canonical ddc-htp transaction or expert envelope containing transaction.");
}

async function loadJson(file, target) {
  const payload = JSON.parse(await file.text());
  const normalized = normalizePayload(payload);
  if (target === "current") currentRecord = normalized;
  else previousRecord = normalized;
  render();
}

$$('.level').forEach((button) => button.addEventListener("click", () => {
  activeLevel = button.dataset.level;
  $$('.level').forEach((item) => item.classList.toggle("active", item === button));
  render();
}));

$("#loadPrevious").addEventListener("click", () => $("#previousFile").click());
$("#loadCurrent").addEventListener("click", () => $("#currentFile").click());
$("#previousFile").addEventListener("change", (event) => event.target.files[0] && loadJson(event.target.files[0], "previous").catch((error) => alert(error.message)));
$("#currentFile").addEventListener("change", (event) => event.target.files[0] && loadJson(event.target.files[0], "current").catch((error) => alert(error.message)));
$("#resetDemo").addEventListener("click", () => {
  currentRecord = { transaction: structuredClone(DEMO_CURRENT), expert: null };
  previousRecord = { transaction: structuredClone(DEMO_PREVIOUS), expert: null };
  render();
});
$("#copyView").addEventListener("click", async () => {
  const text = `${$("#primaryView").innerText}\n\n${$("#changeView").innerText}`;
  try {
    await navigator.clipboard.writeText(text);
    $("#copyView").textContent = "Copied";
    setTimeout(() => { $("#copyView").textContent = "Copy visible view"; }, 1200);
  } catch {
    alert("Clipboard access is unavailable in this browser context.");
  }
});

render();
