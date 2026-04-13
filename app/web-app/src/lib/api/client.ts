const API = 'http://localhost:3000/api';

async function req<T>(path: string, opts?: RequestInit): Promise<T> {
  const res = await fetch(`${API}${path}`, {
    headers: { 'Content-Type': 'application/json' },
    ...opts,
  });
  if (!res.ok) throw new Error(`HTTP ${res.status}: ${await res.text()}`);
  return res.json() as Promise<T>;
}

// ── Types — Module 1 : Matrices de taux ──────────────────────────────────────

export interface RiskType {
  key: string;
  label: string;
  description?: string;
}

export interface MatrixRow {
  date: string;
  period_type: 'observed' | 'contrafactual' | 'projected';
  values: number[];
}

/** Vue résumée (liste) */
export interface RateMatrixSummary {
  id: string;
  name: string;
  description?: string;
  currency?: string;
  status: 'draft' | 'active' | 'archived';
  interp_method: 'linear' | 'cubic' | 'flat_forward';
  tenors: string[];
  row_count: number;
  date_from?: string;
  date_to?: string;
  created_at: string;
  risks: string[];
}

/** Vue complète (détail) */
export interface RateMatrixDetail {
  id: string;
  name: string;
  description?: string;
  currency?: string;
  status: 'draft' | 'active' | 'archived';
  interp_method: 'linear' | 'cubic' | 'flat_forward';
  tenors: string[];
  rows: MatrixRow[];
  created_at: string;
  risks: string[];
}

export interface UpdateRateMatrixRequest {
  name?: string;
  description?: string;
  currency?: string;
  status?: string;
  interp_method?: string;
  risks?: string[];
}

// ── API — Risk types ──────────────────────────────────────────────────────────

export const riskTypes = {
  list: () => req<RiskType[]>('/risk-types'),
};

// ── Types — Module 2 : Hypercubes ────────────────────────────────────────────

export interface HypercubeSummary {
  id: string;
  name: string;
  description?: string;
  start_date: string;
  end_date: string;
  proj_end_date?: string;
  time_granularity: 'monthly';
  status: 'draft' | 'active' | 'archived';
  created_at: string;
  matrix_count: number;
  combination_count: number;
}

export interface HypercubeMatrixRef {
  id: string;
  name: string;
  currency?: string;
  status: string;
  risks: string[];
  date_from?: string;
  date_to?: string;
}

export interface HypercubeDetail {
  id: string;
  name: string;
  description?: string;
  start_date: string;
  end_date: string;
  proj_end_date?: string;
  time_granularity: 'monthly';
  status: 'draft' | 'active' | 'archived';
  created_at: string;
  matrices: HypercubeMatrixRef[];
  combination_count: number;
}

export interface Combination {
  matrix_ids: string[];
  matrix_names: string[];
  risks_covered: string[];
}

export interface CreateHypercubeRequest {
  name: string;
  description?: string;
  start_date: string;
  end_date: string;
  proj_end_date?: string;
  time_granularity?: string;
  status?: string;
  matrix_ids: string[];
}

export interface UpdateHypercubeRequest {
  name?: string;
  description?: string;
  start_date?: string;
  end_date?: string;
  proj_end_date?: string;
  time_granularity?: string;
  status?: string;
  matrix_ids?: string[];
}

// ── API — Rate matrices ───────────────────────────────────────────────────────

export const rateMatrices = {
  list: (params?: { status?: string; currency?: string; risk_key?: string }) => {
    const qs = new URLSearchParams();
    if (params?.status)   qs.set('status',   params.status);
    if (params?.currency) qs.set('currency', params.currency);
    if (params?.risk_key) qs.set('risk_key', params.risk_key);
    const q = qs.toString();
    return req<RateMatrixSummary[]>(`/rate-matrices${q ? `?${q}` : ''}`);
  },

  get: (id: string) => req<RateMatrixDetail>(`/rate-matrices/${id}`),

  /** Upload via FormData multipart */
  create: (form: FormData) =>
    fetch(`${API}/rate-matrices`, { method: 'POST', body: form }).then(async (r) => {
      if (!r.ok) throw new Error(`HTTP ${r.status}: ${await r.text()}`);
      return r.json() as Promise<RateMatrixDetail>;
    }),

  update: (id: string, data: UpdateRateMatrixRequest) =>
    req<RateMatrixDetail>(`/rate-matrices/${id}`, {
      method: 'PUT',
      body: JSON.stringify(data),
    }),

  delete: (id: string) =>
    fetch(`${API}/rate-matrices/${id}`, { method: 'DELETE' }),
};

// ── Types — Module 3 : Portfolios ────────────────────────────────────────────

export interface VectorRow {
  date: string;
  period_type: string;
  value: number;
}

export interface ScheduleRow {
  date: string;
  period_type: string;
  buckets: number[];
}

