<script lang="ts">
  import { tick } from 'svelte';
  import { executions, studies } from '$lib/api/client';
  import type {
    ExecutionSummary, ExecutionDetail, StudySummary,
    StudyUnitResult, AssignmentResult, TimeStep,
  } from '$lib/api/client';
  import * as echarts from 'echarts';
  import { Plus, Trash2, Play, X, CheckCircle, AlertCircle, Clock } from '@lucide/svelte';

  // ── État ──────────────────────────────────────────────────────────────────────

  let execList  = $state<ExecutionSummary[]>([]);
  let loading   = $state(true);
  let error     = $state<string | null>(null);

  let selectedId    = $state<string | null>(null);
  let detail        = $state<ExecutionDetail | null>(null);
  let detailLoading = $state(false);

  // Modal de lancement
  let showLaunchModal = $state(false);
  let allStudies      = $state<StudySummary[]>([]);
  let launchStudyId   = $state('');
  let launchLabel     = $state('');
  let launchError     = $state<string | null>(null);
  let launching       = $state(false);

  // Charts par assignment (key = assignment_id)
  let expandedAssignId = $state<string | null>(null);
  let chartEl = $state<HTMLDivElement | null>(null);
  let chartInstance: echarts.ECharts | null = null;

  // Onglet actif par study unit (key = study_unit_id → assignment index)
  let activeAssignIdx = $state<Record<string, number>>({});

  // ── Chargement ────────────────────────────────────────────────────────────────

  async function loadAll() {
    loading = true; error = null;
    try {
      const [el, sl] = await Promise.all([executions.list(), studies.list()]);
      execList   = el;
      allStudies = sl;
    } catch (e: any) { error = e.message; }
    finally { loading = false; }
  }

  async function selectExec(id: string) {
    if (selectedId === id) return;
    selectedId       = id;
    detail           = null;
    expandedAssignId = null;
    destroyChart();
    detailLoading = true;
    try { detail = await executions.get(id); }
    catch (e: any) { error = e.message; }
    finally { detailLoading = false; }
  }

  loadAll();

  // ── Lancement ─────────────────────────────────────────────────────────────────

  async function openLaunch() {
    launchStudyId = allStudies.find(s => s.status === 'ready')?.id ?? allStudies[0]?.id ?? '';
    launchLabel   = '';
    launchError   = null;
    showLaunchModal = true;
  }

  async function confirmLaunch() {
    if (!launchStudyId) { launchError = 'Sélectionner une étude'; return; }
    launching   = true;
    launchError = null;
    try {
      const result = await executions.create({
        study_id: launchStudyId,
        label:    launchLabel || undefined,
      });
      showLaunchModal = false;
      await loadAll();
      await selectExec(result.id);
    } catch (e: any) {
      launchError = e.message;
    } finally {
      launching = false;
    }
  }

  // ── Suppression ───────────────────────────────────────────────────────────────

  async function deleteExec(id: string) {
    if (!confirm('Supprimer cette exécution et son résultat ?')) return;
    try {
      await executions.delete(id);
      if (selectedId === id) { selectedId = null; detail = null; destroyChart(); }
      await loadAll();
    } catch (e: any) { alert(e.message); }
  }

  // ── ECharts ───────────────────────────────────────────────────────────────────

  function destroyChart() {
    if (chartInstance) { chartInstance.dispose(); chartInstance = null; }
  }

  $effect(() => {
    // Se déclenche quand expandedAssignId change ou que chartEl est monté
    if (!expandedAssignId || !chartEl) { destroyChart(); return; }

    const assign = findAssignment(expandedAssignId);
    if (!assign || assign.time_steps.length === 0) return;

    tick().then(() => {
      if (!chartEl) return;
      destroyChart();
      chartInstance = echarts.init(chartEl);

      const dates  = assign.time_steps.map(t => t.date);
      const ftpPct = assign.time_steps.map(t => +((t.kpis.weighted_ftp_rate * 100).toFixed(4)));
      const outM   = assign.time_steps.map(t => +(t.kpis.total_outstanding / 1e6).toFixed(2));

      chartInstance.setOption({
        backgroundColor: 'transparent',
        tooltip: {
          trigger: 'axis',
          formatter: (params: any[]) => {
            const d = params[0].axisValue;
            let s = `<b>${d}</b><br/>`;
            for (const p of params) {
              const unit = p.seriesName.includes('Encours') ? ' M' : ' %';
              s += `${p.marker}${p.seriesName}: <b>${p.value}${unit}</b><br/>`;
            }
            return s;
          },
        },
        legend: { top: 8, textStyle: { fontSize: 12 } },
        grid:   { top: 46, right: 70, bottom: 42, left: 60 },
        xAxis: {
          type: 'category', data: dates,
          axisLabel: { fontSize: 11, rotate: dates.length > 24 ? 35 : 0 },
        },
        yAxis: [
          {
            type: 'value', name: 'Taux FTP (%)', nameTextStyle: { fontSize: 11 },
            axisLabel: { fontSize: 11, formatter: (v: number) => v.toFixed(2) + '%' },
          },
          {
            type: 'value', name: 'Encours (M)', nameTextStyle: { fontSize: 11 },
            axisLabel: { fontSize: 11, formatter: (v: number) => v.toFixed(0) + 'M' },
          },
        ],
        series: [
          {
            name:  'Taux FTP pondéré',
            type:  'line',
            data:  ftpPct,
            yAxisIndex: 0,
            smooth: true,
            lineStyle: { color: '#6366f1', width: 2 },
            itemStyle: { color: '#6366f1' },
            symbol: 'none',
          },
          {
            name:  'Encours',
            type:  'bar',
            data:  outM,
            yAxisIndex: 1,
            barMaxWidth: 20,
            itemStyle: { color: 'rgba(99,102,241,.15)', borderColor: 'rgba(99,102,241,.3)', borderWidth: 1 },
          },
        ],
      });
    });
  });

  // ── Helpers ───────────────────────────────────────────────────────────────────

  function findAssignment(id: string): AssignmentResult | undefined {
    for (const su of detail?.result?.study_units ?? []) {
      const a = su.assignments.find(a => a.assignment_id === id);
      if (a) return a;
    }
    return undefined;
  }

  function toggleAssign(id: string) {
    if (expandedAssignId === id) {
      expandedAssignId = null;
      destroyChart();
    } else {
      expandedAssignId = id;
    }
  }

  function statusIcon(s: string) {
    return s === 'completed' ? '✓' : s === 'error' ? '✗' : s === 'running' ? '…' : '○';
  }
  function statusClass(s: string) {
    return s === 'completed' ? 'badge-active'
         : s === 'error'     ? 'badge-error'
         : s === 'running'   ? 'badge-pending'
         :                     'badge-draft';
  }
  function statusLabel(s: string) {
    return s === 'completed' ? 'Terminée' : s === 'error' ? 'Erreur'
         : s === 'running'   ? 'En cours' : 'En attente';
  }

  function fmtDuration(ms?: number) {
    if (!ms) return '—';
    if (ms < 1000) return `${ms} ms`;
    return `${(ms / 1000).toFixed(1)} s`;
  }

  function fmtPct(v: number) { return (v * 100).toFixed(4) + ' %'; }
  function fmtM(v: number)   { return (v / 1e6).toLocaleString('fr-FR', { maximumFractionDigits: 2 }) + ' M'; }
  function fmtAmt(v: number) { return v.toLocaleString('fr-FR', { maximumFractionDigits: 0 }); }

  // Statistiques résumées d'un assignment
  function assignStats(a: AssignmentResult) {
    if (!a.time_steps.length) return null;
    const rates = a.time_steps.map(t => t.kpis.weighted_ftp_rate);
    const outs  = a.time_steps.map(t => t.kpis.total_outstanding);
    const ints  = a.time_steps.map(t => t.kpis.ftp_interest_periodic);
    return {
      avgRate:   rates.reduce((a, b) => a + b, 0) / rates.length,
      lastOut:   outs[outs.length - 1],
      totalInt:  ints.reduce((a, b) => a + b, 0),
    };
  }
