// LibreCN's GUI: four panels (#overview, #ues, #provisioning, #system), the
// shown one refreshed every second from the web API; #ues/{supi} shows one
// UE's details. Plain JS, no build.

const $ = (id) => document.getElementById(id);
const esc = (s) => String(s).replace(/[&<>"']/g, (c) => ({ "&": "&amp;", "<": "&lt;", ">": "&gt;", '"': "&quot;", "'": "&#39;" })[c]);
const num = (v, digits = 0) => v === undefined || v === null ? "" : Number(v).toFixed(digits);
const PROTECTED = ["log", "alarm", "control", "server"];
const PANELS = ["overview", "ues", "provisioning", "system"];

function table(el, head, rows) {
  el.innerHTML = "<tr>" + head.map(([h, cls]) => `<th class="${cls || ""}">${h}</th>`).join("") + "</tr>" +
    (rows.length ? rows.join("") : `<tr><td class="muted" colspan="${head.length}">none</td></tr>`);
}

// A "N more" row when `shown` of `count` are listed.
const more = (count, shown, cols) =>
  count > shown ? [`<tr><td class="muted" colspan="${cols}">${count - shown} more</td></tr>`] : [];

async function get(path) {
  const r = await fetch(path);
  if (!r.ok) throw new Error(path + ": " + r.status);
  return r.json();
}

// The NFs started (GET /api/nfs): each view finds its NF by kind.
let nfs = [];
const nfName = (kind) => (nfs.find((n) => n.kind === kind) || {}).name;
const nfPath = (kind, path) => {
  const name = nfName(kind);
  if (!name) throw new Error(`no ${kind.toUpperCase()} started`);
  return `/api/nfs/${encodeURIComponent(name)}/${path}`;
};

// Resource `path` of the NF of `kind`: its JSON, or an error.
async function nfGet(kind, path) {
  const r = await fetch(nfPath(kind, path));
  const body = await r.json();
  if (!r.ok) throw new Error(body.error || r.status);
  return body;
}

// An action on the NF of `kind`, its answer shown; whether it succeeded.
async function act(kind, method, path, body) {
  try {
    const r = await fetch(nfPath(kind, path), { method, body });
    showResult(await r.json());
    return r.ok;
  } catch (e) {
    showResult({ error: e.message });
    return false;
  }
}

function showResult(answer) {
  $("result-box").hidden = false;
  $("result").textContent = JSON.stringify(answer, null, 2);
}

// Refreshes view `id` with `render`, one request at a time; an error shows in its table.
const pending = {};
async function view(id, render) {
  if (pending[id]) return;
  pending[id] = true;
  try {
    await render();
  } catch (e) {
    $(id).innerHTML = `<tr><td class="muted">${esc(e.message)}</td></tr>`;
  } finally {
    pending[id] = false;
  }
}

/* ---- Rates from counters: the UPF's throughput, the workers' load. ---- */

let previous = { at: 0, busy: {}, upf: null };
let rates = null;
function measure(eos, workers, now) {
  const dt = previous.at ? now - previous.at : 0;
  const loads = {};
  for (const w of workers) {
    const key = w.pool + "/" + w.index;
    if (dt && previous.busy[key] !== undefined) loads[key] = Math.min(100, 100 * (w.busy_ms - previous.busy[key]) / dt);
    previous.busy[key] = w.busy_ms;
  }
  const upf = eos.find((e) => e.stats && e.stats.counters && "downlink_bytes" in e.stats.counters);
  const c = upf ? upf.stats.counters : null;
  if (c && previous.upf && dt) {
    const rate = (k) => Math.max(0, ((c[k] || 0) - (previous.upf[k] || 0)) * 1000 / dt);
    rates = {
      up: rate("uplink_bytes") * 8 / 1e6, down: rate("downlink_bytes") * 8 / 1e6,
      upPps: rate("uplink_packets"), downPps: rate("downlink_packets"),
      dropped: rate("dropped_packets"), limited: rate("rate_limited_packets"),
    };
  } else if (!c) {
    rates = null;
  }
  previous = { at: now, busy: previous.busy, upf: c };
  return loads;
}

const throughput = () => rates
  ? `uplink ${num(rates.up, 2)} Mbit/s (${num(rates.upPps)} packets/s), downlink ${num(rates.down, 2)} Mbit/s ` +
    `(${num(rates.downPps)} packets/s); per second ${num(rates.dropped)} dropped, ${num(rates.limited)} beyond the rates`
  : "no UPF";

/* ---- Overview: the core as in TS 23.501 §4.2.3. ---- */

const SVG = "http://www.w3.org/2000/svg";
const W = 120, H = 70, BUS = 150;
// Service-based NFs above and below the bus, then the reference points'
// row. `kind`: a LibreCN NF; `grey`: a 3GPP NF LibreCN has not; `go`: the panel a click opens.
const BOXES = [
  { id: "nssf", name: "NSSF", x: 40, y: 20, sbi: "Nnssf", grey: "Network Slice Selection Function: picks the slice instances for a UE. LibreCN's AMF and SMF check the slices they serve themselves." },
  { id: "nef", name: "NEF", x: 200, y: 20, sbi: "Nnef", grey: "Network Exposure Function: exposes the network's capabilities and events to applications." },
  { id: "nrf", name: "NRF", x: 360, y: 20, sbi: "Nnrf", grey: "NF Repository Function: NF registration and discovery. LibreCN's NFs find each other by name." },
  { id: "pcf", name: "PCF", x: 520, y: 20, sbi: "Npcf", kind: "pcf", go: "provisioning", text: "Policy Control Function: per-subscriber UE-AMBR, session AMBR and QoS flows." },
  { id: "udm", name: "UDM", x: 680, y: 20, sbi: "Nudm", kind: "udm", go: "provisioning", text: "Unified Data Management: subscribers, their keys and SQNs, SUCI de-concealment." },
  { id: "af", name: "AF", x: 840, y: 20, sbi: "Naf", grey: "Application Function: an application influencing traffic routing or policy." },
  { id: "ausf", name: "AUSF", x: 200, y: 200, sbi: "Nausf", kind: "ausf", text: "Authentication Server Function: 5G AKA and EAP-AKA'." },
  { id: "amf", name: "AMF", x: 360, y: 200, sbi: "Namf", kind: "amf", go: "ues", text: "Access and Mobility Management Function: N2 with the gNBs, N1 with the UEs: registration, security, mobility, paging." },
  { id: "smf", name: "SMF", x: 680, y: 200, sbi: "Nsmf", kind: "smf", go: "ues", text: "Session Management Function: PDU sessions, UE addresses, N4 to the UPF." },
  { id: "ue", name: "UE", x: 40, y: 380, go: "ues" },
  { id: "ran", name: "(R)AN", x: 280, y: 380, go: "ues" },
  { id: "upf", name: "UPF", x: 600, y: 380, kind: "upf", text: "User Plane Function: GTP-U on N3, the data network on N6, QoS flows and their rates." },
  { id: "dn", name: "DN", x: 840, y: 380 },
];
// Reference points: from, to (box edge points), label.
const LINKS = [
  [[420, 270], [100, 380], "N1"], [[420, 270], [340, 380], "N2"], [[740, 270], [660, 380], "N4"],
  [[160, 415], [280, 415], ""], [[400, 415], [600, 415], "N3"], [[720, 415], [840, 415], "N6"],
];
const shapes = {};
let picked = null;

function svg(parent, tag, attrs, text) {
  const el = document.createElementNS(SVG, tag);
  for (const [k, v] of Object.entries(attrs)) el.setAttribute(k, v);
  if (text !== undefined) el.textContent = text;
  parent.appendChild(el);
  return el;
}

function drawDiagram() {
  const d = $("diagram");
  svg(d, "line", { x1: 60, y1: BUS, x2: 960, y2: BUS, class: "bus" });
  for (const [[x1, y1], [x2, y2], label] of LINKS) {
    svg(d, "line", { x1, y1, x2, y2 });
    if (label) svg(d, "text", { x: (x1 + x2) / 2 + 8, y: (y1 + y2) / 2 - 6, class: "label" }, label);
  }
  for (const b of BOXES) {
    if (b.sbi) {
      const [from, to] = b.y < BUS ? [b.y + H, BUS] : [BUS, b.y];
      svg(d, "line", { x1: b.x + W / 2, y1: from, x2: b.x + W / 2, y2: to });
      svg(d, "text", { x: b.x + W / 2 + 6, y: (from + to) / 2 + 4, class: "label" }, b.sbi);
    }
    const g = svg(d, "g", { class: "box" + (b.grey ? " grey" : "") + (b.kind || b.grey || b.go ? " nf" : "") });
    svg(g, "rect", { x: b.x, y: b.y, width: W, height: H, rx: 8 });
    svg(g, "text", { x: b.x + W / 2, y: b.y + 26, class: "name" }, b.name);
    const l1 = svg(g, "text", { x: b.x + W / 2, y: b.y + 45, class: "sub" }, b.grey ? "not in LibreCN" : "");
    const l2 = svg(g, "text", { x: b.x + W / 2, y: b.y + 61 }, "");
    if (b.kind || b.grey) g.addEventListener("click", () => pick(b.id));
    else if (b.go) g.addEventListener("click", () => { location.hash = b.go; });
    shapes[b.id] = { g, l1, l2 };
  }
  // Notes under the RAN, the UEs and the DN.
  for (const id of ["ue", "ran", "dn"]) {
    const b = BOXES.find((x) => x.id === id);
    shapes[id].notes = svg(d, "g", { transform: `translate(${b.x + W / 2} ${b.y + H + 24})` });
  }
}

function notes(id, lines) {
  const g = shapes[id].notes;
  g.replaceChildren();
  lines.forEach((line, i) => svg(g, "text", { x: 0, y: i * 18, "text-anchor": "middle", class: "note" + (i ? " sub" : "") }, line));
}

function setBox(id, cls, l1, l2) {
  const s = shapes[id];
  s.g.setAttribute("class", `box nf ${cls}` + (picked === id ? " picked" : ""));
  s.l1.textContent = l1;
  s.l2.textContent = l2;
}

let eosNow = [];
const plural = (n, one, many = one + "s") => `${n} ${n === 1 ? one : many}`;
const refreshOverview = () => view("nf-table", async () => {
  // Each NF's EO and state, then what the running ones say.
  const eoOf = (kind) => eosNow.find((e) => e.name === nfName(kind));
  const state = (kind) => { const eo = eoOf(kind); return !eo ? "absent" : eo.alive ? "running" : "faulted"; };
  const ask = (kind, path) => state(kind) === "running" ? nfGet(kind, path).catch(() => null) : Promise.resolve(null);
  const [gnbs, ues, sessions, dnns, subscribers, policies] = await Promise.all([
    ask("amf", "gnbs"), ask("amf", "ues"), ask("smf", "sessions"), ask("smf", "dnns"),
    ask("udm", "subscribers"), ask("pcf", "policies"),
  ]);
  const registered = ues ? ues.registered + ues.idle : 0;
  const ausf = eoOf("ausf");
  const figure = {
    amf: ues ? `${registered} registered` : "",
    smf: sessions ? plural(sessions.count, "session") : "",
    udm: subscribers ? plural(subscribers.count, "subscriber") : "",
    pcf: policies ? plural(policies.count, "own policy", "own policies") : "",
    ausf: ausf && ausf.stats ? plural(ausf.stats.messages, "message") : "",
    upf: rates ? `↑ ${num(rates.up, 1)}  ↓ ${num(rates.down, 1)} Mbit/s` : "",
  };
  for (const b of BOXES.filter((x) => x.kind)) {
    const eo = eoOf(b.kind);
    setBox(b.id, state(b.kind), eo ? eo.name : "not started", figure[b.kind]);
  }
  setBox("ue", ues ? "running" : "absent", ues ? `${ues.connected} connected` : "", ues ? `${ues.idle} idle` : "");
  notes("ue", ues ? [`${registered} registered`] : []);
  setBox("ran", gnbs && gnbs.length ? "running" : "absent", gnbs ? plural(gnbs.length, "gNB") : "", "");
  const names = (gnbs || []).slice(0, 6).map((g) => `${g.name || g.id || "association " + g.assoc}: ${plural(g.ues, "UE")}`);
  notes("ran", names.concat(gnbs && gnbs.length > 6 ? [`${gnbs.length - 6} more`] : []));
  setBox("dn", dnns && dnns.length ? "running" : "absent", dnns ? plural(dnns.length, "DNN") : "", "");
  notes("dn", (dnns || []).map((d) => `${d.dnn} ${d.pool}: ${plural(d.sessions, "session")}`));
  renderPicked();
});

function pick(id) {
  picked = picked === id ? null : id;
  for (const [k, s] of Object.entries(shapes)) s.g.classList.toggle("picked", k === picked);
  renderPicked();
}

function renderPicked() {
  const b = BOXES.find((x) => x.id === picked);
  $("nf-detail").hidden = !b;
  if (!b) return;
  if (b.grey) {
    $("nf-title").textContent = `${b.name}: not in LibreCN`;
    $("nf-text").textContent = b.grey;
    $("nf-table").innerHTML = "";
    return;
  }
  const name = nfName(b.kind);
  const eo = name && eosNow.find((e) => e.name === name);
  $("nf-title").textContent = name ? `${b.name}: ${name}` : `${b.name}: not started`;
  $("nf-text").innerHTML = esc(b.text) + (b.go ? ` <a href="#${b.go}">See the ${b.go === "ues" ? "UEs" : "provisioning"}.</a>` : "");
  if (eo) renderEos([eo], $("nf-table"));
  else $("nf-table").innerHTML = "";
}

/* ---- UEs: gNBs, UEs, one UE's details, PDU sessions. ---- */

let gnbNames = {};
const refreshRan = () => view("ue-list", async () => {
  const [gnbs, ues] = await Promise.all([nfGet("amf", "gnbs"), nfGet("amf", "ues")]);
  gnbNames = Object.fromEntries(gnbs.map((g) => [g.assoc, g.name || g.id || `association ${g.assoc}`]));
  table($("gnbs"), [["gNB"], ["Name"], ["Tracking areas"], ["UEs connected", "num"]], gnbs.map((g) =>
    `<tr><td class="${g.id ? "" : "warn"}">${esc(g.id ?? "not set up")}</td><td>${esc(g.name ?? "")}</td>` +
    `<td>${esc(g.tas.join(", "))}</td><td class="num">${g.ues}</td></tr>`));
  $("ue-counts").textContent = `${gnbs.length} gNB(s); ${ues.registered + ues.idle} UE(s) registered: ` +
    `${ues.connected} connected (${ues.registered} registered), ${ues.idle} idle. Click a UE for its details.`;
  table($("ue-list"), [["SUPI"], ["CM"], ["5GMM"], ["5G-TMSI"], ["gNB"], ["Sessions"], ["Idle for", "num"]],
    ues.ues.map((u) => {
      const connected = u.cm === "connected";
      return `<tr class="pick" data-supi="${esc(u.supi ?? "")}"><td>${esc(u.supi ?? "unknown yet")}</td>` +
        `<td class="${connected ? "ok" : "muted"}">${u.cm}</td><td>${esc(u.state ?? "registered")}</td>` +
        `<td>${esc(u.tmsi ?? "")}</td><td>${connected ? esc(gnbNames[u.assoc] ?? "") : ""}</td>` +
        `<td>${esc(u.sessions.join(", "))}</td><td class="num">${connected ? "" : u.idle_s + " s"}</td></tr>`;
    }).concat(more(ues.connected + ues.idle, ues.ues.length, 7)));
});

const SESSION_STATE = { active: "ok", releasing: "warn" };
const ambr = (a) => a ? `${a.uplink} / ${a.downlink}` : "";
const refreshSessions = () => view("sessions", async () => {
  $("throughput").textContent = throughput();
  const body = await nfGet("smf", "sessions");
  table($("sessions"), [["Id", "num"], ["SUPI"], ["PSI", "num"], ["DNN"], ["UE address"], ["State"], ["5QI", "num"], ["AMBR Mbit/s", "num"]],
    body.sessions.map((s) =>
      `<tr class="pick" data-supi="${esc(s.supi)}"><td class="num">${s.id}</td><td>${esc(s.supi)}</td><td class="num">${s.pdu_session_id}</td>` +
      `<td>${esc(s.dnn)}</td><td>${esc(s.ue_ip)}</td><td class="${SESSION_STATE[s.state] || "muted"}">${esc(s.state)}</td>` +
      `<td class="num">${num(s.five_qi)}</td><td class="num">${ambr(s.ambr_mbps)}</td></tr>`)
      .concat(more(body.count, body.sessions.length, 8)));
});

// The UE shown in details, from the address (#ues/{supi}).
let selected = null;
function select(supi) {
  if ((supi || null) === selected) return;
  selected = supi || null;
  $("ue-detail").hidden = !selected;
  if (selected) {
    $("ue-title").textContent = selected;
    $("ue-facts").textContent = "…";
    $("ue-sessions").innerHTML = "";
    refreshUe();
    $("ue-detail").scrollIntoView({ behavior: "smooth", block: "nearest" });
  }
}

const refreshUe = () => selected && view("ue-sessions", async () => {
  const supi = selected;
  const [ue, sessions, policy] = await Promise.all([
    nfGet("amf", `ues/${encodeURIComponent(supi)}`).catch(() => null),
    nfGet("smf", `sessions?supi=${encodeURIComponent(supi)}`).catch(() => null),
    nfGet("pcf", `policies/${encodeURIComponent(supi)}`).catch(() => null),
  ]);
  if (supi !== selected) return;
  const fact = (label, value, cls) => `<div><span>${label}</span> <b class="${cls || ""}">${esc(value)}</b></div>`;
  const facts = [];
  if (!ue) {
    facts.push(fact("State", "not registered", "muted"));
  } else {
    const connected = ue.cm === "connected";
    facts.push(fact("CM", ue.cm, connected ? "ok" : "muted"), fact("5GMM", ue.state ?? "registered"), fact("5G-TMSI", ue.tmsi ?? ""));
    if (connected) facts.push(fact("gNB", gnbNames[ue.assoc] ?? `association ${ue.assoc}`), fact("AMF / RAN UE NGAP ids", `${ue.amf_ue_ngap_id} / ${ue.ran_ue_ngap_id}`));
    else facts.push(fact("Idle for", `${ue.idle_s} s`));
  }
  if (policy) facts.push(fact("UE-AMBR (Mbit/s)", ambr(policy.ue_ambr_mbps) + (policy.own ? ", its own policy" : ", the default")));
  $("ue-facts").innerHTML = facts.join("");
  table($("ue-sessions"), [["PSI", "num"], ["DNN"], ["UE address"], ["State"], ["5QI", "num"], ["Session AMBR Mbit/s", "num"], ["Extra QoS flows", "num"]],
    (sessions ? sessions.sessions : []).map((s) =>
      `<tr><td class="num">${s.pdu_session_id}</td><td>${esc(s.dnn)}</td><td>${esc(s.ue_ip)}</td>` +
      `<td class="${SESSION_STATE[s.state] || "muted"}">${esc(s.state)}</td><td class="num">${num(s.five_qi)}</td>` +
      `<td class="num">${ambr(s.ambr_mbps)}</td><td class="num">${num(s.flows)}</td></tr>`));
  $("ue-guti").disabled = $("ue-deregister").disabled = !ue;
});

/* ---- Provisioning: subscribers and policies. ---- */

const refreshSubscribers = () => view("subscribers", async () => {
  const body = await nfGet("udm", "subscribers");
  table($("subscribers"), [["SUPI"], ["Authentication"], ["AMF field"], ["SQN"], [""]], body.subscribers.map((s) =>
    `<tr><td>${esc(s.supi)}</td><td>${esc(s.auth)}</td><td>${esc(s.amf)}</td><td>${esc(s.sqn)}</td>` +
    `<td><button data-remove="${esc(s.supi)}">Remove</button></td></tr>`).concat(more(body.count, body.subscribers.length, 5)));
});

// A policy's sessions in one line: "internet: 50 / 200, 5QI 9, 1 flow".
function sessionPolicies(p) {
  return (p.sessions || []).map((s) => {
    const flows = s.flows.length ? `, ${s.flows.length} flow${s.flows.length > 1 ? "s" : ""}` : "";
    return `${esc(s.dnn ?? "any DNN")}: ${s.ambr_mbps.uplink} / ${s.ambr_mbps.downlink}, 5QI ${s.five_qi}${flows}`;
  }).join("; ");
}

let policies = {};
const refreshPolicies = () => view("policies", async () => {
  const body = await nfGet("pcf", "policies");
  policies = Object.fromEntries(body.policies.map((p) => [p.supi, p]));
  const row = (p, supi, source, remove) =>
    `<tr><td>${supi}</td><td class="muted">${source}</td>` +
    `<td class="num">${p.ue_ambr_mbps.uplink} / ${p.ue_ambr_mbps.downlink}</td><td>${sessionPolicies(p)}</td>` +
    `<td>${remove ? `<button data-edit="${esc(p.supi)}">Edit</button> <button data-remove="${esc(p.supi)}">Remove</button>` : ""}</td></tr>`;
  table($("policies"), [["SUPI"], ["From"], ["UE-AMBR", "num"], ["Sessions"], [""]],
    [row(body.default, "<i>default</i>", "configuration", false)]
      .concat(body.policies.map((p) => row(p, esc(p.supi), p.provisioned ? "web API" : "configuration", true)),
        more(body.count, body.policies.length, 5)));
});

/* ---- System: alarms, EOs, workers. ---- */

function renderAlarms(alarms) {
  const sev = { CRITICAL: "bad", MAJOR: "bad", MINOR: "warn", WARNING: "warn" };
  table($("alarms"), [["Severity"], ["Source"], ["Id", "num"], ["Text"], ["Raised"]], alarms.map((a) =>
    `<tr><td class="${sev[a.severity] || ""}">${esc(a.severity)}</td><td>${esc(a.source)}</td>` +
    `<td class="num">${a.alarm_id}</td><td>${esc(a.text)}</td>` +
    `<td>${new Date(a.raised_unix_ms).toLocaleTimeString()}</td></tr>`));
  const badge = $("alarm-badge");
  badge.hidden = !alarms.length;
  badge.textContent = `${alarms.length} alarm${alarms.length === 1 ? "" : "s"}`;
}

function renderEos(eos, el = $("eos")) {
  table(el, [["Name"], ["Kind"], ["Pool"], ["State"], ["Messages", "num"], ["Mean µs", "num"], ["Max µs", "num"],
    ["Queue", "num"], ["Queue peak", "num"], ["Rejected", "num"], ["Buffers peak/count"], ["Counters"], [""]], eos.map((e) => {
    const s = e.stats || {};
    const kind = (nfs.find((n) => n.name === e.name) || {}).kind || (PROTECTED.includes(e.name) ? "oam" : "");
    const buffers = (s.buffers || []).map((b) =>
      `<span class="${b.failures ? "bad" : b.high_water >= b.count ? "warn" : ""}">${b.size}B ${b.high_water}/${b.count}</span>`).join(" ");
    const counters = Object.entries(s.counters || {}).map(([k, v]) => `${esc(k)} ${v}`).join(", ");
    const stop = e.alive && !PROTECTED.includes(e.name) ? `<button data-stop="${e.id}">Stop</button>` : "";
    return `<tr><td>${esc(e.name)}</td><td>${esc(kind)}</td><td>${esc(e.pool)}</td>` +
      `<td class="${e.alive ? "ok" : "bad"}">${e.alive ? "running" : "faulted"}</td>` +
      `<td class="num">${num(s.messages)}</td><td class="num">${num(s.mean_us, 1)}</td><td class="num">${num(s.max_us, 1)}</td>` +
      `<td class="num">${num(s.data_depth)}</td><td class="num">${num(s.data_high_water)}</td>` +
      `<td class="num ${s.data_rejected ? "bad" : ""}">${num(s.data_rejected)}</td><td>${buffers}</td>` +
      `<td class="muted">${counters}</td><td>${stop}</td></tr>`;
  }));
}

function renderWorkers(workers, loads) {
  table($("workers"), [["Pool"], ["Worker", "num"], ["Core", "num"], ["Load", "num"], ["Messages", "num"], ["Wake-ups", "num"]],
    workers.map((w) => {
      const load = loads[w.pool + "/" + w.index];
      return `<tr><td>${esc(w.pool)}</td><td class="num">${w.index}</td><td class="num">${w.core ?? ""}</td>` +
        `<td class="num">${load === undefined ? "" : num(load) + " %"}</td>` +
        `<td class="num">${w.messages}</td><td class="num">${w.wakeups}</td></tr>`;
    }));
}

/* ---- Panels and the refresh. ---- */

const hash = () => location.hash.slice(1).split("/");
const panel = () => PANELS.includes(hash()[0]) ? hash()[0] : "overview";

function show() {
  const p = panel();
  select(p === "ues" ? decodeURIComponent(hash()[1] || "") : null);
  for (const id of PANELS) $(id).hidden = id !== p;
  for (const a of document.querySelectorAll("#tabs a")) a.classList.toggle("active", a.getAttribute("href") === "#" + p);
  refresh();
}

async function refresh() {
  try {
    const [eos, workers, alarms, list] = await Promise.all([
      get("/api/eos"), get("/api/workers"), get("/api/alarms").catch(() => []), get("/api/nfs").catch(() => []),
    ]);
    nfs = list;
    eosNow = eos;
    const loads = measure(eos, workers, performance.now());
    renderAlarms(alarms);
    switch (panel()) {
      case "overview": refreshOverview(); break;
      case "ues": refreshRan(); refreshSessions(); refreshUe(); break;
      case "provisioning": refreshSubscribers(); refreshPolicies(); break;
      default: renderEos(eos); renderWorkers(workers, loads);
    }
    $("status").textContent = "updated " + new Date().toLocaleTimeString();
    $("status").className = "muted";
  } catch (e) {
    $("status").textContent = "unreachable (" + e.message + ")";
    $("status").className = "bad";
  }
}

/* ---- Actions. ---- */

for (const id of ["ue-list", "sessions"]) {
  $(id).addEventListener("click", (ev) => {
    const row = ev.target.closest("tr[data-supi]");
    if (row && row.dataset.supi) location.hash = "ues/" + encodeURIComponent(row.dataset.supi);
  });
}
$("ue-search").addEventListener("submit", (ev) => {
  ev.preventDefault();
  location.hash = "ues/" + encodeURIComponent(new FormData(ev.target).get("supi").trim());
});
$("ue-close").addEventListener("click", () => { location.hash = "ues"; });
$("ue-guti").addEventListener("click", async () => {
  await act("amf", "POST", `ues/${encodeURIComponent(selected)}/guti`, "");
  refreshUe();
});
$("ue-deregister").addEventListener("click", async () => {
  if (!confirm(`Deregister ${selected}?`)) return;
  await act("amf", "DELETE", `ues/${encodeURIComponent(selected)}`, JSON.stringify({ reregister: $("ue-reregister").checked }));
  refreshUe();
});

$("subscribers").addEventListener("click", async (ev) => {
  const supi = ev.target.dataset.remove;
  if (!supi || !confirm(`Remove ${supi}?`)) return;
  await act("udm", "DELETE", `subscribers/${encodeURIComponent(supi)}`);
  refreshSubscribers();
});
$("add-subscriber").addEventListener("submit", async (ev) => {
  ev.preventDefault();
  const fields = Object.fromEntries([...new FormData(ev.target)].filter(([, v]) => v.trim() !== ""));
  if (await act("udm", "POST", "subscribers", JSON.stringify(fields))) ev.target.reset();
  refreshSubscribers();
});

$("policies").addEventListener("click", async (ev) => {
  const { edit, remove } = ev.target.dataset;
  if (edit && policies[edit]) {
    const { provisioned, ...policy } = policies[edit];
    $("policy").value = JSON.stringify(policy, null, 2);
    return;
  }
  if (!remove || !confirm(`Remove the policy of ${remove}? The default applies.`)) return;
  await act("pcf", "DELETE", `policies/${encodeURIComponent(remove)}`);
  refreshPolicies();
});
$("set-policy").addEventListener("click", async () => {
  await act("pcf", "POST", "policies", $("policy").value);
  refreshPolicies();
});

$("eos").addEventListener("click", async (ev) => {
  const id = ev.target.dataset.stop;
  if (!id || !confirm("Stop this EO?")) return;
  ev.target.disabled = true;
  const r = await fetch(`/api/eos/${id}/stop`, { method: "POST" });
  showResult(await r.json());
  refresh();
});
$("capacity").addEventListener("submit", async (ev) => {
  ev.preventDefault();
  const capacity = Number(new FormData(ev.target).get("relative_capacity"));
  await act("amf", "POST", "capacity", JSON.stringify({ relative_capacity: capacity }));
});
$("deploy").addEventListener("click", async () => {
  const r = await fetch("/api/nfs", { method: "POST", body: $("deployment").value });
  showResult(await r.json());
  refresh();
});

get("/api/kinds").then((k) => { $("kinds").textContent = k.join(", ") || "none"; }).catch(() => { $("kinds").textContent = "unknown"; });
drawDiagram();
window.addEventListener("hashchange", show);
show();
setInterval(refresh, 1000);