export interface VectorSummary {
  id: string;
  name: string;
  description?: string;
  row_count: number;
  date_from?: string;
  date_to?: string;
  created_at: string;
}

export interface VectorDetail {
  id: string;
  name: string;
  description?: string;
  rows: VectorRow[];
  created_at: string;
}

export interface ScheduleSummary {
  id: string;
  name: string;
  description?: string;
  /** "stock" = book existant | "new_production" = nouvelles originations */
  schedule_type: 'stock' | 'new_production';
  bucket_labels: string[];
  row_count: number;
  date_from?: string;
  date_to?: string;
  created_at: string;
}

export interface ScheduleDetail {
  id: string;
  name: string;
  description?: string;
  /** "stock" = book existant | "new_production" = nouvelles originations */
  schedule_type: 'stock' | 'new_production';
  bucket_labels: string[];
  rows: ScheduleRow[];
  created_at: string;
}

export interface PairInfo {
  id: string;
  vector_id: string;
  vector_name: string;
  schedule_id: string;
  schedule_name: string;
  label?: string;
}

export interface PortfolioSummary {
  id: string;
  name: string;
  description?: string;
  created_at: string;
  vector_count: number;
  schedule_count: number;
  pair_count: number;
}

export interface PortfolioDetail {
  id: string;
  name: string;
  description?: string;
  created_at: string;
  vectors: VectorSummary[];
  schedules: ScheduleSummary[];
  pairs: PairInfo[];
}

// ── API — Portfolios ──────────────────────────────────────────────────────────

export const portfolios = {
  list: () => req<PortfolioSummary[]>('/portfolios'),
  get:  (id: string) => req<PortfolioDetail>(`/portfolios/${id}`),
  create: (data: { name: string; description?: string }) =>
    req<PortfolioDetail>('/portfolios', { method: 'POST', body: JSON.stringify(data) }),
  update: (id: string, data: { name?: string; description?: string }) =>
    req<PortfolioDetail>(`/portfolios/${id}`, { method: 'PUT', body: JSON.stringify(data) }),
  delete: (id: string) => fetch(`${API}/portfolios/${id}`, { method: 'DELETE' }),

  addVector:      (id: string, vectorId: string) =>
    fetch(`${API}/portfolios/${id}/vectors`,   { method: 'POST', headers: { 'Content-Type': 'application/json' }, body: JSON.stringify({ id: vectorId }) }),
  removeVector:   (id: string, vectorId: string) =>
    fetch(`${API}/portfolios/${id}/vectors/${vectorId}`, { method: 'DELETE' }),
  addSchedule:    (id: string, scheduleId: string) =>
    fetch(`${API}/portfolios/${id}/schedules`, { method: 'POST', headers: { 'Content-Type': 'application/json' }, body: JSON.stringify({ id: scheduleId }) }),
  removeSchedule: (id: string, scheduleId: string) =>
    fetch(`${API}/portfolios/${id}/schedules/${scheduleId}`, { method: 'DELETE' }),

  createPair: (id: string, data: { vector_id: string; schedule_id: string; label?: string }) =>
    req<PairInfo>(`/portfolios/${id}/pairs`, { method: 'POST', body: JSON.stringify(data) }),
  deletePair:  (id: string, pairId: string) =>
    fetch(`${API}/portfolios/${id}/pairs/${pairId}`, { method: 'DELETE' }),
};

// ── API — Outstanding vectors ─────────────────────────────────────────────────

export const outstandingVectors = {
  list: () => req<VectorSummary[]>('/outstanding-vectors'),
  get:  (id: string) => req<VectorDetail>(`/outstanding-vectors/${id}`),
  create: (form: FormData) =>
    fetch(`${API}/outstanding-vectors`, { method: 'POST', body: form }).then(async r => {
      if (!r.ok) throw new Error(`HTTP ${r.status}: ${await r.text()}`);
      return r.json() as Promise<VectorDetail>;
    }),
  update: (id: string, data: { name?: string; description?: string }) =>
    req<VectorDetail>(`/outstanding-vectors/${id}`, { method: 'PUT', body: JSON.stringify(data) }),
  delete: (id: string) => fetch(`${API}/outstanding-vectors/${id}`, { method: 'DELETE' }),
};

// ── API — Amortization schedules ──────────────────────────────────────────────

export const amortSchedules = {
  list: () => req<ScheduleSummary[]>('/amort-schedules'),
  get:  (id: string) => req<ScheduleDetail>(`/amort-schedules/${id}`),
  create: (form: FormData) =>
    fetch(`${API}/amort-schedules`, { method: 'POST', body: form }).then(async r => {
      if (!r.ok) throw new Error(`HTTP ${r.status}: ${await r.text()}`);
      return r.json() as Promise<ScheduleDetail>;
    }),
  update: (id: string, data: { name?: string; description?: string }) =>
    req<ScheduleDetail>(`/amort-schedules/${id}`, { method: 'PUT', body: JSON.stringify(data) }),
  delete: (id: string) => fetch(`${API}/amort-schedules/${id}`, { method: 'DELETE' }),
};

