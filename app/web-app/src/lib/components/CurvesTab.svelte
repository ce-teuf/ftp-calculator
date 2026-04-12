<script lang="ts">
  import { curves, rateSeries, type RateCurve, type SeriesInfo } from '../api/client.ts';
  import CurveLab from './CurveLab.svelte';
  import { onMount } from 'svelte';

  // ── 14 CoF components ──────────────────────────────────────────────────────
  const COMPONENTS = [
    { key: 'base_rate',      label: 'Base Rate (OIS/€STR)',      color: '#4a4aff' },
    { key: 'credit_spread',  label: 'Credit Spread',             color: '#e53935' },
    { key: 'tlp',            label: 'Term Liquidity Premium',    color: '#00897b' },
    { key: 'clp',            label: 'Contingent Liquidity',      color: '#8e24aa' },
    { key: 'oas',            label: 'OAS (prépaiement)',         color: '#f57c00' },
    { key: 'capital_charge', label: 'Capital Charge',            color: '#c62828' },
    { key: 'basis_risk',     label: 'Basis Risk',                color: '#1565c0' },
    { key: 'operational',    label: 'Operational Risk',          color: '#558b2f' },
    { key: 'xva',            label: 'XVA (CVA/MVA/KVA)',        color: '#6d4c41' },
    { key: 'country_risk',   label: 'Country Risk',              color: '#37474f' },
    { key: 'concentration',  label: 'Concentration Add-on',      color: '#e91e63' },
    { key: 'mrel_levy',      label: 'MREL / FDIC Levy',          color: '#ff6f00' },
    { key: 'incentive',      label: 'Incentive Premium',         color: '#00695c' },
    { key: 'rollover',       label: 'Rollover Risk',             color: '#4527a0' },
  ];

  const DEFAULT_TENORS = ['1M','3M','6M','1Y','2Y','3Y','5Y','7Y','10Y','15Y','20Y','30Y'];

  // ── View: 'library' | 'builder' | 'lab' ──────────────────────────────────
  let view = $state<'library' | 'builder' | 'lab'>('library');

  // ── Library state ─────────────────────────────────────────────────────────
  let list = $state<RateCurve[]>([]);
  let seriesNames = $state<SeriesInfo[]>([]);
  let loading = $state(false);
  let error = $state('');
  let showForm = $state(false);
  let form = $state({
    name: '', component: COMPONENTS[0].key, currency: 'EUR', valid_from: '',
    tenors: DEFAULT_TENORS.join(','),
    values: Array(DEFAULT_TENORS.length).fill(0.04).join(','),
    notes: '',
    series_name: '',
  });

  async function load() {
    loading = true; error = '';
    try { list = await curves.list(); }
    catch(e: any) { error = e.message; }
    finally { loading = false; }
  }

  async function save() {
    const tenorsArr = form.tenors.split(',').map(s => s.trim());
    const valuesArr = form.values.split(',').map(s => parseFloat(s.trim()));
    try {
      await curves.create({
        name: form.name, component: form.component, currency: form.currency,
        valid_from: form.valid_from || undefined,
        tenors_json: JSON.stringify(tenorsArr),
        values_json: JSON.stringify(valuesArr),
        notes: form.notes || undefined,
        series_name: form.series_name || undefined,
      });
      showForm = false;
      form = { name:'', component: COMPONENTS[0].key, currency:'EUR', valid_from:'',
               tenors: DEFAULT_TENORS.join(','),
               values: Array(DEFAULT_TENORS.length).fill(0.04).join(','), notes:'',
               series_name: '' };
      await load();
    } catch(e: any) { error = e.message; }
  }

  async function del(id: string) {
    if (!confirm('Delete this curve?')) return;
    await curves.delete(id); await load();
  }
  function approve(id: string) { curves.update(id, { status: 'approved' }).then(load); }
  function formatValues(json: string) {
    try { return (JSON.parse(json) as number[]).map(v => (v*100).toFixed(3)+'%').join('  '); }
    catch { return json; }
  }

  // ── Builder state ─────────────────────────────────────────────────────────
  let builderName = $state('CoF All-In 2026');
  let builderCurrency = $state('EUR');
  let builderDate = $state('');
  let builderTenors = $state([...DEFAULT_TENORS]);
  // grid[component_idx][tenor_idx] = rate (%)
  let grid = $state<number[][]>(
    COMPONENTS.map(() => Array(DEFAULT_TENORS.length).fill(0))
  );

  // Sums per tenor
  const totals = $derived(() =>
    builderTenors.map((_, j) =>
      COMPONENTS.reduce((s, _, i) => s + (grid[i]?.[j] ?? 0), 0)
    )
  );

  const maxTotal = $derived(() => Math.max(...totals(), 0.001));

  function addTenor() {
    builderTenors = [...builderTenors, ''];
    grid = grid.map(row => [...row, 0]);
  }

  function removeTenor(j: number) {
    builderTenors = builderTenors.filter((_, i) => i !== j);
    grid = grid.map(row => row.filter((_, i) => i !== j));
  }

  async function saveBuilder() {
    const vals = totals().map(v => v / 100);
    try {
      await curves.create({
        name: builderName,
        component: 'all_in_cof',
        currency: builderCurrency,
        valid_from: builderDate || undefined,
        tenors_json: JSON.stringify(builderTenors),
        values_json: JSON.stringify(vals),
        notes: `Built from 14 components: ${COMPONENTS.map(c=>c.key).join(', ')}`,
      });
      alert('Curve saved in the library.');
      view = 'library'; await load();
    } catch(e: any) { error = (e as any).message; }
  }

  function importCSV(e: Event) {
    const file = (e.target as HTMLInputElement).files?.[0];
    if (!file) return;
    const reader = new FileReader();
    reader.onload = (ev) => {
      const text = ev.target?.result as string;
      const lines = text.trim().split('\n').filter(l => l.trim());
      if (lines.length === 0) return;
      // First line = header (tenors)
      const headers = lines[0].split(',').map(s => s.trim()).slice(1);
      builderTenors = headers;
      const newGrid = COMPONENTS.map(() => Array(headers.length).fill(0));
      for (let li = 1; li < lines.length; li++) {
        const cells = lines[li].split(',');
        const compKey = cells[0].trim();
        const ci = COMPONENTS.findIndex(c => c.key === compKey);
        if (ci < 0) continue;
        for (let j = 1; j < cells.length; j++) {
          newGrid[ci][j-1] = parseFloat(cells[j]) || 0;
        }
      }
      grid = newGrid;
    };
    reader.readAsText(file);
  }

  function exportCSV() {
    const rows = ['component,' + builderTenors.join(',')];
    COMPONENTS.forEach((c, i) => rows.push(c.key + ',' + (grid[i] ?? []).join(',')));
    rows.push('TOTAL,' + totals().join(','));
    const blob = new Blob([rows.join('\n')], { type: 'text/csv' });
    const a = document.createElement('a');
    a.href = URL.createObjectURL(blob);
    a.download = `${builderName.replace(/\s+/g,'-')}.csv`;
    a.click();
  }

  onMount(async () => {
    load();
    try {
      const res = await rateSeries.names();
      seriesNames = res.series;
    } catch { /**/ }
  });
