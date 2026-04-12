const API = 'http://localhost:3000/api';

async function req<T>(path: string, opts?: RequestInit): Promise<T> {
  const res = await fetch(`${API}${path}`, {
    headers: { 'Content-Type': 'application/json' },
    ...opts,
  });
  if (!res.ok) throw new Error(`HTTP ${res.status}: ${await res.text()}`);
  return res.json() as Promise<T>;
}

// ── Types ─────────────────────────────────────────────────────────────────────

export interface RateCurve {
  id: string;
  name: string;
  component: string;
  currency: string;
  version: number;
  status: 'draft' | 'approved' | 'archived';
  valid_from?: string;
  valid_to?: string;
  tenors_json: string;
  values_json: string;
  source?: string;
  notes?: string;
  /** Name of the historical rate series this curve is derived from (e.g. "SOFR") */
  series_name?: string;
  created_at: string;
}

export interface Portfolio {
  id: string;
  name: string;
  description?: string;
  version: number;
  status: string;
  as_of_date: string;
  position_count?: number;
  created_at: string;
}

export interface PortfolioPosition {
  id: string;
  portfolio_id: string;
  position_ref?: string;
  product_type: string;
  branch?: string;
  seller?: string;
  currency: string;
  outstanding: number;
  origination_date?: string;
  maturity_date?: string;
  client_rate?: number;
  runoff_model_id?: string;
  risk_weight: number;
  profiles_json?: string;
  rates_json?: string;
}

export interface RunoffModel {
  id: string;
  name: string;
  product_type: string;
  category?: string;
  version: number;
  status: string;
  method: string;
  profile_json: string;
  parameters_json?: string;
  created_at: string;
}

export interface Execution {
  id: string;
  label?: string;
  method: string;
  portfolio_id: string;
  status: string;
  duration_ms?: number;
  result_json?: string;
  error_message?: string;
  created_at: string;
  outstanding_json?: string;
  profiles_json?: string;
  rates_json?: string;
}

export interface ExecutionInputs {
  execution_id: string;
  method: string;
  portfolio_id: string;
  label?: string;
  outstanding_json?: string;
  profiles_json?: string;
  rates_json?: string;
}

export interface ComputeResponse {
  execution_id: string;
  status: string;
  method: string;
  duration_ms: number;
  ftp_rate?: number[][];
  ftp_int?: number[][];
  market_rate?: number[][];
  stock_amort?: number[][];
  total_outstanding: number;
  weighted_ftp_rate: number;
  total_ftp_int_monthly: number;
  error?: string;
}

// ── Curves ────────────────────────────────────────────────────────────────────

export const curves = {
  list: () => req<RateCurve[]>('/curves'),
  get:  (id: string) => req<RateCurve>(`/curves/${id}`),
  create: (data: Partial<RateCurve>) =>
    req<RateCurve>('/curves', { method: 'POST', body: JSON.stringify(data) }),
  update: (id: string, data: Partial<RateCurve>) =>
    req<RateCurve>(`/curves/${id}`, { method: 'PUT', body: JSON.stringify(data) }),
  delete: (id: string) =>
    fetch(`${API}/curves/${id}`, { method: 'DELETE' }),
};

// ── Portfolios ─────────────────────────────────────────────────────────────────

export const portfolios = {
  list: () => req<Portfolio[]>('/portfolios'),
  get:  (id: string) => req<Portfolio>(`/portfolios/${id}`),
  create: (data: { name: string; description?: string; as_of_date: string }) =>
    req<Portfolio>('/portfolios', { method: 'POST', body: JSON.stringify(data) }),
  delete: (id: string) => fetch(`${API}/portfolios/${id}`, { method: 'DELETE' }),
  positions: (id: string) => req<PortfolioPosition[]>(`/portfolios/${id}/positions`),
  addPosition: (id: string, pos: Partial<PortfolioPosition>) =>
    req<PortfolioPosition>(`/portfolios/${id}/positions`, {
      method: 'POST', body: JSON.stringify(pos),
    }),
  bulkImport: (id: string, positions: Partial<PortfolioPosition>[]) =>
    req<{ imported: number }>(`/portfolios/${id}/positions/bulk`, {
      method: 'POST', body: JSON.stringify(positions),
    }),
  deletePosition: (posId: string) =>
    fetch(`${API}/positions/${posId}`, { method: 'DELETE' }),
};

