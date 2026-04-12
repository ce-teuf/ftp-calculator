<script lang="ts">
  import { onMount } from 'svelte';
  import { cubes, stacks } from '../api/client';
  import type { CurveCube, CurveStack, CurveStackDetail, SeriesProjConfig } from '../api/client';
  import { Plus, Trash2, Box, CalendarRange, GitBranch, Layers } from '@lucide/svelte';

  type View = 'list' | 'builder';
  let view = $state<View>('list');

  let cubeList      = $state<CurveCube[]>([]);
  let stackList     = $state<CurveStack[]>([]);
  let loading       = $state(true);
  let error         = $state('');
  let selected      = $state<CurveCube | null>(null);

  // ── Builder form ────────────────────────────────────────────────────────────
  let bName         = $state('');
  let bDescription  = $state('');
  let bStackId      = $state('');
  let bStart        = $state('');
  let bEnd          = $state('');
  let bStep         = $state(1);
  let bMc           = $state(0);
  let saving        = $state(false);
  let saveError     = $state('');

  // Stack detail (loaded when bStackId changes)
  let stackDetail   = $state<CurveStackDetail | null>(null);
  let loadingStack  = $state(false);

  // Per-series projection config
  // keyed by series_name; components without a series get a synthetic key = "comp:${component}"
  let projConfig = $state<Record<string, SeriesProjConfig & { _key: string; _label: string }>>({});

  const PROJ_METHODS: { value: SeriesProjConfig['method']; label: string; hint: string }[] = [
    { value: 'deterministic', label: 'Deterministic',      hint: 'Single trajectory — no scenarios' },
    { value: 'pca_bootstrap', label: 'PCA Bootstrap',     hint: 'Historical resample — 3 factors (level/slope/curvature)' },
    { value: 'hw2f',          label: 'Hull-White 2F',     hint: 'G2++ — two correlated mean-reverting processes' },
    { value: 'custom',        label: 'Python Script',     hint: 'User-defined Python function (Pyodide)' },
  ];

  // Total scenario count (max across all series configs)
  const totalScenarios = $derived.by(() => {
    const vals = Object.values(projConfig);
    if (!vals.length) return Math.max(1, bMc);
    const max = Math.max(...vals.map(v => v.n_scenarios ?? 0));
    return Math.max(max, 1);
  });

  const componentCount = $derived(stackDetail?.components.length ?? 0);

  $effect(() => {
    if (bStackId) loadStackDetail(bStackId);
    else { stackDetail = null; projConfig = {}; }
  });

  const STEPS = [
    { value: 1,  label: 'Monthly (1M)'     },
    { value: 3,  label: 'Quarterly (3M)' },
    { value: 6,  label: 'Biannual (6M)'  },
    { value: 12, label: 'Annual (12M)'     },
  ];

  // Preview: how many analysis times?
  let nTimes = $derived.by(() => {
    if (!bStart || !bEnd || bStep <= 0) return 0;
    const [sy, sm] = bStart.split('-').map(Number);
    const [ey, em] = bEnd.split('-').map(Number);
    if (!sy || !ey) return 0;
    const total = (ey - sy) * 12 + (em - sm);
    if (total < 0) return 0;
    return Math.floor(total / bStep) + 1;
  });

  async function load() {
    loading = true; error = '';
    try {
      [cubeList, stackList] = await Promise.all([cubes.list(), stacks.list()]);
    } catch (e: any) { error = e.message; }
    finally { loading = false; }
  }

  async function deleteCube(id: string, name: string) {
    if (!confirm(`Delete the cube "${name}"?`)) return;
    await cubes.delete(id);
    if (selected?.id === id) selected = null;
    await load();
  }

  async function loadStackDetail(id: string) {
    loadingStack = true;
    try {
      stackDetail = await stacks.get(id);
      // Rebuild projConfig: one entry per unique series_name (or component key as fallback)
      const newConfig: typeof projConfig = {};
      const seen = new Set<string>();
      for (const comp of stackDetail.components) {
        const key = comp.curve_series_name
          ? `series:${comp.curve_series_name}`
          : `comp:${comp.curve_component ?? comp.curve_id}`;
        if (seen.has(key)) continue;
        seen.add(key);
        const label = comp.curve_series_name
          ? comp.curve_series_name
          : `${comp.curve_component ?? 'component'} (no series)`;
        newConfig[key] = {
          _key: key,
          _label: label,
          method: 'deterministic',
          n_scenarios: 1,
          seed: undefined,
          script: undefined,
          params: undefined,
        };
      }
      projConfig = newConfig;
    } catch { stackDetail = null; }
    finally { loadingStack = false; }
  }

  function openBuilder() {
    bName = ''; bDescription = ''; bStackId = '';
    bStart = ''; bEnd = ''; bStep = 1; bMc = 0;
    stackDetail = null; projConfig = {};
    saveError = '';
    view = 'builder';
  }

  async function save() {
    saveError = '';
    if (!bName.trim())    { saveError = 'Name required';             return; }
    if (!bStackId)        { saveError = 'Stack required';           return; }
    if (!bStart || !bEnd) { saveError = 'Start and end dates required'; return; }
    if (bStart >= bEnd)   { saveError = 'End date must be after start date'; return; }

    // Build proj_config_json: keyed by series_name (strip "series:" prefix)
    const configObj: Record<string, Omit<SeriesProjConfig, '_key' | '_label'>> = {};
    let maxScenarios = 0;
    for (const [, cfg] of Object.entries(projConfig)) {
      if (cfg.method === 'deterministic') continue;
      const seriesKey = cfg._key.startsWith('series:')
        ? cfg._key.slice(7)
        : cfg._key;
      configObj[seriesKey] = {
        method: cfg.method,
        n_scenarios: cfg.n_scenarios,
        seed: cfg.seed,
        script: cfg.script || undefined,
        params: cfg.params,
      };
      maxScenarios = Math.max(maxScenarios, cfg.n_scenarios ?? 0);
    }

    saving = true;
    try {
      await cubes.create({
        name: bName.trim(),
        description: bDescription || undefined,
        stack_id: bStackId,
        analysis_start: bStart,
        analysis_end: bEnd,
        step_months: bStep,
        include_proj: maxScenarios > 0,
        mc_scenarios: maxScenarios,
        proj_config_json: Object.keys(configObj).length ? JSON.stringify(configObj) : undefined,
      });
      await load();
      view = 'list';
    } catch (e: any) { saveError = e.message; }
    finally { saving = false; }
  }

  onMount(load);