// ── Types — Module 6 : Executions ────────────────────────────────────────────

export interface ExecutionSummary {
  id: string;
  study_id?: string;
  study_name?: string;
  label?: string;
  method: string;
  status: 'pending' | 'running' | 'completed' | 'error';
  duration_ms?: number;
  error_message?: string;
  created_at: string;
}

export interface TimeStepMatrices {
  /** stock_amort[j]   = outstanding × profile[j]  (balance résiduelle) — taille B+1 */
  stock_amort:     number[];
  /** stock_instal[j]  = stock_amort[j-1] − stock_amort[j]              — taille B+1 */
  stock_instal:    number[];
  /** varstock_amort[j] = variation de stock entre t et t-1              — taille B+1 */
  varstock_amort:  number[];
  /** varstock_instal[j] = différences de varstock_amort                 — taille B+1 */
  varstock_instal: number[];
  /** ftp_rate[j]   = taux FTP calculé par le kernel (moyenne pondérée)  — taille B+1 */
  ftp_rate:        number[];
  /** ftp_int[j]    = intérêts FTP par bucket                            — taille B+1 */
  ftp_int:         number[];
  /** market_rate[j] = taux marché back-solved                           — taille B+1 */
  market_rate:     number[];
}

export interface TimeStep {
  date: string;
  period_type: 'observed' | 'projected' | 'contrafactual';
  kpis: {
    total_outstanding: number;
    weighted_ftp_rate: number;
    ftp_interest_periodic: number;
  };
  /** Taux d'entrée interpolés depuis la matrice de taux, indexés par label de bucket */
  ftp_by_tenor: Record<string, number>;
  /** Poids d'amortissement du schedule à cette date (Σ ≈ 1) */
  profile: number[];
  /** Matrices intermédiaires du kernel matched-maturity (toutes de taille B+1) */
  matrices: TimeStepMatrices;
}

export interface AssignmentResult {
  assignment_id: string;
  pair_id: string;
  pair_label?: string;
  /** 'Stock' | 'Flux' */
  method: string;
  vector_name: string;
  schedule_name: string;
  bucket_labels: string[];
  combination_matrix_ids: string[];
  time_step_count: number;
  time_steps: TimeStep[];
}

export interface StudyUnitResult {
  study_unit_id: string;
  study_unit_name: string;
  hypercube_id: string;
  time_step_range: { start?: string; end?: string; count: number };
  assignments: AssignmentResult[];
}

export interface ExecutionResult {
  study_units: StudyUnitResult[];
}

export interface ExecutionDetail {
  id: string;
  study_id?: string;
  study_name?: string;
  label?: string;
  method: string;
  status: 'pending' | 'running' | 'completed' | 'error';
  result?: ExecutionResult;
  duration_ms?: number;
  error_message?: string;
  created_at: string;
}

// ── API — Executions ──────────────────────────────────────────────────────────

export const executions = {
  list:   () => req<ExecutionSummary[]>('/executions'),
  get:    (id: string) => req<ExecutionDetail>(`/executions/${id}`),
  create: (data: { study_id: string; label?: string }) =>
    req<ExecutionDetail>('/executions', { method: 'POST', body: JSON.stringify(data) }),
  delete: (id: string) => fetch(`${API}/executions/${id}`, { method: 'DELETE' }),
};

// ── Types — Module 5 : Studies ───────────────────────────────────────────────

export interface StudySummary {
  id: string;
  name: string;
  description?: string;
  status: 'draft' | 'ready' | 'archived';
  unit_count: number;
  valid_unit_count: number;
  created_at: string;
}

export interface StudyUnitRef {
  study_unit_id: string;
  name: string;
  hypercube_name: string;
  portfolio_name: string;
  is_valid: boolean;
  assignment_count: number;
  label?: string;
  position: number;
}

export interface StudyDetail {
  id: string;
  name: string;
  description?: string;
  status: 'draft' | 'ready' | 'archived';
  units: StudyUnitRef[];
  created_at: string;
}

// ── API — Studies ─────────────────────────────────────────────────────────────

