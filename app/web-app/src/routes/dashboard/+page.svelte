<script lang="ts">
  import { executions, studies } from '$lib/api/client';
  import type { ExecutionSummary, ExecutionDetail, StudySummary, AssignmentResult } from '$lib/api/client';
  import { Play, Trash2, X, ChevronDown, Copy, Check } from '@lucide/svelte';
  import { C_OBS, C_FLUX, ptColor, ptLabel } from '$lib/components/dashboard/utils';
  import type { KpiData } from '$lib/components/dashboard/utils';
  import KpiStrip      from '$lib/components/dashboard/KpiStrip.svelte';
  import TimelineChart from '$lib/components/dashboard/TimelineChart.svelte';
  import RunoffChart   from '$lib/components/dashboard/RunoffChart.svelte';
  import CourbesChart  from '$lib/components/dashboard/CourbesChart.svelte';
  import PnlChart      from '$lib/components/dashboard/PnlChart.svelte';
  import HeatmapChart  from '$lib/components/dashboard/HeatmapChart.svelte';
  import DataTable     from '$lib/components/dashboard/DataTable.svelte';

  // ── State ─────────────────────────────────────────────────────────────────────
  let execList      = $state<ExecutionSummary[]>([]);
  let loading       = $state(true);
  let error         = $state<string | null>(null);
  let selectedId    = $state<string | null>(null);
  let detail        = $state<ExecutionDetail | null>(null);
  let detailLoading = $state(false);

  // Launch modal
  let showLaunchModal = $state(false);
  let allStudies      = $state<StudySummary[]>([]);
  let launchStudyId   = $state('');
  let launchLabel     = $state('');
  let launchError     = $state<string | null>(null);
  let launching       = $state(false);

  // Dashboard state
  type Tab = 'timeline' | 'runoff' | 'courbes' | 'pnl' | 'heatmap' | 'data';
  let activeTab    = $state<Tab>('timeline');
  let selAssignIdx = $state(0);
  let runoffDates  = $state<string[]>([]);

  // ── Load ──────────────────────────────────────────────────────────────────────
  async function loadAll() {
    loading = true; error = null;
    try {
      const [el, sl] = await Promise.all([executions.list(), studies.list()]);
      execList = el; allStudies = sl;
    } catch (e: any) { error = e.message; }
    finally { loading = false; }
  }

  async function selectExec(id: string) {
    if (selectedId === id) return;
    selectedId = id; detail = null; detailLoading = true;
    runoffDates = []; selAssignIdx = 0;
    try { detail = await executions.get(id); }
    catch (e: any) { error = e.message; }
    finally { detailLoading = false; }
  }

  loadAll();

  // ── Derived ───────────────────────────────────────────────────────────────────
  const allAssignments = $derived<AssignmentResult[]>(
    detail?.result?.study_units?.flatMap(su => su.assignments) ?? []
  );

  const selAssign = $derived<AssignmentResult | null>(
    allAssignments[selAssignIdx] ?? null
  );

  const projBoundary = $derived.by<string | null>(() => {
    for (const a of allAssignments) {
      for (const ts of a.time_steps) {
        if (ts.period_type !== 'observed') return ts.date;
      }
    }
    return null;
  });

  const kpis = $derived.by<KpiData | null>(() => {
    if (!allAssignments.length) return null;
    let obsTotalOut = 0, projTotalOut = 0;
    let obsRateSum = 0, obsRateW = 0, projRateSum = 0, projRateW = 0;
    let obsInt = 0, projInt = 0;
    for (const a of allAssignments) {
      for (const ts of a.time_steps) {
        const o = ts.kpis.total_outstanding;
        const r = ts.kpis.weighted_ftp_rate;
        const i = ts.kpis.ftp_interest_periodic;
        if (ts.period_type === 'observed') {
          obsTotalOut = o; obsRateSum += r * o; obsRateW += o; obsInt += i;
        } else {
          projTotalOut = o; projRateSum += r * o; projRateW += o; projInt += i;
        }
      }
    }
    const lastA  = allAssignments[0];
    const lastTs = lastA?.time_steps[lastA.time_steps.length - 1];
    let wal = 0;
    if (lastTs?.profile?.length) {
      const buckets = lastA.bucket_labels.map(Number).filter(Boolean);
      let wSum = 0, wTot = 0;
      lastTs.profile.forEach((w: number, j: number) => { wSum += (buckets[j] ?? j+1) * w; wTot += w; });
      wal = wTot > 0 ? wSum / wTot : 0;
    }
    return {
      obsTotalOut, projTotalOut,
      obsAvgRate:  obsRateW  > 0 ? obsRateSum  / obsRateW  : 0,
      projAvgRate: projRateW > 0 ? projRateSum / projRateW : 0,
      obsInt, projInt, wal,
      method: allAssignments.map(a => a.method).filter((v,i,arr) => arr.indexOf(v) === i).join(' + '),
    };
  });

  // ── Launch / delete ───────────────────────────────────────────────────────────

  async function confirmLaunch() {
    if (!launchStudyId) { launchError = 'Sélectionner une étude'; return; }
    launching = true; launchError = null;
    try {
      const result = await executions.create({ study_id: launchStudyId, label: launchLabel || undefined });
      showLaunchModal = false; await loadAll(); await selectExec(result.id);
    } catch (e: any) { launchError = e.message; } finally { launching = false; }
  }
  async function deleteExec(id: string) {
    if (!confirm('Supprimer cette exécution ?')) return;
    try {
      await executions.delete(id);
      if (selectedId === id) { selectedId = null; detail = null; }
      await loadAll();
    } catch (e: any) { alert(e.message); }
  }

  // ── Debug JSON panel ─────────────────────────────────────────────────────────
  let showJsonDebug = $state(false);
  let jsonCopied    = $state(false);

  function copyJson() {
    if (!detail?.result) return;
    navigator.clipboard.writeText(JSON.stringify(detail.result, null, 2)).then(() => {
      jsonCopied = true;
      setTimeout(() => { jsonCopied = false; }, 2000);
    });
  }

  // ── Runoff date picker ────────────────────────────────────────────────────────
  function toggleRunoffDate(d: string) {
    if (runoffDates.includes(d)) runoffDates = runoffDates.filter(x => x !== d);
    else runoffDates = [...runoffDates, d].slice(-6);
  }

  // ── Formatters (left panel) ───────────────────────────────────────────────────
  function fmtDuration(ms?: number) {
    if (!ms) return '—'; return ms < 1000 ? `${ms} ms` : `${(ms/1000).toFixed(1)} s`;
  }
  function statusClass(s: string) {
    return s === 'completed' ? 'badge-active' : s === 'error' ? 'badge-error'
         : s === 'running'   ? 'badge-pending' : 'badge-draft';
  }
  function statusLabel(s: string) {
    return s === 'completed' ? 'Terminée' : s === 'error' ? 'Erreur'
         : s === 'running'   ? 'En cours' : 'En attente';
  }
