const API = "http://localhost:3000/api";
async function req(path, opts) {
  const res = await fetch(`${API}${path}`, {
    headers: { "Content-Type": "application/json" },
    ...opts
  });
  if (!res.ok) throw new Error(`HTTP ${res.status}: ${await res.text()}`);
  return res.json();
}
const rateMatrices = {
  list: (params) => {
    const qs = new URLSearchParams();
    if (params?.status) qs.set("status", params.status);
    if (params?.currency) qs.set("currency", params.currency);
    if (params?.risk_key) qs.set("risk_key", params.risk_key);
    const q = qs.toString();
    return req(`/rate-matrices${q ? `?${q}` : ""}`);
  },
  get: (id) => req(`/rate-matrices/${id}`),
  /** Upload via FormData multipart */
  create: (form) => fetch(`${API}/rate-matrices`, { method: "POST", body: form }).then(async (r) => {
    if (!r.ok) throw new Error(`HTTP ${r.status}: ${await r.text()}`);
    return r.json();
  }),
  update: (id, data) => req(`/rate-matrices/${id}`, {
    method: "PUT",
    body: JSON.stringify(data)
  }),
  delete: (id) => fetch(`${API}/rate-matrices/${id}`, { method: "DELETE" })
};
const portfolios = {
  list: () => req("/portfolios"),
  get: (id) => req(`/portfolios/${id}`),
  create: (data) => req("/portfolios", { method: "POST", body: JSON.stringify(data) }),
  update: (id, data) => req(`/portfolios/${id}`, { method: "PUT", body: JSON.stringify(data) }),
  delete: (id) => fetch(`${API}/portfolios/${id}`, { method: "DELETE" }),
  addVector: (id, vectorId) => fetch(`${API}/portfolios/${id}/vectors`, { method: "POST", headers: { "Content-Type": "application/json" }, body: JSON.stringify({ id: vectorId }) }),
  removeVector: (id, vectorId) => fetch(`${API}/portfolios/${id}/vectors/${vectorId}`, { method: "DELETE" }),
  addSchedule: (id, scheduleId) => fetch(`${API}/portfolios/${id}/schedules`, { method: "POST", headers: { "Content-Type": "application/json" }, body: JSON.stringify({ id: scheduleId }) }),
  removeSchedule: (id, scheduleId) => fetch(`${API}/portfolios/${id}/schedules/${scheduleId}`, { method: "DELETE" }),
  createPair: (id, data) => req(`/portfolios/${id}/pairs`, { method: "POST", body: JSON.stringify(data) }),
  deletePair: (id, pairId) => fetch(`${API}/portfolios/${id}/pairs/${pairId}`, { method: "DELETE" })
};
const outstandingVectors = {
  list: () => req("/outstanding-vectors"),
  get: (id) => req(`/outstanding-vectors/${id}`),
  create: (form) => fetch(`${API}/outstanding-vectors`, { method: "POST", body: form }).then(async (r) => {
    if (!r.ok) throw new Error(`HTTP ${r.status}: ${await r.text()}`);
    return r.json();
  }),
  update: (id, data) => req(`/outstanding-vectors/${id}`, { method: "PUT", body: JSON.stringify(data) }),
  delete: (id) => fetch(`${API}/outstanding-vectors/${id}`, { method: "DELETE" })
};
const amortSchedules = {
  list: () => req("/amort-schedules"),
  get: (id) => req(`/amort-schedules/${id}`),
  create: (form) => fetch(`${API}/amort-schedules`, { method: "POST", body: form }).then(async (r) => {
    if (!r.ok) throw new Error(`HTTP ${r.status}: ${await r.text()}`);
    return r.json();
  }),
  update: (id, data) => req(`/amort-schedules/${id}`, { method: "PUT", body: JSON.stringify(data) }),
  delete: (id) => fetch(`${API}/amort-schedules/${id}`, { method: "DELETE" })
};
const executions = {
  list: () => req("/executions"),
  get: (id) => req(`/executions/${id}`),
  create: (data) => req("/executions", { method: "POST", body: JSON.stringify(data) }),
  delete: (id) => fetch(`${API}/executions/${id}`, { method: "DELETE" })
};
const studies = {
  list: () => req("/studies"),
  get: (id) => req(`/studies/${id}`),
  create: (data) => req("/studies", { method: "POST", body: JSON.stringify(data) }),
  update: (id, data) => req(`/studies/${id}`, { method: "PUT", body: JSON.stringify(data) }),
  delete: (id) => fetch(`${API}/studies/${id}`, { method: "DELETE" }),
  addUnit: (id, data) => req(`/studies/${id}/units`, { method: "POST", body: JSON.stringify(data) }),
  removeUnit: (id, unitId) => fetch(`${API}/studies/${id}/units/${unitId}`, { method: "DELETE" })
};
const studyUnits = {
  list: () => req("/study-units"),
  get: (id) => req(`/study-units/${id}`),
  create: (data) => req("/study-units", { method: "POST", body: JSON.stringify(data) }),
  update: (id, data) => req(`/study-units/${id}`, { method: "PUT", body: JSON.stringify(data) }),
  delete: (id) => fetch(`${API}/study-units/${id}`, { method: "DELETE" }),
  validate: (id) => req(`/study-units/${id}/validate`, { method: "POST" }),
  createAssignment: (id, data) => req(`/study-units/${id}/assignments`, {
    method: "POST",
    body: JSON.stringify(data)
  }),
  updateAssignment: (id, aid, data) => req(`/study-units/${id}/assignments/${aid}`, {
    method: "PUT",
    body: JSON.stringify(data)
  }),
  deleteAssignment: (id, aid) => fetch(`${API}/study-units/${id}/assignments/${aid}`, { method: "DELETE" })
};
const hypercubes = {
  list: (params) => {
    const qs = new URLSearchParams();
    if (params?.status) qs.set("status", params.status);
    const q = qs.toString();
    return req(`/hypercubes${q ? `?${q}` : ""}`);
  },
  get: (id) => req(`/hypercubes/${id}`),
  create: (data) => req("/hypercubes", {
    method: "POST",
    body: JSON.stringify(data)
  }),
  update: (id, data) => req(`/hypercubes/${id}`, {
    method: "PUT",
    body: JSON.stringify(data)
  }),
  delete: (id) => fetch(`${API}/hypercubes/${id}`, { method: "DELETE" }),
  combinations: (id) => req(`/hypercubes/${id}/combinations`)
};
export {
  amortSchedules as a,
  studyUnits as b,
  executions as e,
  hypercubes as h,
  outstandingVectors as o,
  portfolios as p,
  rateMatrices as r,
  studies as s
};
