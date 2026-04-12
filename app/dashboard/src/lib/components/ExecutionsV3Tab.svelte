<script lang="ts">
  import { onMount } from 'svelte';
  import { executionsV3, studies } from '../api/client';
  import type { ExecutionV3, ExecutionV3Detail, Study } from '../api/client';
  import { Play, Trash2, CheckCircle, AlertCircle, Clock, ChevronDown, ChevronUp } from '@lucide/svelte';

  type View = 'list' | 'run' | 'detail';
  let view       = $state<View>('list');
  let execList   = $state<ExecutionV3[]>([]);
  let studyList  = $state<Study[]>([]);
  let selected   = $state<ExecutionV3Detail | null>(null);
  let loading    = $state(true);
  let error      = $state('');

  // ── Run form ────────────────────────────────────────────────────────────────
  let rStudyId = $state('');
  let rLabel   = $state('');
  let rMethod  = $state('stock');
  let running  = $state(false);
  let runError = $state('');

  const METHODS = [
    { value: 'stock',       label: 'Stock' },
    { value: 'flux',        label: 'Flux' },
    { value: 'duration',    label: 'Duration' },
    { value: 'pool',        label: 'Pool' },
    { value: 'refinancing', label: 'Refinancing' },
    { value: 'floating',    label: 'Floating' },
  ];

  // ── Detail expand state ─────────────────────────────────────────────────────
  let expandedLinker = $state<string | null>(null);

  async function load() {
    loading = true; error = '';
    try {
      [execList, studyList] = await Promise.all([executionsV3.list(), studies.list()]);
    } catch (e: any) { error = e.message; }
    finally { loading = false; }
  }

  async function runExecution() {
    runError = '';
    if (!rStudyId) { runError = 'Select a study'; return; }
    running = true;
    try {
      const res = await executionsV3.run({
        study_id: rStudyId,
        label: rLabel || undefined,
        method: rMethod,
      });
      await load();
      openDetail(res.id);
    } catch (e: any) { runError = e.message; }
    finally { running = false; }
  }

  async function openDetail(id: string) {
    selected = null;
    view = 'detail';
    expandedLinker = null;
    try { selected = await executionsV3.get(id); }
    catch (e: any) { error = e.message; }
  }

  async function deleteExec(id: string) {
    if (!confirm('Delete this execution?')) return;
    await executionsV3.delete(id);
    if (selected?.id === id) { selected = null; view = 'list'; }
    await load();
  }

  function statusIcon(s: string) {
    if (s === 'completed') return '✅';
    if (s === 'error')     return '❌';
    return '⏳';
  }

  function fmt(n: number, decimals = 4) {
    return (n * 100).toFixed(decimals) + '%';
  }
  function fmtM(n: number) {
    if (Math.abs(n) >= 1e9) return (n / 1e9).toFixed(2) + ' Mrd';
    if (Math.abs(n) >= 1e6) return (n / 1e6).toFixed(1) + ' M';
    return n.toFixed(0);
  }

  onMount(load);
</script>

