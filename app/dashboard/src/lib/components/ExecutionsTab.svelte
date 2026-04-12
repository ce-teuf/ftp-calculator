<script lang="ts">
  import { compute, portfolios, curves, executions,
           type Portfolio, type RateCurve, type Execution, type ComputeResponse, type DiffResponse } from '../api/client.ts';
  import { exportExecutionExcel } from '../export/excel.ts';

  const METHODS = [
    { key:'stock',       label:'Stock (Weighted-Average)' },
    { key:'flux',        label:'Flux (Multi-Vintage)' },
    { key:'duration',    label:'Duration Method' },
    { key:'pool',        label:'Pool Method' },
    { key:'refinancing', label:'Refinancing / Forward Rate' },
    { key:'floating',    label:'Floating-Rate (double profil)' },
    { key:'behavioral',  label:'Behavioral Run-off (NMD)' },
    { key:'replicating', label:'Replicating Portfolio' },
    { key:'ldi',         label:'LDI (Pension / Insurance)' },
  ];

  let portfolioList = $state<Portfolio[]>([]);
  let curveList     = $state<RateCurve[]>([]);
  let history       = $state<Execution[]>([]);
  let running       = $state(false);
  let error         = $state('');
  let lastResult    = $state<ComputeResponse | null>(null);
  let selectedExec  = $state<Execution | null>(null);

  let form = $state({
    portfolio_id: '',
    method: 'stock',
    label: '',
    curve_ids: [] as string[],
    outstanding_json: '[[1000000],[500000]]',
    profiles_json: '[[1,0.8,0.5,0.2,0],[1,0.9,0.7,0.4,0]]',
    rates_json: '[[0.04,0.042,0.044,0.046],[0.04,0.042,0.044,0.046]]',
  });

  async function loadAll() {
    [portfolioList, curveList, history] = await Promise.all([
      portfolios.list(), curves.list(), executions.list(),
    ]);
    if (portfolioList.length > 0 && !form.portfolio_id) {
      form.portfolio_id = portfolioList[0].id;
    }
  }

  async function runCompute() {
    running = true; error = ''; lastResult = null;
    try {
      const outstanding = JSON.parse(form.outstanding_json);
      const outFlat = outstanding[0] instanceof Array
        ? outstanding.map((r: number[]) => r[0])
        : outstanding;
      const result = await compute({
        method:           form.method,
        portfolio_id:     form.portfolio_id,
        label:            form.label || undefined,
        curve_ids:        form.curve_ids.length > 0 ? form.curve_ids : undefined,
        outstanding_json: JSON.stringify(outFlat),
        profiles_json:    form.profiles_json,
        rates_json:       form.rates_json,
      });
      lastResult = result;
      history = await executions.list();
    } catch(e: any) { error = e.message; }
    finally { running = false; }
  }

  function selectExec(ex: Execution) {
    selectedExec = ex;
    if (ex.result_json) {
      try { lastResult = JSON.parse(ex.result_json); } catch { lastResult = null; }
    }
  }

  let replaying = $state(false);

  async function replay(ex: Execution) {
    replaying = true; error = '';
    try {
      const inputs = await executions.inputs(ex.id);
      form.method       = inputs.method;
      form.portfolio_id = inputs.portfolio_id;
      form.label        = `Rejouer — ${inputs.label ?? inputs.method}`;
      if (inputs.outstanding_json) form.outstanding_json = inputs.outstanding_json;
      if (inputs.profiles_json)    form.profiles_json    = inputs.profiles_json;
      if (inputs.rates_json)       form.rates_json       = inputs.rates_json;
      await runCompute();
    } catch(e: any) { error = e.message; }
    finally { replaying = false; }
  }

  function fmtRate(v: number) { return (v * 100).toFixed(3) + '%'; }
  function fmtCcy(v: number)  { return v.toLocaleString('fr-FR', {maximumFractionDigits:0}); }

  let diffA = $state('');
  let diffB = $state('');
  let diffResult = $state<DiffResponse | null>(null);
  let diffError  = $state('');
  let diffRunning = $state(false);

  async function runDiff() {
    if (!diffA || !diffB || diffA === diffB) return;
    diffRunning = true; diffError = ''; diffResult = null;
    try { diffResult = await executions.diff(diffA, diffB); }
    catch(e: any) { diffError = e.message; }
    finally { diffRunning = false; }
  }

  function fmtDelta(v: number | undefined, pct = true) {
    if (v == null) return '—';
    const sign = v >= 0 ? '+' : '';
    return pct ? sign + (v * 100).toFixed(3) + '%' : sign + v.toLocaleString('fr-FR', {maximumFractionDigits:0});
  }

  $effect(() => { loadAll(); });