// ── Runoff models ─────────────────────────────────────────────────────────────

export const runoff = {
  list: () => req<RunoffModel[]>('/runoff'),
  get:  (id: string) => req<RunoffModel>(`/runoff/${id}`),
  create: (data: Partial<RunoffModel>) =>
    req<RunoffModel>('/runoff', { method: 'POST', body: JSON.stringify(data) }),
  delete: (id: string) => fetch(`${API}/runoff/${id}`, { method: 'DELETE' }),
};

// ── Executions ────────────────────────────────────────────────────────────────

export interface ExecSummary {
  id: string;
  label?: string;
  method: string;
  created_at: string;
  kpis?: { weighted_ftp_rate?: number; total_outstanding?: number; total_ftp_int_monthly?: number };
}

export interface DiffResponse {
  a: ExecSummary;
  b: ExecSummary;
  delta_weighted_ftp_rate?: number;
  delta_total_outstanding?: number;
  delta_ftp_int_monthly?: number;
}

export const executions = {
  list:   () => req<Execution[]>('/executions'),
  get:    (id: string) => req<Execution>(`/executions/${id}`),
  inputs: (id: string) => req<ExecutionInputs>(`/executions/${id}/inputs`),
  diff:   (a: string, b: string) => req<DiffResponse>(`/executions/diff?a=${a}&b=${b}`),
};

// ── Compute ───────────────────────────────────────────────────────────────────

export interface ComputeRequest {
  method: string;
  portfolio_id: string;
  label?: string;
  curve_ids?: string[];
  outstanding_json: string;
  profiles_json: string;
  rates_json: string;
  parameters_json?: string;
  seeds_json?: string;
}

export const compute = (data: ComputeRequest) =>
  req<ComputeResponse>('/compute', { method: 'POST', body: JSON.stringify(data) });

// ── Analytics ─────────────────────────────────────────────────────────────────

export interface BucketStats {
  outstanding: number;
  count: number;
  avg_client_rate?: number;
  avg_ftp_rate?: number;
  avg_nim?: number;
  total_nim_income: number;
}

export interface NimHeatmapResponse {
  portfolio_id: string;
  execution_id?: string;
  method?: string;
  positions: {
    index: number;
    branch?: string;
    product_type: string;
    seller?: string;
    outstanding: number;
    client_rate?: number;
    ftp_rate?: number;
    nim?: number;
  }[];
  by_branch:  Record<string, BucketStats>;
  by_product: Record<string, BucketStats>;
  by_seller:  Record<string, BucketStats>;
}

// ── Datasets ──────────────────────────────────────────────────────────────────

export interface Dataset {
  id: string;
  name: string;
  description?: string;
  status: 'active' | 'frozen' | 'archived';
  source: 'manual' | 'uploaded' | 'generated';
  as_of_date?: string;
  created_at: string;
  count_contracts: number;
  count_rate_curves: number;
  count_runoff_models: number;
}

export interface Contract {
  id: string;
  contract_id: string;
  contract_type: string;
  side: string;
  seller_id?: string;
  branch_code?: string;
  currency: string;
  rating?: string;
  notional: number;
  rate_type?: string;
  interest_rate?: number;
  settlement_date?: string;
  maturity_date?: string;
  tenor_months?: number;
  payment_frequency?: string;
  day_count?: string;
  business_day_convention?: string;
  amortization_type?: string;
  prepayment_allowed: boolean;
  prepayment_penalty: number;
  guarantee_type?: string;
  runoff_model_id?: string;
  profiles_json?: string;
  rates_json?: string;
  risk_weight: number;
  created_at: string;
}

export interface FreezeResult {
  freeze_id: string;
  dataset_id: string;
  contracts_csv: string;
  rate_curves_csv: string;
  contracts_count: number;
  rate_curves_count: number;
  frozen_at: string;
}

export interface FsDatasetMeta {
  id: string;
  name: string;
  description?: string;
  as_of_date?: string;
  created_at?: string;
  contract_count?: number;
  filters?: Record<string, string>;
}

export interface FsDataset {
  folder: string;
  loaded_in_db: boolean;
  meta: FsDatasetMeta;
}