</script>

<div class="tab-content">
  <div class="tab-header">
    <h2>Curve Cubes</h2>
    <div style="display:flex;gap:8px">
      {#if view === 'builder'}
        <button class="btn-sm" onclick={() => view = 'list'}>← Back</button>
      {:else}
        <button class="btn-primary" onclick={openBuilder}>
          <Plus size={14}/> New cube
        </button>
      {/if}
    </div>
  </div>

  {#if error}<div class="alert-error">{error}</div>{/if}

  <!-- ── List ──────────────────────────────────────────────────────────────── -->
  {#if view === 'list'}
    {#if loading}
      <p class="loading">Loading…</p>
    {:else if cubeList.length === 0}
      <div class="empty-state">
        <p>No cube defined.</p>
        <p>A cube adds the temporal dimension to a stack: it defines analysis dates, step and projection options.</p>
        <button class="btn-primary" onclick={openBuilder}><Plus size={14}/> Create a cube</button>
      </div>
    {:else}
      <div class="cube-grid">
        {#each cubeList as c}
          <div
            class="cube-card"
            class:selected={selected?.id === c.id}
            onclick={() => selected = selected?.id === c.id ? null : c}
          >
            <div class="cc-top">
              <span class="cc-name">{c.name}</span>
              <span class="badge badge-{c.status}">{c.status}</span>
            </div>

            <div class="cc-stack">
              <GitBranch size={12}/> {c.stack_name ?? c.stack_id}
            </div>

            <div class="cc-timeline">
              <CalendarRange size={12}/>
              {c.analysis_start} → {c.analysis_end}
            </div>

            <div class="cc-tags">
              <span class="tag">{c.n_analysis_times} date{c.n_analysis_times !== 1 ? 's' : ''}</span>
              <span class="tag">pas {c.step_months}M</span>
              {#if c.include_proj}<span class="tag tag-purple">proj.</span>{/if}
              {#if c.mc_scenarios > 0}<span class="tag tag-orange">MC×{c.mc_scenarios}</span>{/if}
            </div>

            {#if c.description}
              <div class="cc-desc">{c.description}</div>
            {/if}

            <div class="cc-actions" onclick={e => e.stopPropagation()}>
              <button class="btn-sm btn-danger" onclick={() => deleteCube(c.id, c.name)}>
                <Trash2 size={12}/>
              </button>
            </div>
          </div>
        {/each}
      </div>

      <!-- Detail panel -->
      {#if selected}
        <div class="detail-panel card">
          <div class="dp-header">
            <Box size={16}/>
            <strong>{selected.name}</strong>
            <span class="badge badge-{selected.status}">{selected.status}</span>
          </div>

          <div class="dp-grid">
            <div class="dp-item">
              <span class="dp-lbl">Stack</span>
              <span class="dp-val">{selected.stack_name ?? selected.stack_id}</span>
            </div>
            <div class="dp-item">
              <span class="dp-lbl">Analysis start</span>
              <span class="dp-val">{selected.analysis_start}</span>
            </div>
            <div class="dp-item">
              <span class="dp-lbl">Analysis end</span>
              <span class="dp-val">{selected.analysis_end}</span>
            </div>
            <div class="dp-item">
              <span class="dp-lbl">Step</span>
              <span class="dp-val">{selected.step_months} mois</span>
            </div>
            <div class="dp-item">
              <span class="dp-lbl">Analysis dates</span>
              <span class="dp-val dp-highlight">{selected.n_analysis_times}</span>
            </div>
            <div class="dp-item">
              <span class="dp-lbl">Projections</span>
              <span class="dp-val">{selected.include_proj ? 'Yes' : 'No'}</span>
            </div>
            <div class="dp-item">
              <span class="dp-lbl">Monte Carlo</span>
              <span class="dp-val">{selected.mc_scenarios > 0 ? `${selected.mc_scenarios} scenarios` : 'No'}</span>
            </div>
          </div>

          <!-- Timeline visual -->
          <div class="timeline-wrap">
            {#each {length: Math.min(selected.n_analysis_times, 60)} as _, i}
              <div class="tl-dot" title="t+{i * selected.step_months}M"></div>
            {/each}
            {#if selected.n_analysis_times > 60}
              <span class="tl-more">+{selected.n_analysis_times - 60}</span>
            {/if}
          </div>
        </div>
      {/if}
    {/if}

  <!-- ── Builder ─────────────────────────────────────────────────────────────── -->
  {:else}
    <div class="builder card">
      <div class="builder-grid">
        <label>
          Cube name
          <input bind:value={bName} placeholder="Ex: Cube ESTR 2025-2030"/>
        </label>

        <label>
          Description (optional)
          <input bind:value={bDescription}/>
        </label>

        <label style="grid-column:1/-1">
          Curve stack
          <select bind:value={bStackId}>
            <option value="">-- Select a stack --</option>
            {#each stackList as s}
              <option value={s.id}>{s.name} ({s.component_count} component{s.component_count !== 1 ? 's' : ''})</option>
            {/each}
          </select>
          {#if stackList.length === 0}
            <span class="hint-warn">No stack available — first create a stack in the Stacks tab.</span>
          {/if}
        </label>

<label>
          Start date
          <input type="date" bind:value={bStart}/>
        </label>

        <label>
          End date
          <input type="date" bind:value={bEnd}/>
        </label>

        <label>
          Temporal step
          <select bind:value={bStep}>
            {#each STEPS as s}
              <option value={s.value}>{s.label}</option>
            {/each}
          </select>
        </label>

        <!-- 4D Preview box -->
        {#if nTimes > 0}
          <div class="preview-box" style="grid-column:1/-1">
            <div class="dim-grid">
              <div class="dim">
                <span class="dim-n">{nTimes}</span>
                <span class="dim-lbl">dates<br/><small>{bStart} → {bEnd} / {bStep}M</small></span>
              </div>
              <span class="dim-op">×</span>
              <div class="dim">
                <span class="dim-n">12</span>
                <span class="dim-lbl">tenors<br/><small>1M … 30Y</small></span>
              </div>
              <span class="dim-op">×</span>
              <div class="dim">
                <span class="dim-n">{totalScenarios}</span>
                <span class="dim-lbl">scenarios<br/><small>projections</small></span>
              </div>
              <span class="dim-op">×</span>
              <div class="dim">
                <span class="dim-n">{componentCount || '?'}</span>
                <span class="dim-lbl">components<br/><small>stack</small></span>
              </div>
            </div>
          </div>
        {/if}

        <!-- Per-series projection config -->
        {#if stackDetail || loadingStack}
          <div class="proj-section" style="grid-column:1/-1">
            <div class="section-sep">Underlying series projections</div>
            {#if loadingStack}
              <p class="loading">Loading the stack…</p>
            {:else if stackDetail}
<p class="proj-hint">
                For each rate series used in the stack, choose how to project it into the future.
                The "scenarios" dimension of the cube corresponds to simulated trajectories.
              </p>
              {#each Object.values(projConfig) as cfg}
                <div class="proj-row" class:proj-row--active={cfg.method !== 'deterministic'}>
                  <div class="proj-series-name">
                    <span class="series-pill">{cfg._label}</span>
                  </div>
                  <label class="proj-method">
                    Method
                    <select bind:value={cfg.method}>
                      {#each PROJ_METHODS as m}
                        <option value={m.value}>{m.label}</option>
                      {/each}
                    </select>
                    <span class="hint">{PROJ_METHODS.find(m => m.value === cfg.method)?.hint ?? ''}</span>
                  </label>
                  {#if cfg.method !== 'deterministic'}
                    <label class="proj-n">
                      Scenarios
                      <input type="number" bind:value={cfg.n_scenarios} min={1} max={10000} step={50} />
                    </label>
                    <label class="proj-seed">
                      Seed
                      <input type="number" bind:value={cfg.seed} min={0} placeholder="random" />
                    </label>
                  {/if}
                  {#if cfg.method === 'hw2f'}
                    <div class="proj-params">
                      <span class="params-title">G2++ Parameters</span>
                      <div class="params-grid">
                        {#each [['a','0.05'],['b','0.50'],['sigma','0.01'],['eta','0.005'],['rho','-0.70']] as [p, def]}
                          <label class="param-field">
                            {p}
                            <input
                              type="number" step="0.001"
                              value={cfg.params?.[p] ?? parseFloat(def)}
                              oninput={e => {
                                if (!cfg.params) cfg.params = {};
                                cfg.params[p] = parseFloat((e.target as HTMLInputElement).value);
                              }}
                            />
                          </label>
                        {/each}
                      </div>
                    </div>
                  {/if}
                  {#if cfg.method === 'custom'}
                    <label class="proj-script" style="grid-column:1/-1">
                      Python Script (Pyodide)
                      <textarea
                        bind:value={cfg.script}
                        rows={6}
                        placeholder="def simulate(as_of_date, n_scenarios, horizon_months, current_curve, historical_data, seed=None, **kwargs):&#10;    # return a list of n_scenarios simulated curves&#10;    ..."
                        style="font-family:monospace;font-size:11.5px"
                      ></textarea>
                    </label>
                  {/if}
                </div>
              {/each}
            {/if}
          </div>
        {/if}
      </div>

      {#if saveError}<div class="alert-error">{saveError}</div>{/if}

      <div class="builder-footer">
        <button class="btn-sm" onclick={() => view = 'list'}>Cancel</button>
        <button class="btn-primary" onclick={save} disabled={saving}>
          {saving ? 'Creating…' : 'Create the cube'}
        </button>
      </div>
    </div>
  {/if}
</div>

<style>
  /* ── Cube grid ───────────────────────────────────────────────────────────── */
  .cube-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(280px,1fr));
    gap: 14px;
    margin-bottom: 24px;
  }

  .cube-card {
    background: #fff;
    border: 1px solid #e5e7eb;
    border-radius: 12px;
    padding: 16px;
    cursor: pointer;
    position: relative;
    transition: border-color 150ms, box-shadow 150ms;
  }
  .cube-card:hover   { border-color: #6366f1; box-shadow: 0 0 0 3px rgba(99,102,241,.08); }
  .cube-card.selected{ border-color: #6366f1; box-shadow: 0 0 0 3px rgba(99,102,241,.14); }

  .cc-top      { display:flex; justify-content:space-between; align-items:flex-start; gap:8px; margin-bottom:8px; }
  .cc-name     { font-weight:600; font-size:14px; color:#1a1a2e; }
  .cc-stack    { display:flex; align-items:center; gap:5px; font-size:12px; color:#6b7280; margin-bottom:4px; }
  .cc-timeline { display:flex; align-items:center; gap:5px; font-size:12px; color:#6b7280; margin-bottom:8px; }
  .cc-tags     { display:flex; flex-wrap:wrap; gap:4px; margin-bottom:4px; }
  .cc-desc     { font-size:12px; color:#9ca3af; margin-top:4px; }
  .cc-actions  { position:absolute; bottom:12px; right:12px; opacity:0; transition:opacity 150ms; }
  .cube-card:hover .cc-actions { opacity:1; }

  .tag-purple { background:#ede9fe; color:#5b21b6; }
  .tag-orange { background:#fff7ed; color:#c2410c; }

  /* ── Detail panel ────────────────────────────────────────────────────────── */
  .detail-panel { padding:20px; }
  .dp-header { display:flex; align-items:center; gap:8px; margin-bottom:14px; font-size:15px; }

  .dp-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(180px,1fr));
    gap: 10px;
    margin-bottom: 16px;
  }
  .dp-item  { background:#f9fafb; border-radius:8px; padding:10px 12px; }
  .dp-lbl   { display:block; font-size:11px; color:#9ca3af; font-weight:600; text-transform:uppercase; letter-spacing:.04em; margin-bottom:3px; }
  .dp-val   { font-size:14px; font-weight:600; color:#1a1a2e; }
  .dp-highlight { color:#6366f1; font-size:18px; }

  /* Timeline dots */
  .timeline-wrap {
    display:flex; flex-wrap:wrap; gap:4px;
    padding:12px; background:#f9fafb; border-radius:8px;
    align-items:center;
  }
  .tl-dot {
    width:8px; height:8px;
    background:#6366f1; border-radius:50%;
    opacity:.7;
  }
  .tl-more { font-size:12px; color:#9ca3af; margin-left:4px; }

  /* ── Builder ─────────────────────────────────────────────────────────────── */
  .builder { padding:24px; }
  .builder-grid {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 14px;
    margin-bottom: 16px;
  }

  /* 4D preview */
  .preview-box {
    background: #ede9fe;
    border-radius: 10px;
    padding: 16px 20px;
    color: #4c1d95;
  }
  .dim-grid {
    display: flex;
    align-items: center;
    gap: 12px;
    flex-wrap: wrap;
  }
  .dim {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 2px;
  }
  .dim-n   { font-size: 28px; font-weight: 800; color: #6366f1; line-height: 1; }
  .dim-lbl { font-size: 11px; text-align: center; color: #5b21b6; line-height: 1.3; }
  .dim-lbl small { opacity: .7; }
  .dim-op  { font-size: 22px; font-weight: 700; color: #a5b4fc; }

  .section-sep {
    font-size: 11px;
    font-weight: 700;
    text-transform: uppercase;
    letter-spacing: .06em;
    color: #9ca3af;
    padding: 12px 0 8px;
    border-top: 1px solid #e5e7eb;
    margin-top: 4px;
  }

  /* Projection section */
  .proj-section { display: flex; flex-direction: column; gap: 10px; }

  .proj-hint {
    font-size: 12.5px;
    color: #6b7280;
    background: #f9fafb;
    border-radius: 8px;
    padding: 10px 12px;
    margin-bottom: 4px;
  }

  .proj-row {
    background: #f9fafb;
    border: 1.5px solid #e5e7eb;
    border-radius: 10px;
    padding: 12px 14px;
    display: grid;
    grid-template-columns: 160px 1fr auto auto;
    gap: 10px;
    align-items: start;
    flex-wrap: wrap;
  }
  .proj-row--active { border-color: #a5b4fc; background: #f5f3ff; }

  .proj-series-name {
    display: flex;
    align-items: flex-start;
    padding-top: 2px;
  }
  .series-pill {
    background: #dbeafe;
    color: #1e40af;
    padding: 3px 10px;
    border-radius: 20px;
    font-size: 12px;
    font-weight: 700;
    white-space: nowrap;
  }

  .proj-method, .proj-n, .proj-seed {
    display: flex;
    flex-direction: column;
    gap: 3px;
  }
  .proj-method select { min-width: 160px; }
  .proj-n input, .proj-seed input { width: 100px; }

  .proj-params {
    grid-column: 1 / -1;
    background: #fff;
    border: 1px solid #e5e7eb;
    border-radius: 8px;
    padding: 10px 12px;
  }
  .params-title {
    font-size: 11px;
    font-weight: 700;
    text-transform: uppercase;
    letter-spacing: .05em;
    color: #9ca3af;
    display: block;
    margin-bottom: 8px;
  }
  .params-grid {
    display: flex;
    gap: 12px;
    flex-wrap: wrap;
  }
  .param-field {
    display: flex;
    flex-direction: column;
    gap: 3px;
    font-size: 12px;
    font-weight: 600;
    color: #374151;
  }
  .param-field input { width: 80px; }

  .proj-script { display: flex; flex-direction: column; gap: 4px; }

  .hint      { display:block; font-size:11.5px; color:#9ca3af; font-weight:400; margin-top:2px; }
  .hint-warn { font-size:11.5px; color:#d97706; display:block; margin-top:3px; }

  .builder-footer {
    display: flex;
    justify-content: flex-end;
    gap: 10px;
    padding-top: 16px;
    border-top: 1px solid #e5e7eb;
  }
</style>