</script>

<div class="tab-layout">
  <!-- ── Sidebar historique ── -->
  <aside class="sidebar">
    <div class="sidebar-header">
      <h3>Historique</h3>
      <span class="badge">{history.length}</span>
    </div>
    <div class="history-list">
      {#each history as ex}
        <div
          class="history-item"
          class:active={selectedExec?.id === ex.id}
          onclick={() => selectExec(ex)}
          role="button"
          tabindex="0"
          onkeydown={e => e.key === 'Enter' && selectExec(ex)}
        >
          <div class="hi-header">
            <span class="hi-method">{ex.method.toUpperCase()}</span>
            <span class="hi-status" class:ok={ex.status==='completed'} class:err={ex.status==='error'}>
              {ex.status === 'completed' ? '✓' : ex.status === 'error' ? '✗' : '…'}
            </span>
          </div>
          <div class="hi-label">{ex.label || ex.portfolio_id}</div>
          <div class="hi-date">{ex.created_at?.slice(0,16).replace('T',' ') ?? ''}</div>
          {#if ex.status === 'completed' && ex.outstanding_json}
            <button class="btn-xs" onclick={e => { e.stopPropagation(); replay(ex); }} disabled={replaying}>
              ↻ Rejouer
            </button>
          {/if}
        </div>
      {/each}
    </div>
  </aside>

  <!-- ── Panneau principal ── -->
  <main class="main-panel">

    <!-- Formulaire de calcul -->
    <section class="card">
      <h2>Nouveau calcul FTP</h2>
      {#if error}
        <div class="alert-error">{error}</div>
      {/if}

      <div class="form-grid">
        <label>
          Portefeuille
          <select bind:value={form.portfolio_id}>
            {#each portfolioList as p}
              <option value={p.id}>{p.name}</option>
            {/each}
          </select>
        </label>

        <label>
          Méthode
          <select bind:value={form.method}>
            {#each METHODS as m}
              <option value={m.key}>{m.label}</option>
            {/each}
          </select>
        </label>

        <label class="full-width">
          Libellé (optionnel)
          <input type="text" bind:value={form.label} placeholder="Ex: Pricing T4 2026" />
        </label>

        <label class="full-width">
          Encours JSON <span class="hint">(tableau de nombres)</span>
          <textarea rows="2" bind:value={form.outstanding_json} spellcheck="false"></textarea>
        </label>

        <label class="full-width">
          Profils JSON <span class="hint">(matrice N×P)</span>
          <textarea rows="3" bind:value={form.profiles_json} spellcheck="false"></textarea>
        </label>

        <label class="full-width">
          Taux marché JSON <span class="hint">(matrice N×(P-1))</span>
          <textarea rows="3" bind:value={form.rates_json} spellcheck="false"></textarea>
        </label>
      </div>

      <div class="form-actions">
        <button class="btn-primary" onclick={runCompute} disabled={running || !form.portfolio_id}>
          {running ? 'Calcul en cours…' : '▶ Lancer le calcul'}
        </button>
      </div>
    </section>

    <!-- Résultats -->
    {#if lastResult}
      <section class="card result-card">
        <div class="result-header">
          <h2>Résultats — {lastResult.method?.toUpperCase()}</h2>
          <button class="btn-sm" onclick={() => lastResult && exportExecutionExcel(lastResult)}>
            ⬇ Export Excel
          </button>
        </div>

        <div class="kpi-row">
          <div class="kpi">
            <div class="kpi-label">Taux FTP pondéré</div>
            <div class="kpi-value">{fmtRate(lastResult.weighted_ftp_rate)}</div>
          </div>
          <div class="kpi">
            <div class="kpi-label">Encours total</div>
            <div class="kpi-value">{fmtCcy(lastResult.total_outstanding)} €</div>
          </div>
          <div class="kpi">
            <div class="kpi-label">Intérêts FTP/mois</div>
            <div class="kpi-value">{fmtCcy(lastResult.total_ftp_int_monthly)} €</div>
          </div>
          <div class="kpi">
            <div class="kpi-label">Durée calcul</div>
            <div class="kpi-value">{lastResult.duration_ms} ms</div>
          </div>
        </div>

        {#if lastResult.ftp_rate && lastResult.ftp_rate.length > 0}
          <div class="matrix-scroll">
            <table>
              <thead>
                <tr>
                  <th>Position</th>
                  {#each lastResult.ftp_rate[0] as _, j}
                    <th>T{j}</th>
                  {/each}
                </tr>
              </thead>
              <tbody>
                {#each lastResult.ftp_rate as row, i}
                  <tr>
                    <td class="row-label">#{i+1}</td>
                    {#each row as v}
                      <td>{(v * 100).toFixed(3)}%</td>
                    {/each}
                  </tr>
                {/each}
              </tbody>
            </table>
          </div>
        {/if}
      </section>
    {/if}

    <!-- Diff A/B -->
    <section class="card">
      <h2>Comparer deux exécutions</h2>
      <div class="diff-selectors">
        <select bind:value={diffA}>
          <option value="">— Exécution A —</option>
          {#each history as ex}
            <option value={ex.id}>{ex.created_at?.slice(0,16).replace('T',' ')} — {ex.method} — {ex.label || ex.id.slice(0,8)}</option>
          {/each}
        </select>
        <select bind:value={diffB}>
          <option value="">— Exécution B —</option>
          {#each history as ex}
            <option value={ex.id}>{ex.created_at?.slice(0,16).replace('T',' ')} — {ex.method} — {ex.label || ex.id.slice(0,8)}</option>
          {/each}
        </select>
        <button class="btn-primary" onclick={runDiff} disabled={diffRunning || !diffA || !diffB}>
          {diffRunning ? 'Comparaison…' : 'Comparer'}
        </button>
      </div>

      {#if diffError}
        <div class="alert-error">{diffError}</div>
      {/if}

      {#if diffResult}
        <table class="diff-table">
          <thead>
            <tr><th>KPI</th><th>A</th><th>B</th><th>Δ</th></tr>
          </thead>
          <tbody>
            <tr>
              <td>Méthode</td>
              <td>{diffResult.a.method}</td>
              <td>{diffResult.b.method}</td>
              <td>—</td>
            </tr>
            <tr>
              <td>Taux FTP pondéré</td>
              <td>{diffResult.a.kpis ? fmtRate(diffResult.a.kpis.weighted_ftp_rate) : '—'}</td>
              <td>{diffResult.b.kpis ? fmtRate(diffResult.b.kpis.weighted_ftp_rate) : '—'}</td>
              <td class:pos={diffResult.delta_weighted_ftp_rate != null && diffResult.delta_weighted_ftp_rate > 0}
                  class:neg={diffResult.delta_weighted_ftp_rate != null && diffResult.delta_weighted_ftp_rate < 0}>
                {fmtDelta(diffResult.delta_weighted_ftp_rate)}
              </td>
            </tr>
            <tr>
              <td>Encours total</td>
              <td>{diffResult.a.kpis ? fmtCcy(diffResult.a.kpis.total_outstanding) + ' €' : '—'}</td>
              <td>{diffResult.b.kpis ? fmtCcy(diffResult.b.kpis.total_outstanding) + ' €' : '—'}</td>
              <td>{fmtDelta(diffResult.delta_total_outstanding, false)}</td>
            </tr>
            <tr>
              <td>Intérêts FTP/mois</td>
              <td>{diffResult.a.kpis ? fmtCcy(diffResult.a.kpis.total_ftp_int_monthly) + ' €' : '—'}</td>
              <td>{diffResult.b.kpis ? fmtCcy(diffResult.b.kpis.total_ftp_int_monthly) + ' €' : '—'}</td>
              <td>{fmtDelta(diffResult.delta_ftp_int_monthly, false)}</td>
            </tr>
          </tbody>
        </table>
      {/if}
    </section>
  </main>
</div>

<style>
.tab-layout { display: flex; gap: 1rem; height: 100%; }
.sidebar { width: 260px; flex-shrink: 0; display: flex; flex-direction: column; gap: .5rem; }
.sidebar-header { display: flex; align-items: center; justify-content: space-between; padding: .5rem 0; }
.sidebar-header h3 { margin: 0; font-size: .95rem; }
.badge { background: #1a73e8; color: #fff; border-radius: 20px; padding: .1rem .5rem; font-size: .75rem; }
.history-list { flex: 1; overflow-y: auto; display: flex; flex-direction: column; gap: .4rem; }
.history-item { background: #fff; border: 1px solid #e0e0e0; border-radius: 8px; padding: .6rem .8rem;
  cursor: pointer; transition: border-color .15s; }
.history-item:hover { border-color: #1a73e8; }
.history-item.active { border-color: #1a73e8; background: #e8f0fe; }
.hi-header { display: flex; justify-content: space-between; align-items: center; margin-bottom: .2rem; }
.hi-method { font-weight: 600; font-size: .8rem; color: #1a73e8; }
.hi-status.ok { color: #34a853; }
.hi-status.err { color: #ea4335; }
.hi-label { font-size: .8rem; color: #333; white-space: nowrap; overflow: hidden; text-overflow: ellipsis; }
.hi-date { font-size: .72rem; color: #888; }
.btn-xs { margin-top: .4rem; font-size: .72rem; padding: .2rem .5rem; border: 1px solid #1a73e8;
  background: #fff; color: #1a73e8; border-radius: 4px; cursor: pointer; }
.btn-xs:hover { background: #e8f0fe; }

.main-panel { flex: 1; overflow-y: auto; display: flex; flex-direction: column; gap: 1rem; }
.card { background: #fff; border-radius: 12px; padding: 1.25rem; box-shadow: 0 1px 4px rgba(0,0,0,.08); }
.card h2 { margin: 0 0 1rem; font-size: 1rem; font-weight: 600; }
.alert-error { background: #fce8e6; color: #c5221f; padding: .6rem .9rem; border-radius: 6px; margin-bottom: .8rem; font-size: .85rem; }

.form-grid { display: grid; grid-template-columns: 1fr 1fr; gap: .75rem; }
.form-grid label { display: flex; flex-direction: column; gap: .3rem; font-size: .85rem; color: #555; }
.form-grid .full-width { grid-column: 1 / -1; }
.form-grid select, .form-grid input, .form-grid textarea {
  border: 1px solid #ddd; border-radius: 6px; padding: .4rem .6rem; font-size: .85rem;
  font-family: monospace; resize: vertical;
}
.hint { font-size: .75rem; color: #999; font-weight: 400; }
.form-actions { margin-top: 1rem; }
.btn-primary { background: #1a73e8; color: #fff; border: none; border-radius: 8px;
  padding: .5rem 1.2rem; font-size: .9rem; cursor: pointer; }
.btn-primary:hover { background: #1557b0; }
.btn-primary:disabled { background: #b0c4de; cursor: not-allowed; }
.btn-sm { background: #fff; border: 1px solid #1a73e8; color: #1a73e8;
  border-radius: 6px; padding: .3rem .8rem; font-size: .82rem; cursor: pointer; }
.btn-sm:hover { background: #e8f0fe; }

.result-card { border-left: 4px solid #34a853; }
.result-header { display: flex; justify-content: space-between; align-items: center; margin-bottom: 1rem; }
.result-header h2 { margin: 0; }
.kpi-row { display: grid; grid-template-columns: repeat(4, 1fr); gap: .75rem; margin-bottom: 1rem; }
.kpi { background: #f8f9fa; border-radius: 8px; padding: .75rem; text-align: center; }
.kpi-label { font-size: .75rem; color: #666; margin-bottom: .3rem; }
.kpi-value { font-size: 1.1rem; font-weight: 700; color: #1a73e8; }
.matrix-scroll { overflow-x: auto; }
.matrix-scroll table { border-collapse: collapse; font-size: .8rem; width: 100%; }
.matrix-scroll th, .matrix-scroll td { border: 1px solid #e8e8e8; padding: .3rem .5rem; text-align: right; }
.matrix-scroll th { background: #f0f2f5; font-weight: 600; }
.row-label { font-weight: 600; color: #555; text-align: left; }

.diff-selectors { display: flex; gap: .75rem; margin-bottom: 1rem; flex-wrap: wrap; }
.diff-selectors select { flex: 1; min-width: 200px; border: 1px solid #ddd; border-radius: 6px;
  padding: .4rem .6rem; font-size: .85rem; }
.diff-table { width: 100%; border-collapse: collapse; font-size: .85rem; }
.diff-table th, .diff-table td { border: 1px solid #e8e8e8; padding: .4rem .7rem; }
.diff-table th { background: #f0f2f5; font-weight: 600; }
.diff-table td:first-child { font-weight: 500; color: #444; }
.pos { color: #c5221f; font-weight: 600; }
.neg { color: #34a853; font-weight: 600; }
</style>