</script>

<div class="tab-content">
  <div class="tab-header">
    <h2>CoF Curve Builder</h2>
    <div class="view-toggle">
      <button class="toggle-btn" class:active={view === 'library'} onclick={() => view = 'library'}>
        Library
      </button>
      <button class="toggle-btn" class:active={view === 'builder'} onclick={() => view = 'builder'}>
        14-component Builder
      </button>
      <button class="toggle-btn toggle-lab" class:active={view === 'lab'} onclick={() => view = 'lab'}>
        🧪 Python Lab
      </button>
    </div>
    {#if view === 'library'}
      <button class="btn-primary" onclick={() => showForm = !showForm}>
        {showForm ? '✕ Cancel' : '+ New curve'}
      </button>
    {/if}
  </div>

  {#if error}<div class="alert-error">{error}</div>{/if}

  <!-- ── Bibliothèque ─────────────────────────────────────────────────────── -->
  {#if view === 'library'}
    {#if showForm}
      <div class="card form-card">
        <h3>Nouvelle courbe (composante unique)</h3>
        <div class="form-grid">
          <label>Nom <input bind:value={form.name} placeholder="SOFR OIS 2026-04" /></label>
          <label>Composante
            <select bind:value={form.component}>
              {#each COMPONENTS as c}<option value={c.key}>{c.label}</option>{/each}
              <option value="all_in_cof">CoF All-In (agrégée)</option>
            </select>
          </label>
          <label>
            Série historique sous-jacente
            <select bind:value={form.series_name}>
              <option value="">— Aucune (spread / saisie manuelle) —</option>
              {#each seriesNames as s}
                <option value={s.name}>{s.name} ({s.currency} · {s.component})</option>
              {/each}
            </select>
          </label>
          <label>Devise <input bind:value={form.currency} style="width:80px" /></label>
          <label>Date effective <input type="date" bind:value={form.valid_from} /></label>
        </div>
        <label class="full-width">
          Tenors (séparés par virgule)
          <input bind:value={form.tenors} placeholder="1M,3M,6M,1Y,2Y,5Y,10Y" />
        </label>
        <label class="full-width">
          Taux décimaux (ex: 0.0450 = 4.50%)
          <input bind:value={form.values} />
        </label>
        <label class="full-width">Notes <textarea bind:value={form.notes} rows="2"></textarea></label>
        <div class="form-actions"><button class="btn-primary" onclick={save}>Enregistrer</button></div>
      </div>
    {/if}

    {#if loading}
      <p class="loading">Chargement...</p>
    {:else if list.length === 0}
      <div class="empty-state">
        <p>Aucune courbe. Utilisez le Constructeur 14 composantes ou créez une courbe simple.</p>
      </div>
    {:else}
      <div class="curves-table">
        <table>
          <thead>
            <tr><th>Nom</th><th>Composante</th><th>Série</th><th>Devise</th><th>Taux</th><th>Statut</th><th>Date</th><th>Actions</th></tr>
          </thead>
          <tbody>
            {#each list as c}
              {@const comp = COMPONENTS.find(x => x.key === c.component)}
              <tr class:approved={c.status === 'approved'}>
                <td><strong>{c.name}</strong></td>
                <td>
                  <span class="comp-badge" style={comp ? `background:${comp.color}22;color:${comp.color}` : ''}>
                    {comp?.label ?? c.component}
                  </span>
                </td>
                <td>
                  {#if c.series_name}
                    <span class="series-tag">{c.series_name}</span>
                  {:else}
                    <span class="no-series">—</span>
                  {/if}
                </td>
                <td>{c.currency}</td>
                <td class="mono small">{formatValues(c.values_json)}</td>
                <td><span class="badge badge-{c.status}">{c.status}</span></td>
                <td class="small">{c.created_at?.slice(0,10)}</td>
                <td class="actions">
                  {#if c.status === 'draft'}
                    <button class="btn-sm btn-success" onclick={() => approve(c.id)}>Approuver</button>
                  {/if}
                  <button class="btn-sm btn-danger" onclick={() => del(c.id)}>✕</button>
                </td>
              </tr>
            {/each}
          </tbody>
        </table>
      </div>
    {/if}

  <!-- ── Python Lab ──────────────────────────────────────────────────────── -->
  {:else if view === 'lab'}
    <div class="lab-wrap">
      <CurveLab />
    </div>

  <!-- ── Constructeur 14 composantes ─────────────────────────────────────── -->
  {:else}
    <div class="builder">
      <!-- Header controls -->
      <div class="builder-header card">
        <div class="builder-meta">
          <label>Nom de la courbe CoF
            <input bind:value={builderName} style="width:260px" />
          </label>
          <label>Devise
            <input bind:value={builderCurrency} style="width:80px" />
          </label>
          <label>Date effective
            <input type="date" bind:value={builderDate} />
          </label>
        </div>
        <div class="builder-actions">
          <label class="btn-sm file-import">
            ⬆ Importer CSV
            <input type="file" accept=".csv" style="display:none" onchange={importCSV} />
          </label>
          <button class="btn-sm" onclick={exportCSV}>⬇ Exporter CSV</button>
          <button class="btn-primary" onclick={saveBuilder}>💾 Sauvegarder dans bibliothèque</button>
        </div>
      </div>

      <!-- Stacked bar chart -->
      <div class="card chart-card">
        <h3>Visualisation CoF par tenor</h3>
        <div class="bar-chart">
          {#each builderTenors as t, j}
            <div class="bar-group">
              <div class="bar-stack">
                {#each COMPONENTS as comp, i}
                  {@const h = totals()[j] > 0 ? ((grid[i]?.[j] ?? 0) / maxTotal()) * 180 : 0}
                  {#if (grid[i]?.[j] ?? 0) > 0}
                    <div class="bar-segment"
                         style="height:{h}px;background:{comp.color}"
                         title="{comp.label}: {(grid[i]?.[j] ?? 0).toFixed(3)}%">
                    </div>
                  {/if}
                {/each}
              </div>
              <div class="bar-total">{totals()[j].toFixed(2)}%</div>
              <div class="bar-label">{t}</div>
            </div>
          {/each}
        </div>
        <!-- Legend -->
        <div class="legend">
          {#each COMPONENTS as comp, i}
            {#if COMPONENTS.some((_, ci) => grid[ci]?.some(v => v > 0) && ci === i)}
              <div class="legend-item">
                <div class="legend-dot" style="background:{comp.color}"></div>
                <span>{comp.label}</span>
              </div>
            {/if}
          {/each}
        </div>
      </div>

      <!-- Input grid -->
      <div class="card grid-card">
        <div class="grid-header">
          <h3>Grille de saisie (valeurs en %)</h3>
          <div style="display:flex;gap:.5rem">
            <button class="btn-sm" onclick={addTenor}>+ Tenor</button>
          </div>
        </div>
        <div class="grid-scroll">
          <table class="input-grid">
            <thead>
              <tr>
                <th class="comp-col">Composante</th>
                {#each builderTenors as t, j}
                  <th class="tenor-th">
                    <input class="tenor-input" bind:value={builderTenors[j]} />
                    <button class="del-tenor" onclick={() => removeTenor(j)}>✕</button>
                  </th>
                {/each}
              </tr>
            </thead>
            <tbody>
              {#each COMPONENTS as comp, i}
                <tr>
                  <td class="comp-label">
                    <div class="comp-dot" style="background:{comp.color}"></div>
                    {comp.label}
                  </td>
                  {#each builderTenors as _, j}
                    <td>
                      <input class="rate-input"
                             type="number" step="0.001" min="0"
                             bind:value={grid[i][j]} />
                    </td>
                  {/each}
                </tr>
              {/each}
              <tr class="total-row">
                <td class="comp-label"><strong>TOTAL CoF</strong></td>
                {#each totals() as t}
                  <td class="total-cell">{t.toFixed(3)}%</td>
                {/each}
              </tr>
            </tbody>
          </table>
        </div>
      </div>
    </div>
  {/if}
</div>

<style>
  .tab-header { display:flex; justify-content:space-between; align-items:center; margin-bottom:1.5rem; gap:1rem; }
  .view-toggle { display:flex; gap:4px; background:#eee; border-radius:6px; padding:3px; }
  .toggle-btn { border:none; background:transparent; padding:.35rem .75rem; border-radius:4px; cursor:pointer; font-size:.85rem; color:#555; }
  .toggle-btn.active { background:white; color:#4a4aff; font-weight:600; box-shadow:0 1px 3px rgba(0,0,0,.15); }
  .form-card { padding:1.5rem; margin-bottom:1.5rem; }
  .form-grid { display:grid; grid-template-columns:1fr 1fr 1fr 1fr; gap:1rem; margin-bottom:1rem; }
  .full-width { display:flex; flex-direction:column; gap:.25rem; margin-bottom:1rem; }
  .form-actions { display:flex; justify-content:flex-end; }
  .curves-table table { width:100%; border-collapse:collapse; background:white;
    border-radius:8px; overflow:hidden; box-shadow:0 1px 3px rgba(0,0,0,.1); }
  .curves-table :global(th) { background:#f0f0f0; padding:.75rem 1rem; text-align:left; font-size:.85rem; color:#555; }
  .curves-table :global(td) { padding:.75rem 1rem; border-top:1px solid #eee; vertical-align:middle; }
  .comp-badge { padding:2px 8px; border-radius:12px; font-size:.78rem; font-weight:500; white-space:nowrap; }
  .series-tag { background:#dbeafe; color:#1e40af; padding:2px 7px; border-radius:6px; font-size:.78rem; font-weight:600; }
  .no-series  { color:#d1d5db; font-size:.85rem; }
  .mono { font-family:monospace; }
  .small { font-size:.8rem; }
  .actions { display:flex; gap:.5rem; }

  /* Lab */
  .lab-wrap { height: calc(100vh - 160px); min-height: 500px; border-radius: 10px; overflow: hidden; }
  .toggle-lab { background: #1a0a2e !important; color: #a78bfa !important; }
  .toggle-lab.active { background: #4c1d95 !important; color: white !important; border-color: #7c3aed !important; }

  /* Builder */
  .builder { display:flex; flex-direction:column; gap:1.5rem; }
  .builder-header { display:flex; justify-content:space-between; align-items:flex-end; padding:1.25rem; flex-wrap:wrap; gap:1rem; }
  .builder-meta { display:flex; gap:1rem; align-items:flex-end; flex-wrap:wrap; }
  .builder-actions { display:flex; gap:.5rem; align-items:center; }
  .file-import { cursor:pointer; display:inline-flex; align-items:center; }

  .chart-card { padding:1.25rem; }
  .chart-card h3 { font-size:.9rem; margin-bottom:1rem; color:#333; }
  .bar-chart { display:flex; align-items:flex-end; gap:8px; overflow-x:auto; padding-bottom:.5rem; min-height:220px; }
  .bar-group { display:flex; flex-direction:column; align-items:center; min-width:44px; }
  .bar-stack { display:flex; flex-direction:column-reverse; width:36px; border-radius:3px 3px 0 0; overflow:hidden; }
  .bar-segment { width:100%; transition:height .2s; }
  .bar-total { font-size:.7rem; font-weight:600; color:#333; margin-top:2px; }
  .bar-label { font-size:.72rem; color:#888; margin-top:2px; }
  .legend { display:flex; flex-wrap:wrap; gap:.5rem 1rem; margin-top:1rem; }
  .legend-item { display:flex; align-items:center; gap:.3rem; font-size:.75rem; color:#555; }
  .legend-dot { width:10px; height:10px; border-radius:50%; flex-shrink:0; }

  .grid-card { padding:1.25rem; }
  .grid-header { display:flex; justify-content:space-between; align-items:center; margin-bottom:.75rem; }
  .grid-header h3 { font-size:.9rem; color:#333; }
  .grid-scroll { overflow-x:auto; }
  .input-grid { border-collapse:collapse; width:max-content; min-width:100%; }
  .input-grid th, .input-grid td { border:1px solid #eee; padding:.25rem .3rem; }
  .comp-col { min-width:200px; background:#fafafa; }
  .tenor-th { text-align:center; min-width:70px; background:#f5f5f5; padding:.3rem; }
  .tenor-input { width:60px; border:none; background:transparent; text-align:center;
    font-size:.8rem; font-weight:500; }
  .del-tenor { font-size:.6rem; color:#ccc; border:none; background:none; cursor:pointer; padding:0; }
  .del-tenor:hover { color:#e53935; }
  .comp-label { display:flex; align-items:center; gap:.4rem; font-size:.82rem;
    padding:.3rem .5rem; min-width:200px; background:#fafafa; }
  .comp-dot { width:8px; height:8px; border-radius:50%; flex-shrink:0; }
  .rate-input { width:65px; border:1px solid #eee; border-radius:3px; padding:.2rem .25rem;
    font-size:.82rem; text-align:right; }
  .rate-input:focus { outline:none; border-color:#4a4aff; }
  .total-row { background:#f0f4ff; font-weight:600; }
  .total-cell { text-align:right; font-family:monospace; font-size:.85rem; color:#1a73e8;
    padding:.35rem .5rem; }
</style>