export const studies = {
  list: () => req<StudySummary[]>('/studies'),
  get:  (id: string) => req<StudyDetail>(`/studies/${id}`),
  create: (data: { name: string; description?: string; status?: string }) =>
    req<StudyDetail>('/studies', { method: 'POST', body: JSON.stringify(data) }),
  update: (id: string, data: { name?: string; description?: string; status?: string }) =>
    req<StudyDetail>(`/studies/${id}`, { method: 'PUT', body: JSON.stringify(data) }),
  delete: (id: string) => fetch(`${API}/studies/${id}`, { method: 'DELETE' }),
  addUnit:    (id: string, data: { study_unit_id: string; label?: string }) =>
    req<StudyDetail>(`/studies/${id}/units`, { method: 'POST', body: JSON.stringify(data) }),
  removeUnit: (id: string, unitId: string) =>
    fetch(`${API}/studies/${id}/units/${unitId}`, { method: 'DELETE' }),
};

// ── Types — Module 4 : Study units ───────────────────────────────────────────

export interface StudyUnitSummary {
  id: string;
  name: string;
  description?: string;
  hypercube_id: string;
  hypercube_name: string;
  portfolio_id: string;
  portfolio_name: string;
  start_date: string;
  granularity_rule: string;
  is_valid: boolean;
  assignment_count: number;
  created_at: string;
}

export interface AssignmentInfo {
  id: string;
  pair_id: string;
  pair_label?: string;
  vector_id: string;
  vector_name: string;
  schedule_id: string;
  schedule_name: string;
  schedule_type: 'stock' | 'new_production';
  combination_matrix_ids: string[];
  label?: string;
  is_existing_stock: boolean;
  methods: string[];
  initial_ftp_profile_json?: { tenor: string; rate: number }[];
  created_at: string;
}

export interface StudyUnitDetail {
  id: string;
  name: string;
  description?: string;
  hypercube_id: string;
  portfolio_id: string;
  start_date: string;
  granularity_rule: string;
  is_valid: boolean;
  validation_log?: string;
  assignments: AssignmentInfo[];
  created_at: string;
}

export interface ValidationCheck {
  check: string;
  passed: boolean;
  message: string;
}

export interface ValidationReport {
  is_valid: boolean;
  checks: ValidationCheck[];
}

// ── API — Hypercubes ──────────────────────────────────────────────────────────

// ── API — Study units ─────────────────────────────────────────────────────────

export const studyUnits = {
  list: () => req<StudyUnitSummary[]>('/study-units'),
  get:  (id: string) => req<StudyUnitDetail>(`/study-units/${id}`),
  create: (data: {
    name: string; description?: string;
    hypercube_id: string; portfolio_id: string;
    start_date: string; granularity_rule?: string;
  }) => req<StudyUnitDetail>('/study-units', { method: 'POST', body: JSON.stringify(data) }),
  update: (id: string, data: {
    name?: string; description?: string;
    start_date?: string; granularity_rule?: string;
  }) => req<StudyUnitDetail>(`/study-units/${id}`, { method: 'PUT', body: JSON.stringify(data) }),
  delete: (id: string) => fetch(`${API}/study-units/${id}`, { method: 'DELETE' }),

  validate: (id: string) =>
    req<ValidationReport>(`/study-units/${id}/validate`, { method: 'POST' }),

  createAssignment: (id: string, data: {
    pair_id: string; combination_matrix_ids: string[];
    label?: string; is_existing_stock?: boolean;
    methods?: string[];
    initial_ftp_profile_json?: { tenor: string; rate: number }[];
  }) => req<AssignmentInfo>(`/study-units/${id}/assignments`, {
    method: 'POST', body: JSON.stringify(data),
  }),
  updateAssignment: (id: string, aid: string, data: {
    combination_matrix_ids?: string[]; label?: string;
    is_existing_stock?: boolean; methods?: string[];
    initial_ftp_profile_json?: { tenor: string; rate: number }[];
  }) => req<AssignmentInfo>(`/study-units/${id}/assignments/${aid}`, {
    method: 'PUT', body: JSON.stringify(data),
  }),
  deleteAssignment: (id: string, aid: string) =>
    fetch(`${API}/study-units/${id}/assignments/${aid}`, { method: 'DELETE' }),
};

export const hypercubes = {
  list: (params?: { status?: string }) => {
    const qs = new URLSearchParams();
    if (params?.status) qs.set('status', params.status);
    const q = qs.toString();
    return req<HypercubeSummary[]>(`/hypercubes${q ? `?${q}` : ''}`);
  },

  get: (id: string) => req<HypercubeDetail>(`/hypercubes/${id}`),

  create: (data: CreateHypercubeRequest) =>
    req<HypercubeDetail>('/hypercubes', {
      method: 'POST',
      body: JSON.stringify(data),
    }),

  update: (id: string, data: UpdateHypercubeRequest) =>
    req<HypercubeDetail>(`/hypercubes/${id}`, {
      method: 'PUT',
      body: JSON.stringify(data),
    }),

  delete: (id: string) =>
    fetch(`${API}/hypercubes/${id}`, { method: 'DELETE' }),

  combinations: (id: string) =>
    req<Combination[]>(`/hypercubes/${id}/combinations`),
};