<div class="tab-content">
  <div class="tab-header">
    <h2>Executions</h2>
    <div style="display:flex;gap:8px">
      {#if view !== 'list'}
        <button class="btn-sm" onclick={() => { view='list'; selected=null; }}>← Back</button>
      {/if}
      {#if view === 'list'}
        <button class="btn-primary" onclick={() => { view='run'; runError=''; }}>
          <Play size={14}/> New execution
        </button>
      {/if}
    </div>
  </div>

  {#if error}<div class="alert-error">{error}</div>{/if}

  <!-- ── List ──────────────────────────────────────────────────────────────── -->
  {#if view === 'list'}
    {#if loading}
<p class="loading">Loading…</p>
      {:else if execList.length === 0}
        <div class="empty-state">
          <p>No execution.</p>
          <p>Run an execution by selecting a study.</p>
          <button class="btn-primary" onclick={() => view='run'}><Play size={14}/> Launch</button>
      </div>
    {:else}
      <div class="exec-table-wrap card">
        <table class="exec-table">
          <thead>
            <tr>
              <th>Statut</th>
              <th>Label</th>
              <th>Study</th>
              <th>Method</th>
              <th>Duration</th>
              <th>Date</th>
              <th></th>
            </tr>
          </thead>
          <tbody>
            {#each execList as e}
              <tr class="exec-row" onclick={() => openDetail(e.id)}>
                <td><span class="status-pill status-{e.status}">{statusIcon(e.status)} {e.status}</span></td>
                <td>{e.label ?? '—'}</td>
                <td>{e.study_name ?? '—'}</td>
                <td><span class="tag">{e.method}</span></td>
                <td class="mono">{e.duration_ms != null ? e.duration_ms + ' ms' : '—'}</td>
                <td class="mono">{e.created_at.slice(0,16).replace('T',' ')}</td>
                <td onclick={ev => ev.stopPropagation()}>
                  <button class="btn-sm btn-danger" onclick={() => deleteExec(e.id)}>
                    <Trash2 size={12}/>
                  </button>
                </td>
              </tr>
            {/each}
          </tbody>
        </table>
      </div>
    {/if}

  <!-- ── Run form ────────────────────────────────────────────────────────────── -->
  {:else if view === 'run'}
    <div class="card run-card">
      <h3 class="form-title">Launch an execution</h3>
      <div class="form-grid">
        <label style="grid-column:1/-1">
Study to execute
          <select bind:value={rStudyId}>
            <option value="">-- Choose a study --</option>
            {#each studyList as s}
              <option value={s.id}>{s.name}</option>
            {/each}
          </select>
          {#if studyList.length === 0}
            <span class="hint-warn">No study available — create one in the Studies tab.</span>
          {/if}
        </label>

        <label>
          FTP method
          <select bind:value={rMethod}>
            {#each METHODS as m}
              <option value={m.value}>{m.label}</option>
            {/each}
          </select>
        </label>

        <label>
          Label (optional)
          <input bind:value={rLabel} placeholder="Ex: Run baseline Q1 2025"/>
        </label>
      </div>

      {#if runError}<div class="alert-error">{runError}</div>{/if}

      <div class="form-footer">
        <button class="btn-sm" onclick={() => view='list'}>Cancel</button>
        <button class="btn-primary" onclick={runExecution} disabled={running}>
          <Clock size={14}/> Calculating…
          <Play size={14}/> Launch
          
        </button>
      </div>
    </div>

  <!-- ── Detail ─────────────────────────────────────────────────────────────── -->
  {:else if view === 'detail'}
    {#if !selected}
      <p class="loading">Chargement…</p>
    {:else}
      <div class="detail-header card">
        <div class="dh-row">
          <span class="status-pill status-{selected.status}">{statusIcon(selected.status)} {selected.status}</span>
          <strong>{selected.label ?? selected.id.slice(0,8)}</strong>
          {#if selected.study_name}<span class="dh-study">Study: {selected.study_name}</span>{/if}
          <span class="tag">{selected.method}</span>
          {#if selected.duration_ms}<span class="dh-dur">{selected.duration_ms} ms</span>{/if}
        </div>
        {#if selected.error}
          <div class="alert-error" style="margin-top:10px">{selected.error}</div>
        {/if}
      </div>

      {#if selected.result?.linkers?.length}
        <!-- Global KPI summary -->
        {@const allTimes = selected.result.linkers.flatMap((l: any) => l.analysis_times ?? [])}
        {@const validTimes = allTimes.filter((t: any) => t.kpis)}
        {#if validTimes.length > 0}
          {@const avgFtp = validTimes.reduce((s: number, t: any) => s + (t.kpis.weighted_ftp_rate ?? 0), 0) / validTimes.length}
          {@const totalOut = validTimes.reduce((s: number, t: any) => s + (t.kpis.total_outstanding ?? 0), 0) / validTimes.length}
          <div class="kpi-strip">
            <div class="kpi-card">
              <div class="kpi-val">{fmt(avgFtp)}</div>
              <div class="kpi-lbl">Taux FTP moyen pondéré</div>
            </div>
            <div class="kpi-card">
              <div class="kpi-val">{fmtM(totalOut)}</div>
              <div class="kpi-lbl">Encours moyen</div>
            </div>
            <div class="kpi-card">
              <div class="kpi-val">{validTimes.length}</div>
              <div class="kpi-lbl">Dates d'analyse</div>
            </div>
            <div class="kpi-card">
              <div class="kpi-val">{selected.result.linkers.length}</div>
              <div class="kpi-lbl">Linkers</div>
            </div>
          </div>
        {/if}

        <!-- Per-linker results -->
        <div class="linkers-results">
          {#each selected.result.linkers as lnk}
            <div class="lr-card card">
              <button
                class="lr-header"
                onclick={() => expandedLinker = expandedLinker === lnk.linker_id ? null : lnk.linker_id}
              >
                <strong>{lnk.label ?? lnk.linker_name ?? lnk.linker_id}</strong>
                <div class="lr-header-right">
                  {#if lnk.error}
                    <AlertCircle size={14} color="#ef4444"/>
                    <span class="lr-err">{lnk.error}</span>
                  {:else if lnk.analysis_times?.length}
                    {@const last = lnk.analysis_times[lnk.analysis_times.length - 1]}
                    <span class="lr-kpi">{fmt(last?.kpis?.weighted_ftp_rate ?? 0)}</span>
                    <span class="lr-sub">{lnk.analysis_times.length} dates</span>
                  {/if}
                  {#if expandedLinker === lnk.linker_id}
                    <ChevronUp size={14}/>
                  {:else}
                    <ChevronDown size={14}/>
                  {/if}
                </div>
              </button>

              {#if expandedLinker === lnk.linker_id && lnk.analysis_times?.length}
                {@const rates = lnk.analysis_times.map((t: any) => t.kpis?.weighted_ftp_rate ?? 0)}
                {@const maxR = Math.max(...rates, 0.001)}
                {@const minR = Math.min(...rates)}
                <div class="lr-body">
                  <!-- Sparkline of FTP rate over time -->
                  {#if rates.length > 1}
                    <div class="spark-wrap">
                      {#each rates as r, i}
                        <div
                          class="spark-pt"
                          style="height:{Math.max(2, ((r - minR) / (maxR - minR + 0.0001)) * 48 + 4)}px"
                          title="{lnk.analysis_times[i]?.date}: {fmt(r)}"
                        ></div>
                      {/each}
                    </div>
                    <div class="spark-bounds">
                      <span>{fmt(minR)}</span><span>{fmt(maxR)}</span>
                    </div>
                  {/if}

                  <!-- Table -->
                  <div class="time-table-wrap">
                    <table class="time-table">
                      <thead>
                        <tr>
                          <th>Date</th>
                          <th>Taux FTP pondéré</th>
                          <th>Encours total</th>
                          <th>Intérêts FTP / mois</th>
                        </tr>
                      </thead>
                      <tbody>
                        {#each lnk.analysis_times as t}
                          <tr>
                            <td class="mono">{t.date}</td>
                            {#if t.error}
                              <td colspan="3" class="td-err">{t.error}</td>
                            {:else}
                              <td class="td-rate">{fmt(t.kpis?.weighted_ftp_rate ?? 0)}</td>
                              <td class="mono">{fmtM(t.kpis?.total_outstanding ?? 0)}</td>
                              <td class="mono">{fmtM(t.kpis?.total_ftp_int_monthly ?? 0)}</td>
                            {/if}
                          </tr>
                        {/each}
                      </tbody>
                    </table>
                  </div>
                </div>
              {/if}
            </div>
          {/each}
        </div>
      {:else if selected.status === 'completed'}
        <div class="empty-state">No result — the study may not contain any linkers.</div>
      {/if}
    {/if}
  {/if}
</div>

<style>
  /* Table */
  .exec-table-wrap { overflow-x: auto; }
  .exec-table {
    width: 100%; border-collapse: collapse; font-size: 13px;
  }
  .exec-table th {
    text-align: left; padding: 10px 12px; font-weight: 600;
    color: #6b7280; border-bottom: 2px solid #e5e7eb;
    background: #fafafa; font-size: 12px;
  }
  .exec-table td { padding: 10px 12px; border-bottom: 1px solid #f3f4f6; }
  .exec-row { cursor: pointer; transition: background 120ms; }
  .exec-row:hover td { background: #f9f9fb; }
  .mono { font-family: monospace; font-size: 12.5px; }

  .status-pill {
    display: inline-flex; align-items: center; gap: 4px;
    padding: 3px 10px; border-radius: 20px; font-size: 12px; font-weight: 600;
  }
  .status-completed { background: #d1fae5; color: #065f46; }
  .status-running   { background: #dbeafe; color: #1e40af; }
  .status-error     { background: #fee2e2; color: #991b1b; }
  .status-pending   { background: #f3f4f6; color: #6b7280; }

  /* Run form */
  .run-card   { padding: 24px; max-width: 560px; }
  .form-title { font-size: 16px; font-weight: 700; margin-bottom: 18px; }
  .form-grid  { display: grid; grid-template-columns: 1fr 1fr; gap: 14px; margin-bottom: 16px; }
  .form-footer{ display: flex; justify-content: flex-end; gap: 10px; padding-top: 16px; border-top: 1px solid #e5e7eb; }
  .hint-warn  { font-size: 11.5px; color: #d97706; display: block; margin-top: 3px; }

  /* Detail */
  .detail-header { padding: 16px 20px; margin-bottom: 16px; }
  .dh-row   { display: flex; align-items: center; gap: 10px; flex-wrap: wrap; }
  .dh-study { font-size: 13px; color: #6b7280; }
  .dh-dur   { font-size: 12px; color: #9ca3af; font-family: monospace; }

  /* KPI strip */
  .kpi-strip {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(160px, 1fr));
    gap: 12px;
    margin-bottom: 20px;
  }
  .kpi-card { background: #fff; border: 1px solid #e5e7eb; border-radius: 12px; padding: 16px 18px; }
  .kpi-val  { font-size: 22px; font-weight: 800; color: #6366f1; margin-bottom: 4px; }
  .kpi-lbl  { font-size: 11.5px; color: #9ca3af; font-weight: 500; }

  /* Linker results */
  .linkers-results { display: flex; flex-direction: column; gap: 10px; }
  .lr-card  { overflow: hidden; }
  .lr-header {
    display: flex; justify-content: space-between; align-items: center;
    padding: 14px 18px; background: none; border: none; width: 100%;
    cursor: pointer; text-align: left; font-size: 14px;
    transition: background 120ms;
  }
  .lr-header:hover { background: #f9f9fb; }
  .lr-header-right { display: flex; align-items: center; gap: 10px; }
  .lr-kpi   { font-size: 16px; font-weight: 700; color: #6366f1; }
  .lr-sub   { font-size: 12px; color: #9ca3af; }
  .lr-err   { font-size: 12px; color: #ef4444; }

  .lr-body  { padding: 0 18px 16px; border-top: 1px solid #f3f4f6; }

  /* Sparkline */
  .spark-wrap {
    display: flex; align-items: flex-end; gap: 2px;
    height: 56px; background: #f9fafb; border-radius: 6px;
    padding: 4px 6px; margin: 12px 0 2px;
  }
  .spark-pt { flex: 1; background: #6366f1; border-radius: 1px; min-width: 2px; }
  .spark-bounds {
    display: flex; justify-content: space-between;
    font-size: 11px; color: #9ca3af; margin-bottom: 10px;
  }

  /* Time table */
  .time-table-wrap { overflow-x: auto; margin-top: 8px; }
  .time-table { width: 100%; border-collapse: collapse; font-size: 12.5px; }
  .time-table th {
    text-align: left; padding: 6px 10px; font-weight: 600;
    color: #9ca3af; border-bottom: 1px solid #e5e7eb;
    font-size: 11px; text-transform: uppercase; letter-spacing: .04em;
  }
  .time-table td { padding: 6px 10px; border-bottom: 1px solid #f9fafb; }
  .td-rate { font-weight: 700; color: #6366f1; }
  .td-err  { color: #ef4444; font-style: italic; }
</style>