</script>

<!-- ── Layout ─────────────────────────────────────────────────────────────── -->
<div class="page">

  <!-- Panneau gauche -->
  <aside class="left-panel card">
    <div class="panel-header">
      <h2>Exécutions</h2>
      <button class="btn-primary" onclick={openLaunch}><Play size={12} /> Lancer</button>
    </div>

    {#if loading}
      <p class="loading" style="padding:16px">Chargement…</p>
    {:else if error}
      <div class="alert-error" style="margin:12px">{error}</div>
    {:else if execList.length === 0}
      <div class="empty-state" style="margin:16px;padding:32px 16px">
        <p>Aucune exécution</p>
        <p>Lancez votre première simulation.</p>
      </div>
    {:else}
      <div class="exec-list">
        {#each execList as ex}
          <!-- svelte-ignore a11y_click_events_have_key_events a11y_no_static_element_interactions -->
          <div
            class="exec-item"
            class:exec-item--active={selectedId === ex.id}
            onclick={() => selectExec(ex.id)}
          >
            <div class="exec-row1">
              <span class="exec-study">{ex.study_name ?? '—'}</span>
              <span class="badge {statusClass(ex.status)}">{statusLabel(ex.status)}</span>
            </div>
            {#if ex.label}
              <div class="exec-label">{ex.label}</div>
            {/if}
            <div class="exec-row2">
              <span class="exec-meta">{new Date(ex.created_at).toLocaleDateString('fr-FR')}</span>
              <span class="exec-meta">{fmtDuration(ex.duration_ms)}</span>
            </div>
            <!-- svelte-ignore a11y_click_events_have_key_events a11y_no_static_element_interactions -->
            <div class="exec-actions" onclick={e => e.stopPropagation()}>
              <button class="btn-sm btn-danger" onclick={() => deleteExec(ex.id)}><Trash2 size={11} /></button>
            </div>
          </div>
        {/each}
      </div>
    {/if}
  </aside>

  <!-- Panneau droit -->
  <main class="right-panel">
    {#if !selectedId}
      <div class="empty-state" style="margin:40px auto;max-width:400px">
        <Play size={32} style="margin:0 auto 12px;opacity:.3" />
        <p>Sélectionnez une exécution ou lancez-en une nouvelle</p>
      </div>

    {:else if detailLoading}
      <p class="loading" style="padding:32px">Chargement…</p>

    {:else if detail}
      <!-- En-tête -->
      <div class="detail-header">
        <div>
          <h2>{detail.study_name ?? 'Exécution'}</h2>
          {#if detail.label}
            <p class="detail-desc">{detail.label}</p>
          {/if}
          <div class="detail-meta-row">
            <span class="badge {statusClass(detail.status)}" style="font-size:12px">
              {statusIcon(detail.status)} {statusLabel(detail.status)}
            </span>
            <span class="detail-meta">Durée : {fmtDuration(detail.duration_ms)}</span>
            <span class="detail-meta">Méthode : {detail.method}</span>
            <span class="detail-meta">{new Date(detail.created_at).toLocaleString('fr-FR')}</span>
          </div>
        </div>
      </div>

      <!-- Erreur -->
      {#if detail.status === 'error' && detail.error_message}
        <div class="alert-error" style="margin:20px 28px">
          <strong>Erreur de calcul :</strong> {detail.error_message}
        </div>

      <!-- Résultats -->
      {:else if detail.status === 'completed' && detail.result}
        <div class="results-body">
          {#each detail.result.study_units as su}
            <div class="su-section">
              <div class="su-header">
                <div class="su-title">
                  <span class="su-name">{su.study_unit_name}</span>
                  <span class="su-range">
                    {su.time_step_range.start} → {su.time_step_range.end}
                    ({su.time_step_range.count} pas)
                  </span>
                </div>
                <span class="su-count">{su.assignments.length} assignment(s)</span>
              </div>

              {#each su.assignments as a}
                {@const stats = assignStats(a)}
                <div class="assign-card card">
                  <!-- En-tête de l'assignment -->
                  <!-- svelte-ignore a11y_click_events_have_key_events a11y_no_static_element_interactions -->
                  <div class="assign-head" onclick={() => toggleAssign(a.assignment_id)}>
                    <div class="assign-title-area">
                      <span class="assign-title">
                        {a.pair_label ?? `${a.vector_name} × ${a.schedule_name}`}
                      </span>
                      <span class="assign-subtitle">
                        {a.vector_name} × {a.schedule_name}
                        · {a.time_step_count} pas
                      </span>
                    </div>
                    {#if stats}
                      <div class="assign-kpis">
                        <div class="kpi-chip">
                          <span class="kpi-label">Taux moyen</span>
                          <span class="kpi-val">{fmtPct(stats.avgRate)}</span>
                        </div>
                        <div class="kpi-chip">
                          <span class="kpi-label">Dernier encours</span>
                          <span class="kpi-val">{fmtM(stats.lastOut)}</span>
                        </div>
                        <div class="kpi-chip">
                          <span class="kpi-label">Intérêts cumulés</span>
                          <span class="kpi-val">{fmtAmt(stats.totalInt)}</span>
                        </div>
                      </div>
                    {/if}
                    <span class="toggle-icon">{expandedAssignId === a.assignment_id ? '▲' : '▼'}</span>
                  </div>

                  <!-- Contenu expandable -->
                  {#if expandedAssignId === a.assignment_id}
                    <!-- Chart -->
                    <div class="chart-wrap">
                      <div
                        class="chart-el"
                        bind:this={chartEl}
                      ></div>
                    </div>

                    <!-- Table des pas de temps -->
                    <div class="ts-table-wrap">
                      <table class="ts-table">
                        <thead>
                          <tr>
                            <th>Date</th>
                            <th class="num">Encours</th>
                            <th class="num">Taux FTP</th>
                            <th class="num">Intérêts</th>
                            {#each a.bucket_labels as bl}
                              <th class="num tenor">{bl}</th>
                            {/each}
                          </tr>
                        </thead>
                        <tbody>
                          {#each a.time_steps as ts}
                            <tr>
                              <td class="cell-date">{ts.date}</td>
                              <td class="num">{fmtM(ts.kpis.total_outstanding)}</td>
                              <td class="num ftp-cell">{fmtPct(ts.kpis.weighted_ftp_rate)}</td>
                              <td class="num">{fmtAmt(ts.kpis.ftp_interest_periodic)}</td>
                              {#each a.bucket_labels as bl}
                                <td class="num tenor">
                                  {((ts.ftp_by_tenor[bl] ?? 0) * 100).toFixed(4)}%
                                </td>
                              {/each}
                            </tr>
                          {/each}
                        </tbody>
                      </table>
                    </div>
                  {/if}
                </div>
              {/each}
            </div>
          {/each}
        </div>

      {:else}
        <p class="loading" style="padding:32px">En attente des résultats…</p>
      {/if}
    {/if}
  </main>
</div>

<!-- ── Modal de lancement ────────────────────────────────────────────────────── -->
{#if showLaunchModal}
  <!-- svelte-ignore a11y_click_events_have_key_events a11y_no_static_element_interactions -->
  <div class="overlay" onclick={() => showLaunchModal = false}>
    <!-- svelte-ignore a11y_click_events_have_key_events a11y_no_static_element_interactions -->
    <div class="modal" onclick={e => e.stopPropagation()}>
      <div class="modal-hd">
        <h3>Lancer une exécution</h3>
        <button onclick={() => showLaunchModal = false}><X size={16} /></button>
      </div>
      <div class="modal-bd">
        {#if launchError}
          <div class="alert-error">{launchError}</div>
        {/if}

        <label>Étude
          <select bind:value={launchStudyId}>
            <option value="">— Sélectionner —</option>
            {#each allStudies as s}
              <option value={s.id}>
                {s.name}
                ({s.status === 'ready' ? '✓ Prête' : s.status})
                · {s.unit_count} unité(s)
              </option>
            {/each}
          </select>
        </label>

        <label>Label (optionnel)
          <input bind:value={launchLabel} placeholder="Ex. Run Q4 2024 baseline" />
        </label>

        {#if launching}
          <div class="launch-progress">
            <span class="spinner">⏳</span>
            Calcul en cours — maturity matching…
          </div>
        {/if}
      </div>
      <div class="modal-ft">
        <button class="btn-sm" onclick={() => showLaunchModal = false} disabled={launching}>Annuler</button>
        <button class="btn-primary" onclick={confirmLaunch} disabled={launching || !launchStudyId}>
          {#if launching}
            Calcul…
          {:else}
            <Play size={13} /> Lancer
          {/if}
        </button>
      </div>
    </div>
  </div>
{/if}

<style>
  /* ── Layout ── */
  .page { display: flex; height: 100vh; overflow: hidden; }

  .left-panel {
    width: 280px; flex-shrink: 0;
    display: flex; flex-direction: column;
    border-radius: 0; border-right: 1px solid #e5e7eb;
    overflow-y: auto;
  }
  .right-panel { flex: 1; overflow-y: auto; background: #f4f5f9; }

  /* ── Panel header ── */
  .panel-header {
    display: flex; align-items: center; justify-content: space-between;
    padding: 20px 16px 14px; border-bottom: 1px solid #f1f1f5; flex-shrink: 0;
  }
  .panel-header h2 { font-size: 15px; font-weight: 700; color: #1a1a2e; }

  /* ── Exec list ── */
  .exec-list { padding: 8px; }
  .exec-item {
    border: 1px solid transparent; border-radius: 8px;
    padding: 10px 12px; margin-bottom: 4px; cursor: pointer;
    transition: background 100ms, border-color 100ms;
  }
  .exec-item:hover { background: #f4f5f9; }
  .exec-item--active { background: #eef2ff; border-color: #c7d2fe; }

  .exec-row1 { display: flex; justify-content: space-between; align-items: center; margin-bottom: 2px; }
  .exec-study { font-size: 13px; font-weight: 600; color: #1a1a2e; }
  .exec-label { font-size: 11.5px; color: #6366f1; font-style: italic; margin-bottom: 2px; }
  .exec-row2  { display: flex; justify-content: space-between; }
  .exec-meta  { font-size: 11.5px; color: #9ca3af; }
  .exec-actions { display: flex; justify-content: flex-end; margin-top: 4px; }

  /* ── Detail header ── */
  .detail-header {
    padding: 22px 28px 14px;
    background: #fff;
    border-bottom: 1px solid #e5e7eb;
  }
  .detail-header h2 { font-size: 18px; font-weight: 700; color: #1a1a2e; margin-bottom: 3px; }
  .detail-desc { font-size: 13px; color: #6366f1; margin-bottom: 6px; }
  .detail-meta-row { display: flex; align-items: center; gap: 12px; flex-wrap: wrap; }
  .detail-meta { font-size: 12px; color: #6b7280; }

  /* ── Results body ── */
  .results-body { padding: 20px 28px 40px; }

  /* ── Study unit section ── */
  .su-section { margin-bottom: 28px; }
  .su-header {
    display: flex; justify-content: space-between; align-items: center;
    margin-bottom: 12px;
  }
  .su-name  { font-size: 14px; font-weight: 700; color: #1a1a2e; margin-right: 10px; }
  .su-range { font-size: 12px; color: #6b7280; }
  .su-count { font-size: 12px; color: #9ca3af; }
  .su-title { display: flex; align-items: baseline; gap: 0; flex-wrap: wrap; }

  /* ── Assignment card ── */
  .assign-card { border-radius: 10px; margin-bottom: 10px; overflow: hidden; }

  .assign-head {
    display: flex; align-items: center; gap: 12px;
    padding: 12px 16px; cursor: pointer;
    transition: background 100ms;
  }
  .assign-head:hover { background: #f9fafb; }
  .assign-title-area { flex: 1; min-width: 0; }
  .assign-title   { display: block; font-size: 13px; font-weight: 600; color: #1a1a2e; }
  .assign-subtitle{ display: block; font-size: 11.5px; color: #9ca3af; margin-top: 1px; }

  .assign-kpis { display: flex; gap: 10px; flex-shrink: 0; }
  .kpi-chip {
    background: #f4f5f9; border-radius: 8px; padding: 5px 10px;
    display: flex; flex-direction: column; align-items: center; min-width: 90px;
  }
  .kpi-label { font-size: 10px; font-weight: 600; color: #9ca3af; text-transform: uppercase; }
  .kpi-val   { font-size: 13px; font-weight: 700; color: #1a1a2e; }

  .toggle-icon { font-size: 11px; color: #9ca3af; flex-shrink: 0; }

  /* ── Chart ── */
  .chart-wrap {
    padding: 0 16px 4px;
    border-top: 1px solid #f1f1f5;
    background: #fafafa;
  }
  .chart-el { width: 100%; height: 240px; }

  /* ── Time step table ── */
  .ts-table-wrap {
    overflow-x: auto;
    border-top: 1px solid #f1f1f5;
    max-height: 340px;
    overflow-y: auto;
  }
  .ts-table { width: 100%; border-collapse: collapse; font-size: 12px; }
  .ts-table th {
    padding: 7px 12px; background: #f9fafb;
    text-align: left; font-size: 11px; font-weight: 600; color: #6b7280;
    text-transform: uppercase; position: sticky; top: 0;
    border-bottom: 1px solid #f1f1f5;
  }
  .ts-table th.num  { text-align: right; }
  .ts-table th.tenor { color: #6366f1; }
  .ts-table td { padding: 6px 12px; border-bottom: 1px solid #f9fafb; vertical-align: middle; }
  .ts-table tr:last-child td { border-bottom: none; }
  .cell-date { font-weight: 600; color: #374151; font-variant-numeric: tabular-nums; }
  .num      { text-align: right; font-variant-numeric: tabular-nums; color: #374151; }
  .ftp-cell { color: #6366f1; font-weight: 600; }
  .tenor    { color: #7c3aed; }

  /* ── Launch progress ── */
  .launch-progress {
    display: flex; align-items: center; gap: 8px;
    background: #eff6ff; border: 1px solid #bfdbfe;
    border-radius: 8px; padding: 10px 14px;
    font-size: 13px; color: #1e40af;
  }
  .spinner { font-size: 16px; animation: spin 2s linear infinite; display: inline-block; }
  @keyframes spin { from { transform: rotate(0deg); } to { transform: rotate(360deg); } }

  /* ── Modal ── */
  .overlay {
    position: fixed; inset: 0; background: rgba(0,0,0,.45);
    display: flex; align-items: center; justify-content: center; z-index: 100;
  }
  .modal {
    background: #fff; border-radius: 14px;
    width: 460px; max-width: 95vw; max-height: 85vh;
    display: flex; flex-direction: column;
    box-shadow: 0 20px 60px rgba(0,0,0,.2);
  }
  .modal-hd {
    display: flex; justify-content: space-between; align-items: center;
    padding: 18px 20px 14px; border-bottom: 1px solid #f1f1f5;
  }
  .modal-hd h3 { font-size: 15px; font-weight: 700; }
  .modal-hd button { background: none; border: none; cursor: pointer; color: #6b7280; }
  .modal-bd {
    padding: 16px 20px; overflow-y: auto;
    display: flex; flex-direction: column; gap: 12px;
  }
  .modal-ft {
    display: flex; justify-content: flex-end; gap: 8px;
    padding: 12px 20px; border-top: 1px solid #f1f1f5;
  }
</style>