export interface AvailableDatasetsResponse {
  datasets_dir: string;
  datasets: FsDataset[];
}

export interface LoadFsDatasetResult {
  dataset_id: string;
  name: string;
  loaded: { curves: number; runoff_models: number; contracts: number };
}

export const datasets = {
  list: () => req<Dataset[]>('/datasets'),
  create: (data: { name: string; description?: string; source?: string; as_of_date?: string }) =>
    req<Dataset>('/datasets', { method: 'POST', body: JSON.stringify(data) }),
  delete: (id: string) => fetch(`${API}/datasets/${id}`, { method: 'DELETE' }),
  contracts: (id: string, limit = 200, offset = 0) =>
    req<Contract[]>(`/datasets/${id}/contracts?limit=${limit}&offset=${offset}`),
  summary: (id: string) => req<{ dataset_id: string; counts: Record<string, number> }>(`/datasets/${id}/summary`),
  freeze: (id: string) => req<FreezeResult>(`/datasets/${id}/freeze`),
  exportZip: (id: string) => fetch(`${API}/datasets/${id}/export-zip`),
  addItems: (id: string, entity_type: string, entity_ids: string[]) =>
    req<{ added: number }>(`/datasets/${id}/items`, {
      method: 'POST', body: JSON.stringify({ entity_type, entity_ids }),
    }),
  uploadCsv: (formData: FormData) =>
    fetch(`${API}/datasets/upload`, { method: 'POST', body: formData })
      .then(async r => {
        if (!r.ok) throw new Error(`HTTP ${r.status}: ${await r.text()}`);
        return r.json() as Promise<{ dataset_id: string; imported: number; errors: number; error_sample: string[] }>;
      }),
  available: () => req<AvailableDatasetsResponse>('/datasets/available'),
  loadFs: (folder: string) =>
    req<LoadFsDatasetResult>(`/datasets/fs/${encodeURIComponent(folder)}/load`, { method: 'POST' }),
  ingest: (formData: FormData) =>
    fetch(`${API}/datasets/ingest`, { method: 'POST', body: formData })
      .then(async r => {
        if (!r.ok) throw new Error(`HTTP ${r.status}: ${await r.text()}`);
        return r.json() as Promise<{ dataset_id: string; imported: Record<string, number> }>;
      }),
};

// ── Rate Series ───────────────────────────────────────────────────────────────

export interface RateSeriesPoint {
  date: string;
  tenor: string | null;
  rate: number;
}

export interface RateSeriesResponse {
  total_rows: number;
  data: Record<string, RateSeriesPoint[]>;
}

export interface SeriesInfo {
  name: string;
  component: string;
  currency: string;
}

export const rateSeries = {
  names: () => req<{ series: SeriesInfo[] }>('/rate-series/names'),
  query: (params: { series?: string[]; from?: string; to?: string; tenor?: string; limit?: number }) => {
    const qs = new URLSearchParams();
    if (params.series?.length) qs.set('series', params.series.join(','));
    if (params.from) qs.set('from', params.from);
    if (params.to)   qs.set('to', params.to);
    if (params.tenor) qs.set('tenor', params.tenor);
    if (params.limit) qs.set('limit', String(params.limit));
    return req<RateSeriesResponse>(`/rate-series?${qs.toString()}`);
  },
};

export const analytics = {
  portfolioNim: (portfolioId: string, executionId?: string) => {
    const qs = executionId ? `?portfolio_id=${portfolioId}&execution_id=${executionId}`
                           : `?portfolio_id=${portfolioId}`;
    return req<NimHeatmapResponse>(`/analytics/portfolio-nim${qs}`);
  },
};

// ── Curve Stacks (V3) ─────────────────────────────────────────────────────────

export interface CurveStack {
  id: string;
  name: string;
  description?: string;
  status: string;
  created_at: string;
  component_count: number;
}

export interface StackComponentDetail {
  id: string;
  stack_id: string;
  position: number;
  label: string;
  curve_id: string;
  weight: number;
  interp_method: string;
  curve_name?: string;
  curve_component?: string;
  /** The rate series underlying this curve (e.g. "SOFR") — used for projection config */
  curve_series_name?: string;
}