</script>

<!-- ══════════════════════════════════════════════════════════ Layout -->
<div class="page">

  <!-- ── Top bar ────────────────────────────────────────────────────────────── -->
  <div class="top-bar">
    <h1 class="top-title">Dashboard</h1>
    <div class="top-controls">
      {#if loading}
        <span class="loading">Chargement…</span>
      {:else if error}
        <span class="top-error">{error}</span>
      {:else}
        <div class="exec-select-wrap">
          <ChevronDown size={14} class="select-icon" />
          <select
            class="exec-select"
            onchange={(e) => selectExec((e.target as HTMLSelectElement).value)}
          >
            <option value="">— Sélectionner une exécution —</option>
            {#each execList as ex}
              <option value={ex.id} selected={selectedId === ex.id}>
                {ex.study_name ?? '—'}{ex.label ? ` · ${ex.label}` : ''} — {new Date(ex.created_at).toLocaleDateString('fr-FR')} [{ex.method}] · {statusLabel(ex.status)}
              </option>
            {/each}
          </select>
        </div>
      {/if}
      {#if selectedId}
        <button class="btn-sm btn-danger" onclick={() => deleteExec(selectedId!)} title="Supprimer cette exécution">
          <Trash2 size={12} />
        </button>
      {/if}

    </div>
  </div>

  <!-- ── Main content ───────────────────────────────────────────────────────── -->
  <main class="main-content">

    {#if !selectedId}
      <div class="empty-state" style="margin:40px auto;max-width:400px">
        <Play size={32} style="margin:0 auto 12px;opacity:.3" />
        <p>Sélectionnez une exécution dans la liste déroulante ci-dessus</p>
      </div>

    {:else if detailLoading}
      <p class="loading" style="padding:32px">Chargement…</p>

    {:else if detail}

      <!-- ── Debug JSON panel ───────────────────────────────────────────────── -->
      <details class="json-debug" bind:open={showJsonDebug}>
        <summary class="json-debug-summary">
          🛠 JSON résultat brut
          <div class="json-debug-actions">
            <!-- svelte-ignore a11y_click_events_have_key_events a11y_no_static_element_interactions -->
            <span class="json-copy-btn" onclick={(e) => { e.preventDefault(); copyJson(); }}
                  title="Copier le JSON">
              {#if jsonCopied}
                <Check size={12} /> Copié
              {:else}
                <Copy size={12} /> Copier
              {/if}
            </span>
            <span class="json-debug-hint">{showJsonDebug ? 'Masquer' : 'Afficher'}</span>
          </div>
        </summary>
        <pre class="json-debug-body">{JSON.stringify(detail.result, null, 2)}</pre>
      </details>

      <!-- Execution header -->
      <div class="detail-header">
        <div class="detail-title-row">
          <div>
            <h2>{detail.study_name ?? 'Exécution'}</h2>
            {#if detail.label}<p class="detail-desc">{detail.label}</p>{/if}
          </div>
          <div class="detail-meta-row">
            <span class="badge {statusClass(detail.status)}">{statusLabel(detail.status)}</span>
            <span class="detail-meta">Méthode : <b>{detail.method}</b></span>
            <span class="detail-meta">Durée : {fmtDuration(detail.duration_ms)}</span>
            <span class="detail-meta">{new Date(detail.created_at).toLocaleString('fr-FR')}</span>
          </div>
        </div>
      </div>

      {#if detail.status === 'error' && detail.error_message}
        <div class="alert-error" style="margin:20px 28px">
          <strong>Erreur de calcul :</strong> {detail.error_message}
        </div>

      {:else if detail.status === 'completed' && detail.result && kpis}

        <KpiStrip {kpis} />

        <!-- Assignment selector (if multiple) -->
        {#if allAssignments.length > 1}
          <div class="assign-selector">
            <span class="assign-sel-label">Assignment :</span>
            {#each allAssignments as a, i}
              <button class="assign-chip" class:assign-chip--on={selAssignIdx === i}
                      onclick={() => { selAssignIdx = i; }}>
                <span class="method-dot" style="background:{a.method.includes('Flux') ? C_FLUX : C_OBS}"></span>
                {a.pair_label ?? a.vector_name}
                <span class="method-tag-sm">{a.method.includes('Flux') ? 'Flux' : 'Stock'}</span>
              </button>
            {/each}
          </div>
        {/if}

        <!-- Tab nav -->
        <nav class="tab-nav">
          {#each ([
            ['timeline', 'Timeline'],
            ['runoff',   'Runoff'],
            ['courbes',  'Courbes FTP'],
            ['pnl',      'P&L'],
            ['heatmap',  'Heatmap'],
            ['data',     'Données'],
          ] as [Tab, string][]) as [tid, tname]}
            <button class="tab-btn" class:tab-btn--on={activeTab === tid}
                    onclick={() => activeTab = tid}>{tname}</button>
          {/each}
        </nav>

        <!-- Tab body -->
        <div class="tab-body">

          {#if activeTab === 'timeline'}
            <TimelineChart assignments={allAssignments} {projBoundary} />

          {:else if activeTab === 'runoff' || activeTab === 'courbes'}
            {#if selAssign}
              <div class="runoff-section">
                <div class="runoff-picker-label">
                  Sélectionner jusqu'à 6 dates ({runoffDates.length}/6) :
                </div>
                <div class="date-chips">
                  {#each selAssign.time_steps as ts}
                    <button
                      class="date-chip"
                      class:date-chip--on={runoffDates.includes(ts.date)}
                      style="--c:{ptColor(ts.period_type)}"
                      onclick={() => toggleRunoffDate(ts.date)}
                    >
                      {ts.date}
                      <span class="date-chip-pt">{ptLabel(ts.period_type)}</span>
                    </button>
                  {/each}
                </div>
              </div>

              {#if runoffDates.length === 0}
                <div class="empty-state" style="padding:40px">
                  <p>Sélectionnez des dates ci-dessus pour afficher les profils.</p>
                </div>
              {:else if activeTab === 'runoff'}
                <RunoffChart assign={selAssign} {runoffDates} />
              {:else}
                <CourbesChart assign={selAssign} {runoffDates} />
              {/if}
            {/if}

          {:else if activeTab === 'pnl'}
            <PnlChart assignments={allAssignments} />

          {:else if activeTab === 'heatmap'}
            {#if selAssign}
              <HeatmapChart assign={selAssign} />
            {/if}

          {:else if activeTab === 'data'}
            {#if selAssign}
              <DataTable assign={selAssign} />
            {/if}
          {/if}

        </div><!-- tab-body -->

      {:else}
        <p class="loading" style="padding:32px">En attente…</p>
      {/if}
    {/if}
  </main>
</div>


<!-- Launch modal -->
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
        {#if launchError}<div class="alert-error">{launchError}</div>{/if}
        <label>Étude
          <select bind:value={launchStudyId}>
            <option value="">— Sélectionner —</option>
            {#each allStudies as s}
              <option value={s.id}>{s.name} ({s.status === 'ready' ? '✓' : s.status}) · {s.unit_count} unité(s)</option>
            {/each}
          </select>
        </label>
        <label>Label (optionnel)
          <input bind:value={launchLabel} placeholder="Ex. Run Q4 2024 baseline" />
        </label>
        {#if launching}
          <div class="launch-progress">⏳ Calcul en cours…</div>
        {/if}
      </div>
      <div class="modal-ft">
        <button class="btn-sm" onclick={() => showLaunchModal = false} disabled={launching}>Annuler</button>
        <button class="btn-primary" onclick={confirmLaunch} disabled={launching || !launchStudyId}>
          {#if launching}Calcul…{:else}<Play size={13} /> Lancer{/if}
        </button>
      </div>
    </div>
  </div>
{/if}

<style>
  /* ── Layout ── */
  .page {
    display: flex; flex-direction: column; height: 100vh; overflow: hidden;
    background: #f4f5f9;
  }
  .main-content { flex: 1; overflow-y: auto; display: flex; flex-direction: column; }

  /* ── Top bar ── */
  .top-bar {
    display: flex; align-items: center; gap: 16px;
    padding: 12px 24px; background: #fff;
    border-bottom: 1px solid #e5e7eb; flex-shrink: 0;
  }
  .top-title { font-size: 16px; font-weight: 700; color: #1a1a2e; white-space: nowrap; }
  .top-controls { display: flex; align-items: center; gap: 8px; flex: 1; }
  .top-error { font-size: 12px; color: #b91c1c; }

  .exec-select-wrap {
    position: relative; flex: 1; max-width: 520px;
  }
  .exec-select-wrap :global(.select-icon) {
    position: absolute; right: 10px; top: 50%; transform: translateY(-50%);
    pointer-events: none; color: #6b7280;
  }
  .exec-select {
    width: 100%; appearance: none; padding: 7px 32px 7px 12px;
    border: 1px solid #e5e7eb; border-radius: 8px; background: #f9fafb;
    font-size: 13px; color: #1a1a2e; cursor: pointer;
    transition: border-color 120ms, box-shadow 120ms;
  }
  .exec-select:focus {
    outline: none; border-color: #6366f1;
    box-shadow: 0 0 0 3px rgba(99,102,241,.12);
  }

  /* ── Detail header ── */
  .detail-header {
    padding: 16px 24px 12px; background: #fff;
    border-bottom: 1px solid #e5e7eb; flex-shrink: 0;
  }
  .detail-title-row {
    display: flex; justify-content: space-between; align-items: flex-start;
    gap: 16px; flex-wrap: wrap;
  }
  .detail-header h2 { font-size: 17px; font-weight: 700; color: #1a1a2e; margin-bottom: 2px; }
  .detail-desc { font-size: 12px; color: #6366f1; }
  .detail-meta-row { display: flex; align-items: center; gap: 10px; flex-wrap: wrap; }
  .detail-meta { font-size: 12px; color: #6b7280; }

  /* ── Assignment selector ── */
  .assign-selector {
    display: flex; align-items: center; gap: 8px; flex-wrap: wrap;
    padding: 8px 24px; background: #fafafa; border-bottom: 1px solid #f1f1f5;
    flex-shrink: 0;
  }
  .assign-sel-label { font-size: 11px; font-weight: 600; color: #9ca3af; }
  .assign-chip {
    display: flex; align-items: center; gap: 5px;
    border: 1px solid #e5e7eb; border-radius: 20px; padding: 3px 10px;
    font-size: 12px; background: #fff; cursor: pointer; transition: all 100ms;
  }
  .assign-chip--on { border-color: #6366f1; background: #eef2ff; color: #4338ca; }
  .method-dot { width: 8px; height: 8px; border-radius: 50%; flex-shrink: 0; }
  .method-tag-sm {
    font-size: 10px; font-weight: 600; background: #f3f4f6;
    padding: 1px 5px; border-radius: 4px; color: #374151;
  }

  /* ── Tab nav ── */
  .tab-nav {
    display: flex; gap: 2px; padding: 10px 24px 0;
    background: #fff; border-bottom: 1px solid #e5e7eb; flex-shrink: 0;
  }
  .tab-btn {
    padding: 7px 16px; font-size: 13px; font-weight: 500; color: #6b7280;
    background: none; border: none; border-bottom: 2px solid transparent;
    cursor: pointer; transition: all 100ms; border-radius: 4px 4px 0 0;
  }
  .tab-btn:hover { color: #1a1a2e; background: #f4f5f9; }
  .tab-btn--on { color: #6366f1; border-bottom-color: #6366f1; font-weight: 600; }

  /* ── Tab body ── */
  .tab-body { flex: 1; padding: 20px 24px; min-height: 0; overflow-y: auto; }

  /* ── Runoff date picker ── */
  .runoff-section { margin-bottom: 16px; }
  .runoff-picker-label { font-size: 12px; font-weight: 600; color: #6b7280; margin-bottom: 8px; }
  .date-chips { display: flex; gap: 6px; flex-wrap: wrap; }
  .date-chip {
    display: flex; align-items: center; gap: 4px;
    border: 1.5px solid var(--c, #6366f1); border-radius: 20px;
    padding: 3px 10px; font-size: 11.5px; font-weight: 500; color: var(--c, #6366f1);
    background: #fff; cursor: pointer; transition: all 100ms; opacity: 0.55;
  }
  .date-chip:hover { opacity: 0.8; }
  .date-chip--on { background: var(--c, #6366f1); color: #fff; opacity: 1; }
  .date-chip-pt { font-size: 9px; font-weight: 700; opacity: 0.8; }

  /* ── Debug JSON panel ── */
  .json-debug {
    border-bottom: 1px solid #e5e7eb; background: #fafafa; flex-shrink: 0;
  }
  .json-debug-summary {
    display: flex; align-items: center; justify-content: space-between;
    padding: 8px 24px; cursor: pointer; font-size: 12px; font-weight: 600;
    color: #6b7280; list-style: none; user-select: none;
  }
  .json-debug-summary::-webkit-details-marker { display: none; }
  .json-debug-summary:hover { background: #f1f1f5; color: #374151; }
  .json-debug-actions { display: flex; align-items: center; gap: 12px; }
  .json-debug-hint { font-size: 11px; font-weight: 400; color: #9ca3af; }
  .json-copy-btn {
    display: inline-flex; align-items: center; gap: 4px;
    font-size: 11px; font-weight: 500; color: #6366f1;
    background: #eef2ff; border-radius: 5px; padding: 2px 8px;
    cursor: pointer; transition: background 100ms;
  }
  .json-copy-btn:hover { background: #e0e7ff; }
  .json-debug-body {
    margin: 0; padding: 16px 24px;
    font-family: 'JetBrains Mono', 'Fira Code', monospace; font-size: 11px;
    line-height: 1.6; color: #1e293b; background: #f8fafc;
    border-top: 1px solid #e5e7eb;
    max-height: 400px; overflow-y: auto; overflow-x: auto;
    white-space: pre;
  }

  /* ── Launch progress ── */
  .launch-progress {
    background: #eff6ff; border: 1px solid #bfdbfe; border-radius: 8px;
    padding: 10px 14px; font-size: 13px; color: #1e40af;
  }

  /* ── Modal ── */
  .overlay {
    position: fixed; inset: 0; background: rgba(0,0,0,.45);
    display: flex; align-items: center; justify-content: center; z-index: 100;
  }
  .modal {
    background: #fff; border-radius: 14px; width: 460px; max-width: 95vw;
    display: flex; flex-direction: column; box-shadow: 0 20px 60px rgba(0,0,0,.2);
  }
  .modal-hd {
    display: flex; justify-content: space-between; align-items: center;
    padding: 18px 20px 14px; border-bottom: 1px solid #f1f1f5;
  }
  .modal-hd h3 { font-size: 15px; font-weight: 700; }
  .modal-hd button { background: none; border: none; cursor: pointer; color: #6b7280; }
  .modal-bd { padding: 16px 20px; display: flex; flex-direction: column; gap: 12px; }
  .modal-ft {
    display: flex; justify-content: flex-end; gap: 8px;
    padding: 12px 20px; border-top: 1px solid #f1f1f5;
  }
</style>