export interface CurveStackDetail {
  id: string;
  name: string;
  description?: string;
  status: string;
  created_at: string;
  components: StackComponentDetail[];
}

export interface CreateStackRequest {
  name: string;
  description?: string;
  components: { label: string; curve_id: string; weight?: number; interp_method?: string }[];
}

export interface GenerateCombinationsRequest {
  name_prefix: string;
  description?: string;
  components: { label: string; curve_ids: string[] }[];
}

export const stacks = {
  list: () => req<CurveStack[]>('/stacks'),
  get:  (id: string) => req<CurveStackDetail>(`/stacks/${id}`),
  create: (data: CreateStackRequest) =>
    req<CurveStackDetail>('/stacks', { method: 'POST', body: JSON.stringify(data) }),
  update: (id: string, data: CreateStackRequest) =>
    req<CurveStackDetail>(`/stacks/${id}`, { method: 'PUT', body: JSON.stringify(data) }),
  delete: (id: string) => fetch(`${API}/stacks/${id}`, { method: 'DELETE' }),
  generateCombinations: (data: GenerateCombinationsRequest) =>
    req<{ created: number; stack_ids: string[] }>('/stacks/generate-combinations', {
      method: 'POST', body: JSON.stringify(data),
    }),
};

// ── Portfolios V3 ─────────────────────────────────────────────────────────────

export interface PortfolioV3 {
  id: string;
  name: string;
  description?: string;
  schedule_type: 'stock_amort' | 'new_prod_amort';
  created_at: string;
  row_count: number;
}

export interface PortfolioV3Row {
  id: string;
  label?: string;
  schedule_json: string;
  outstanding_json: string;
  row_order: number;
}

export interface PortfolioV3Detail {
  id: string;
  name: string;
  description?: string;
  schedule_type: 'stock_amort' | 'new_prod_amort';
  created_at: string;
  rows: PortfolioV3Row[];
}

export const portfoliosV3 = {
  list: () => req<PortfolioV3[]>('/portfolios-v3'),
  get:  (id: string) => req<PortfolioV3Detail>(`/portfolios-v3/${id}`),
  create: (data: { name: string; description?: string; schedule_type?: string }) =>
    req<PortfolioV3>('/portfolios-v3', { method: 'POST', body: JSON.stringify(data) }),
  delete: (id: string) => fetch(`${API}/portfolios-v3/${id}`, { method: 'DELETE' }),
  uploadRow: (portfolioId: string, form: FormData) =>
    fetch(`${API}/portfolios-v3/${portfolioId}/rows/upload`, { method: 'POST', body: form })
      .then(async r => {
        if (!r.ok) throw new Error(`HTTP ${r.status}: ${await r.text()}`);
        return r.json();
      }),
  getRow: (portfolioId: string, rowId: string) =>
    req<PortfolioV3Row>(`/portfolios-v3/${portfolioId}/rows/${rowId}`),
  deleteRow: (rowId: string) =>
    fetch(`${API}/portfolio-rows/${rowId}`, { method: 'DELETE' }),
};

// ── Curve Cubes (V3) ──────────────────────────────────────────────────────────

export interface CurveCube {
  id: string;
  name: string;
  description?: string;
  stack_id: string;
  stack_name?: string;
  analysis_start: string;
  analysis_end: string;
  step_months: number;
  include_proj: boolean;
  proj_script?: string;
  mc_scenarios: number;
  /** JSON: { "SOFR": { method, n_scenarios, seed, params? }, ... } */
  proj_config_json?: string;
  status: string;
  created_at: string;
  n_analysis_times: number;
}

/** Per-series projection configuration stored inside proj_config_json */
export interface SeriesProjConfig {
  method: 'pca_bootstrap' | 'hw2f' | 'deterministic' | 'custom';
  n_scenarios: number;
  seed?: number;
  /** Custom Python script (Pyodide) */
  script?: string;
  /** Model-specific parameters (kappa, sigma, rho…) */
  params?: Record<string, number>;
}

export interface CreateCubeRequest {
  name: string;
  description?: string;
  stack_id: string;
  analysis_start: string;
  analysis_end: string;
  step_months?: number;
  include_proj?: boolean;
  proj_script?: string;
  mc_scenarios?: number;
  proj_config_json?: string;
}

export const cubes = {
  list:   () => req<CurveCube[]>('/cubes'),
  get:    (id: string) => req<CurveCube>(`/cubes/${id}`),
  create: (data: CreateCubeRequest) =>
    req<CurveCube>('/cubes', { method: 'POST', body: JSON.stringify(data) }),
  update: (id: string, data: CreateCubeRequest) =>
    req<CurveCube>(`/cubes/${id}`, { method: 'PUT', body: JSON.stringify(data) }),
  delete: (id: string) => fetch(`${API}/cubes/${id}`, { method: 'DELETE' }),
};

// ── Linkers (V3) ──────────────────────────────────────────────────────────────

export interface Linker {
  id: string;
  name: string;
  portfolio_id: string;
  portfolio_name?: string;
  cube_id: string;
  cube_name?: string;
  start_date: string;
  fwd_schedule_json?: string;
  fwd_outstanding_json?: string;
  created_at: string;
}

export const linkers = {
  list:   () => req<Linker[]>('/linkers'),
  get:    (id: string) => req<Linker>(`/linkers/${id}`),
  create: (data: { name: string; portfolio_id: string; cube_id: string; start_date: string; fwd_schedule_json?: string; fwd_outstanding_json?: string }) =>
    req<Linker>('/linkers', { method: 'POST', body: JSON.stringify(data) }),
  delete: (id: string) => fetch(`${API}/linkers/${id}`, { method: 'DELETE' }),
};

// ── Studies (V3) ──────────────────────────────────────────────────────────────

export interface Study {
  id: string;
  name: string;
  description?: string;
  notes?: string;
  created_at: string;
  linker_count: number;
}

export interface StudyLinkerEntry {
  linker_id: string;
  linker_name?: string;
  portfolio_name?: string;
  cube_name?: string;
  start_date?: string;
  label?: string;
  position: number;
}

export interface StudyDetail {
  id: string;
  name: string;
  description?: string;
  notes?: string;
  created_at: string;
  linkers: StudyLinkerEntry[];
}

export const studies = {
  list:   () => req<Study[]>('/studies'),
  get:    (id: string) => req<StudyDetail>(`/studies/${id}`),
  create: (data: { name: string; description?: string; notes?: string }) =>
    req<StudyDetail>('/studies', { method: 'POST', body: JSON.stringify(data) }),
  update: (id: string, data: { name?: string; description?: string; notes?: string }) =>
    req<StudyDetail>(`/studies/${id}`, { method: 'PUT', body: JSON.stringify(data) }),
  delete: (id: string) => fetch(`${API}/studies/${id}`, { method: 'DELETE' }),
  addLinker: (studyId: string, data: { linker_id: string; label?: string }) =>
    req<StudyDetail>(`/studies/${studyId}/linkers`, { method: 'POST', body: JSON.stringify(data) }),
  removeLinker: (studyId: string, linkerId: string) =>
    req<StudyDetail>(`/studies/${studyId}/linkers/${linkerId}`, { method: 'DELETE' }),
};

// ── Executions V3 ─────────────────────────────────────────────────────────────

export interface ExecutionV3 {
  id: string;
  label?: string;
  study_name?: string;
  method: string;
  status: string;
  duration_ms?: number;
  created_at: string;
}

export interface ExecutionV3Detail {
  id: string;
  label?: string;
  study_name?: string;
  method: string;
  status: string;
  duration_ms?: number;
  created_at: string;
  error?: string;
  result?: {
    linkers: {
      linker_id: string;
      linker_name?: string;
      label?: string;
      error?: string;
      analysis_times: {
        date: string;
        error?: string;
        kpis?: {
          total_outstanding: number;
          weighted_ftp_rate: number;
          total_ftp_int_monthly: number;
        };
      }[];
    }[];
  };
}

export const executionsV3 = {
  list:   () => req<ExecutionV3[]>('/executions-v3'),
  get:    (id: string) => req<ExecutionV3Detail>(`/executions-v3/${id}`),
  run:    (data: { study_id: string; label?: string; method?: string }) =>
    req<ExecutionV3Detail>('/executions-v3', { method: 'POST', body: JSON.stringify(data) }),
  delete: (id: string) => fetch(`${API}/executions-v3/${id}`, { method: 'DELETE' }),
};
