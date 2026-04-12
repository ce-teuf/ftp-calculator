<script lang="ts">
  import { executions } from '$lib/api/client';
  import type { ExecutionSummary, ExecutionDetail, AssignmentResult, TimeStep } from '$lib/api/client';
  import * as echarts from 'echarts';
  import { LayoutDashboard, Download, RefreshCw } from '@lucide/svelte';

  // ── State ─────────────────────────────────────────────────────────────────────

  let execList       = $state<ExecutionSummary[]>([]);
  let loading        = $state(true);
  let error          = $state<string | null>(null);

  let primaryId      = $state('');
  let compareId      = $state('');
  let primaryDetail  = $state<ExecutionDetail | null>(null);
  let compareDetail  = $state<ExecutionDetail | null>(null);
  let loadingPrimary = $state(false);
  let loadingCompare = $state(false);

  // Filters
  let dateFrom      = $state('');
  let dateTo        = $state('');
  let hiddenAssigns = $state<Record<string, boolean>>({});
  let tenorDateIdx  = $state(0);
  let heatAssignIdx = $state(0);

  // Chart DOM refs
  let chartFtpEl   = $state<HTMLDivElement | null>(null);
  let chartOutEl   = $state<HTMLDivElement | null>(null);
  let chartTenorEl = $state<HTMLDivElement | null>(null);
  let chartHeatEl  = $state<HTMLDivElement | null>(null);

  // Chart instances (non-reactive, managed via effects)
  let chartFtp:   echarts.ECharts | null = null;
  let chartOut:   echarts.ECharts | null = null;
  let chartTenor: echarts.ECharts | null = null;
  let chartHeat:  echarts.ECharts | null = null;

  const PALETTE = ['#6366f1','#10b981','#f59e0b','#ef4444','#8b5cf6','#06b6d4','#f97316','#14b8a6'];

  // ── Init ──────────────────────────────────────────────────────────────────────

  async function load() {
    loading = true; error = null;
    try { execList = (await executions.list()).filter(e => e.status === 'completed'); }
    catch (e: any) { error = e.message; }
    finally { loading = false; }
  }
  load();

  // ── Load execution details ────────────────────────────────────────────────────

  $effect(() => {
    const id = primaryId;
    if (!id) { primaryDetail = null; return; }
    loadingPrimary = true;
    hiddenAssigns = {};
    tenorDateIdx = 0;
    heatAssignIdx = 0;
    executions.get(id)
      .then(d  => { primaryDetail = d; })
      .catch(e  => { error = e.message; })
      .finally(() => { loadingPrimary = false; });
  });

  $effect(() => {
    const id = compareId;
    if (!id) { compareDetail = null; return; }
    loadingCompare = true;
    executions.get(id)
      .then(d  => { compareDetail = d; })
      .catch(e  => { error = e.message; })
      .finally(() => { loadingCompare = false; });
  });

  // ── Helpers ───────────────────────────────────────────────────────────────────

  function allAssignments(detail: ExecutionDetail | null): AssignmentResult[] {
    if (!detail?.result) return [];
    return detail.result.study_units.flatMap(su => su.assignments);
  }

  function filteredSteps(a: AssignmentResult): TimeStep[] {
    return a.time_steps.filter(ts =>
      (!dateFrom || ts.date >= dateFrom) && (!dateTo || ts.date <= dateTo)
    );
  }

  function assignLabel(a: AssignmentResult): string {
    return a.pair_label ?? `${a.vector_name} × ${a.schedule_name}`;
  }

  function tenorToMonths(label: string): number {
    const n = parseFloat(label);
    if (isNaN(n)) return 0;
    if (label.endsWith('Y')) return n * 12;
    if (label.endsWith('M')) return n;
    if (label.endsWith('W')) return n / 4.33;
    if (label.endsWith('D')) return n / 30;
    return n;
  }

  function sortedTenors(keys: string[]): string[] {
    return [...keys].sort((a, b) => tenorToMonths(a) - tenorToMonths(b));
  }

  function fmtPct(v: number)  { return (v * 100).toFixed(4) + '%'; }
  function fmtM(v: number)    { return (v / 1e6).toFixed(2) + ' M'; }
  function fmtAmt(v: number)  { return v.toLocaleString('fr-FR', { maximumFractionDigits: 0 }); }
  function fmtStd(v: number)  { return (v * 100).toFixed(4) + ' pp'; }

  // ── Derived ───────────────────────────────────────────────────────────────────

  let primaryAssigns = $derived(
    allAssignments(primaryDetail).filter(a => !hiddenAssigns[a.assignment_id])
  );
  let compareAssigns = $derived(
    allAssignments(compareDetail).filter(a => !hiddenAssigns[a.assignment_id])
  );

  let uniqueDates = $derived.by(() => {
    const set = new Set<string>();
    for (const a of primaryAssigns) {
      for (const ts of filteredSteps(a)) set.add(ts.date);
    }
    return [...set].sort();
  });

  let globalKpis = $derived.by(() => {
    if (!primaryAssigns.length) return null;
    let wRateSum = 0, outSum = 0, totalInterest = 0, lastOut = 0;
    const rates: number[] = [];
    for (const a of primaryAssigns) {
      const steps = filteredSteps(a);
      if (!steps.length) continue;
      for (const ts of steps) {
        wRateSum      += ts.kpis.weighted_ftp_rate * ts.kpis.total_outstanding;
        outSum        += ts.kpis.total_outstanding;
        totalInterest += ts.kpis.ftp_interest_periodic;
        rates.push(ts.kpis.weighted_ftp_rate);
      }
      lastOut += steps[steps.length - 1].kpis.total_outstanding;
    }
    const avgRate = outSum > 0 ? wRateSum / outSum : 0;
    const variance = rates.length
      ? rates.reduce((s, r) => s + (r - avgRate) ** 2, 0) / rates.length
      : 0;
    return { avgRate, stdRate: Math.sqrt(variance), totalInterest, lastOut };
  });

  // ── Chart: FTP rate evolution ─────────────────────────────────────────────────

  $effect(() => {
    const assigns    = primaryAssigns;
    const cmpAssigns = compareAssigns;
    const el = chartFtpEl;
    if (!el) { if (chartFtp) { chartFtp.dispose(); chartFtp = null; } return; }

    if (chartFtp) { chartFtp.dispose(); chartFtp = null; }
    chartFtp = echarts.init(el);

    const mkSeries = (list: AssignmentResult[], dashed: boolean) =>
      list.map((a, i) => {
        const steps = filteredSteps(a);
        return {
          name: dashed ? `[Comp] ${assignLabel(a)}` : assignLabel(a),
          type: 'line',
          data: steps.map(ts => [ts.date, +((ts.kpis.weighted_ftp_rate * 100).toFixed(4))]),
          smooth: true,
          lineStyle: { color: PALETTE[i % PALETTE.length], width: dashed ? 1.5 : 2, type: dashed ? 'dashed' : 'solid' },
          itemStyle: { color: PALETTE[i % PALETTE.length] },
          symbol: 'none',
        };
      });

    chartFtp.setOption({
      backgroundColor: 'transparent',
      tooltip: {
        trigger: 'axis',
        formatter: (params: any[]) => {
          const d = params[0]?.axisValue ?? '';
          return `<b>${d}</b><br/>` + params.map(p =>
            `${p.marker}${p.seriesName}: <b>${(+p.value[1]).toFixed(4)}%</b>`
          ).join('<br/>');
        },
      },
      legend: { top: 4, type: 'scroll', textStyle: { fontSize: 11 } },
      grid:   { top: 44, right: 20, bottom: 40, left: 64 },
      xAxis:  { type: 'category', data: uniqueDates, axisLabel: { fontSize: 10, rotate: uniqueDates.length > 24 ? 35 : 0 } },
      yAxis:  { type: 'value', name: 'Taux FTP (%)', axisLabel: { fontSize: 10, formatter: (v: number) => v.toFixed(2) + '%' } },
      series: [...mkSeries(assigns, false), ...mkSeries(cmpAssigns, true)],
    });

    return () => { if (chartFtp) { chartFtp.dispose(); chartFtp = null; } };
  });

  // ── Chart: Outstanding evolution ─────────────────────────────────────────────

  $effect(() => {
    const assigns = primaryAssigns;
    const el = chartOutEl;
    if (!el) { if (chartOut) { chartOut.dispose(); chartOut = null; } return; }

    if (chartOut) { chartOut.dispose(); chartOut = null; }
    chartOut = echarts.init(el);

    chartOut.setOption({
      backgroundColor: 'transparent',
      tooltip: {
        trigger: 'axis',
        formatter: (params: any[]) => {
          const d = params[0]?.axisValue ?? '';
          return `<b>${d}</b><br/>` + params.map(p =>
            `${p.marker}${p.seriesName}: <b>${(+p.value[1]).toFixed(2)} M</b>`
          ).join('<br/>');
        },
      },
      legend: { top: 4, type: 'scroll', textStyle: { fontSize: 11 } },
      grid:   { top: 44, right: 20, bottom: 40, left: 68 },
      xAxis:  { type: 'category', data: uniqueDates, axisLabel: { fontSize: 10, rotate: uniqueDates.length > 24 ? 35 : 0 } },
      yAxis:  { type: 'value', name: 'Encours (M)', axisLabel: { fontSize: 10, formatter: (v: number) => v.toFixed(0) + 'M' } },
      series: assigns.map((a, i) => {
        const steps = filteredSteps(a);
        return {
          name: assignLabel(a),
          type: 'line',
          stack: 'total',
          areaStyle: { color: PALETTE[i % PALETTE.length], opacity: 0.18 },
          data: steps.map(ts => [ts.date, +(ts.kpis.total_outstanding / 1e6).toFixed(2)]),
          smooth: true,
          lineStyle: { color: PALETTE[i % PALETTE.length], width: 1.5 },
          itemStyle: { color: PALETTE[i % PALETTE.length] },
          symbol: 'none',
        };
      }),
    });

    return () => { if (chartOut) { chartOut.dispose(); chartOut = null; } };
  });

  // ── Chart: FTP by tenor at selected date ─────────────────────────────────────

  $effect(() => {
    const assigns = primaryAssigns;
    const dates   = uniqueDates;
    const idx     = tenorDateIdx;
    const el      = chartTenorEl;
    if (!el || !dates.length) { if (chartTenor) { chartTenor.dispose(); chartTenor = null; } return; }

    if (chartTenor) { chartTenor.dispose(); chartTenor = null; }
    chartTenor = echarts.init(el);

    const date = dates[Math.min(idx, dates.length - 1)];
    const tenorSet = new Set<string>();
    for (const a of assigns) {
      const ts = a.time_steps.find(t => t.date === date);
      if (ts) Object.keys(ts.ftp_by_tenor).forEach(k => tenorSet.add(k));
    }
    const tenors = sortedTenors([...tenorSet]);

    chartTenor.setOption({
      backgroundColor: 'transparent',
      tooltip: { trigger: 'axis', valueFormatter: (v: number) => (+v).toFixed(4) + '%' },
      legend: { top: 4, type: 'scroll', textStyle: { fontSize: 11 } },
      grid:   { top: 44, right: 20, bottom: 40, left: 64 },
      xAxis:  { type: 'category', data: tenors, axisLabel: { fontSize: 11 } },
      yAxis:  { type: 'value', name: 'Taux FTP (%)', axisLabel: { fontSize: 10, formatter: (v: number) => v.toFixed(2) + '%' } },
      series: assigns.map((a, i) => {
        const ts = a.time_steps.find(t => t.date === date);
        return {
          name: assignLabel(a),
          type: 'line',
          data: tenors.map(t => ts ? +((ts.ftp_by_tenor[t] ?? 0) * 100).toFixed(4) : 0),
          smooth: false,
          lineStyle: { color: PALETTE[i % PALETTE.length], width: 2 },
          itemStyle: { color: PALETTE[i % PALETTE.length] },
          symbolSize: 5,
        };
      }),
    });

    return () => { if (chartTenor) { chartTenor.dispose(); chartTenor = null; } };
  });

  // ── Chart: Heatmap time × tenor ───────────────────────────────────────────────

  $effect(() => {
    const assigns = primaryAssigns;
    const idx     = heatAssignIdx;
    const el      = chartHeatEl;
    if (!el || !assigns.length) { if (chartHeat) { chartHeat.dispose(); chartHeat = null; } return; }

    const a = assigns[Math.min(idx, assigns.length - 1)];
    if (!a) return;

    const steps = filteredSteps(a);
    if (!steps.length) return;

    if (chartHeat) { chartHeat.dispose(); chartHeat = null; }
    chartHeat = echarts.init(el);

    const tenors = sortedTenors(Object.keys(steps[0]?.ftp_by_tenor ?? {}));
    const dates  = steps.map(ts => ts.date);
    const data: [number, number, number][] = [];
    let minV = Infinity, maxV = -Infinity;

    for (let ti = 0; ti < dates.length; ti++) {
      const ts = steps[ti];
      for (let yi = 0; yi < tenors.length; yi++) {
        const v = (ts.ftp_by_tenor[tenors[yi]] ?? 0) * 100;
        data.push([ti, yi, +v.toFixed(4)]);
        if (v < minV) minV = v;
        if (v > maxV) maxV = v;
      }
    }

    chartHeat.setOption({
      backgroundColor: 'transparent',
      tooltip: {
        formatter: (p: any) =>
          `<b>${dates[p.value[0]]}</b> · ${tenors[p.value[1]]}<br/>Taux FTP : <b>${p.value[2].toFixed(4)}%</b>`,
      },
      grid: { top: 12, right: 90, bottom: 60, left: 52 },
      xAxis: {
        type: 'category', data: dates, splitArea: { show: true },
        axisLabel: { fontSize: 9, rotate: dates.length > 16 ? 45 : 0 },
      },
      yAxis: { type: 'category', data: tenors, splitArea: { show: true }, axisLabel: { fontSize: 11 } },
      visualMap: {
        min: minV, max: maxV, calculable: true,
        orient: 'vertical', right: 0, top: 'center',
        inRange: { color: ['#e0e7ff', '#6366f1', '#312e81'] },
        textStyle: { fontSize: 9 },
        formatter: (v: number) => v.toFixed(2) + '%',
      },
      series: [{
        type: 'heatmap', data,
        emphasis: { itemStyle: { shadowBlur: 8, shadowColor: 'rgba(0,0,0,.4)' } },
      }],
    });

    return () => { if (chartHeat) { chartHeat.dispose(); chartHeat = null; } };
  });

  // ── Export ────────────────────────────────────────────────────────────────────

  function exportPng(chart: echarts.ECharts | null, filename: string) {
    if (!chart) return;
    const url = chart.getDataURL({ type: 'png', pixelRatio: 2, backgroundColor: '#fff' });
    const a = document.createElement('a');
    a.href = url; a.download = filename + '.png'; a.click();
  }

  function exportCsv() {
    if (!primaryAssigns.length) return;

    const firstSteps = primaryAssigns.flatMap(a => filteredSteps(a));
    const tenors = firstSteps[0] ? sortedTenors(Object.keys(firstSteps[0].ftp_by_tenor)) : [];
    const header = ['assignment','date','total_outstanding','weighted_ftp_rate_%','ftp_interest_periodic',...tenors.map(t => `ftp_${t}_%`)];

    const rows: string[][] = [header];
    for (const a of primaryAssigns) {
      for (const ts of filteredSteps(a)) {
        rows.push([
          assignLabel(a), ts.date,
          String(ts.kpis.total_outstanding),
          String((ts.kpis.weighted_ftp_rate * 100).toFixed(6)),
          String(ts.kpis.ftp_interest_periodic),
          ...tenors.map(t => String(((ts.ftp_by_tenor[t] ?? 0) * 100).toFixed(6))),
        ]);
      }
    }

    const csv  = rows.map(r => r.map(v => `"${v.replace(/"/g, '""')}"`).join(',')).join('\n');
    const blob = new Blob([csv], { type: 'text/csv;charset=utf-8;' });
    const url  = URL.createObjectURL(blob);
    const a    = document.createElement('a');
    a.href = url; a.download = 'ftp-results.csv'; a.click();
    URL.revokeObjectURL(url);
  }

  function toggleAssign(id: string) {
    hiddenAssigns = { ...hiddenAssigns, [id]: !hiddenAssigns[id] };
  }

  const hasResults = $derived(
    !!primaryDetail && primaryDetail.status === 'completed' && !!primaryDetail.result
  );
</script>

<!-- ── Page ──────────────────────────────────────────────────────────────────── -->
<div class="dash-page">

  <!-- ── Header ── -->
  <header class="dash-header card">
    <div class="header-left">
      <span class="header-icon"><LayoutDashboard size={18} /></span>
      <span class="header-title">Dashboard</span>
    </div>

    <div class="header-controls">
      <!-- Execution selectors -->
      <div class="ctrl-group">
        <label class="ctrl-label">Exécution
          <select class="ctrl-select" bind:value={primaryId} disabled={loading}>
            <option value="">— Sélectionner —</option>
            {#each execList as ex}
              <option value={ex.id}>{ex.study_name ?? ex.id}{ex.label ? ` · ${ex.label}` : ''}</option>
            {/each}
          </select>
        </label>

        <label class="ctrl-label">Comparaison
          <select class="ctrl-select" bind:value={compareId} disabled={loading || !primaryId}>
            <option value="">— Aucune —</option>
            {#each execList.filter(e => e.id !== primaryId) as ex}
              <option value={ex.id}>{ex.study_name ?? ex.id}{ex.label ? ` · ${ex.label}` : ''}</option>
            {/each}
          </select>
        </label>
      </div>

      <!-- Date filters -->
      <div class="ctrl-group">
        <label class="ctrl-label">Du
          <input class="ctrl-month" type="month" bind:value={dateFrom} />
        </label>
        <label class="ctrl-label">Au
          <input class="ctrl-month" type="month" bind:value={dateTo} />
        </label>
        {#if dateFrom || dateTo}
          <button class="btn-sm" onclick={() => { dateFrom = ''; dateTo = ''; }}>
            <RefreshCw size={11} /> Reset
          </button>
        {/if}
      </div>

      <!-- CSV export -->
      {#if hasResults}
        <button class="btn-sm btn-export" onclick={exportCsv}>
          <Download size={12} /> CSV
        </button>
      {/if}
    </div>
  </header>

  <!-- ── Empty / Loading ── -->
  {#if loading}
    <p class="loading" style="padding:40px 32px">Chargement des exécutions…</p>
  {:else if error}
    <div class="alert-error" style="margin:24px 32px">{error}</div>
  {:else if !primaryId}
    <div class="empty-state" style="margin:60px auto;max-width:380px">
      <LayoutDashboard size={36} style="margin:0 auto 14px;opacity:.25" />
      <p>Sélectionnez une exécution terminée pour visualiser les résultats FTP.</p>
    </div>
  {:else if loadingPrimary}
    <p class="loading" style="padding:40px 32px">Chargement des résultats…</p>
  {:else if primaryDetail?.status !== 'completed'}
    <div class="alert-error" style="margin:24px 32px">
      L'exécution sélectionnée n'est pas terminée (statut : {primaryDetail?.status ?? '?'}).
    </div>
  {:else if hasResults}

    <!-- ── Assignment filter chips ── -->
    {@const allAssigns = allAssignments(primaryDetail)}
    {#if allAssigns.length > 1}
      <div class="assign-chips">
        <span class="chips-label">Assignments :</span>
        {#each allAssigns as a, i}
          <button
            class="chip"
            class:chip--off={hiddenAssigns[a.assignment_id]}
            style="--chip-color:{PALETTE[i % PALETTE.length]}"
            onclick={() => toggleAssign(a.assignment_id)}
          >
            {assignLabel(a)}
          </button>
        {/each}
      </div>
    {/if}

    <!-- ── KPI row ── -->
    {#if globalKpis}
      <div class="kpi-row">
        <div class="kpi-card">
          <span class="kpi-label">Taux FTP moyen pondéré</span>
          <span class="kpi-value">{fmtPct(globalKpis.avgRate)}</span>
        </div>
        <div class="kpi-card">
          <span class="kpi-label">Écart-type du taux</span>
          <span class="kpi-value">{fmtStd(globalKpis.stdRate)}</span>
        </div>
        <div class="kpi-card">
          <span class="kpi-label">Dernier encours total</span>
          <span class="kpi-value">{fmtM(globalKpis.lastOut)}</span>
        </div>
        <div class="kpi-card">
          <span class="kpi-label">Intérêts FTP cumulés</span>
          <span class="kpi-value">{fmtAmt(globalKpis.totalInterest)}</span>
        </div>
        {#if compareDetail?.result}
          {@const cmpAssigns = allAssignments(compareDetail)}
          {@const cmpOut = cmpAssigns.flatMap(a => filteredSteps(a)).reduce((s, ts) => s + ts.kpis.total_outstanding, 0)}
          {@const cmpWR  = cmpAssigns.flatMap(a => filteredSteps(a)).reduce((s, ts) => s + ts.kpis.weighted_ftp_rate * ts.kpis.total_outstanding, 0)}
          <div class="kpi-card kpi-card--compare">
            <span class="kpi-label">Taux moyen [Comp]</span>
            <span class="kpi-value">{cmpOut > 0 ? fmtPct(cmpWR / cmpOut) : '—'}</span>
          </div>
        {/if}
      </div>
    {/if}

    <!-- ── Charts grid ── -->
    <div class="charts-grid">

      <!-- Chart 1 — FTP rate evolution -->
      <div class="chart-card card">
        <div class="chart-card-head">
          <span class="chart-title">Évolution du Taux FTP</span>
          <button class="btn-sm" onclick={() => exportPng(chartFtp, 'ftp-rate')}>
            <Download size={11} /> PNG
          </button>
        </div>
        <div bind:this={chartFtpEl} class="chart-el"></div>
      </div>

      <!-- Chart 2 — Outstanding evolution -->
      <div class="chart-card card">
        <div class="chart-card-head">
          <span class="chart-title">Évolution des Encours</span>
          <button class="btn-sm" onclick={() => exportPng(chartOut, 'encours')}>
            <Download size={11} /> PNG
          </button>
        </div>
        <div bind:this={chartOutEl} class="chart-el"></div>
      </div>

      <!-- Chart 3 — FTP by tenor at date -->
      <div class="chart-card card">
        <div class="chart-card-head">
          <span class="chart-title">Courbe FTP par Tenor</span>
          <div class="chart-head-controls">
            {#if uniqueDates.length}
              <select
                class="date-select"
                value={uniqueDates[Math.min(tenorDateIdx, uniqueDates.length - 1)]}
                onchange={e => { tenorDateIdx = uniqueDates.indexOf((e.target as HTMLSelectElement).value); }}
              >
                {#each uniqueDates as d, i}
                  <option value={d}>{d}</option>
                {/each}
              </select>
            {/if}
            <button class="btn-sm" onclick={() => exportPng(chartTenor, 'ftp-tenor')}>
              <Download size={11} /> PNG
            </button>
          </div>
        </div>
        <div bind:this={chartTenorEl} class="chart-el"></div>
      </div>

      <!-- Chart 4 — Heatmap -->
      <div class="chart-card card">
        <div class="chart-card-head">
          <span class="chart-title">Heatmap FTP (Temps × Tenor)</span>
          <div class="chart-head-controls">
            {#if primaryAssigns.length > 1}
              <select
                class="date-select"
                value={heatAssignIdx}
                onchange={e => { heatAssignIdx = +((e.target as HTMLSelectElement).value); }}
              >
                {#each primaryAssigns as a, i}
                  <option value={i}>{assignLabel(a)}</option>
                {/each}
              </select>
            {/if}
            <button class="btn-sm" onclick={() => exportPng(chartHeat, 'heatmap-ftp')}>
              <Download size={11} /> PNG
            </button>
          </div>
        </div>
        <div bind:this={chartHeatEl} class="chart-el chart-el--heat"></div>
      </div>

    </div><!-- /charts-grid -->

    <!-- ── Multi-execution comparison note ── -->
    {#if compareDetail?.result}
      <div class="compare-banner">
        Comparaison activée — tirets = <strong>{compareDetail.study_name ?? compareDetail.id}</strong>
        {#if compareDetail.label}<em> ({compareDetail.label})</em>{/if}
      </div>
    {/if}

  {/if}
</div>

<style>
  /* ── Page ── */
  .dash-page {
    min-height: 100vh;
    background: #f4f5f9;
    padding: 0 0 40px;
  }

  /* ── Header ── */
  .dash-header {
    display: flex; align-items: center; gap: 24px;
    padding: 14px 28px; border-radius: 0;
    border-bottom: 1px solid #e5e7eb;
    flex-wrap: wrap; row-gap: 10px;
  }
  .header-left  { display: flex; align-items: center; gap: 8px; flex-shrink: 0; }
  .header-icon  { color: #6366f1; display: flex; }
  .header-title { font-size: 16px; font-weight: 700; color: #1a1a2e; }

  .header-controls {
    display: flex; align-items: flex-end; gap: 16px; flex: 1; flex-wrap: wrap;
  }
  .ctrl-group { display: flex; align-items: flex-end; gap: 10px; flex-wrap: wrap; }
  .ctrl-label {
    display: flex; flex-direction: column; gap: 3px;
    font-size: 11.5px; font-weight: 500; color: #6b7280;
  }
  .ctrl-select {
    border: 1px solid #e5e7eb; border-radius: 7px; padding: 5px 10px;
    font-size: 12.5px; background: #fff; color: #1a1a2e;
    min-width: 180px; max-width: 240px; width: auto;
    cursor: pointer;
  }
  .ctrl-month {
    border: 1px solid #e5e7eb; border-radius: 7px; padding: 5px 9px;
    font-size: 12.5px; background: #fff; color: #1a1a2e;
    width: 140px;
  }
  .btn-export {
    background: #f0fdf4; color: #15803d; align-self: flex-end;
  }
  .btn-export:hover { background: #dcfce7; }

  /* ── Assignment chips ── */
  .assign-chips {
    display: flex; align-items: center; gap: 6px;
    padding: 10px 28px; flex-wrap: wrap;
    border-bottom: 1px solid #eee;
  }
  .chips-label { font-size: 11.5px; font-weight: 600; color: #6b7280; flex-shrink: 0; }
  .chip {
    border: 1.5px solid var(--chip-color, #6366f1);
    background: transparent;
    color: var(--chip-color, #6366f1);
    border-radius: 20px; padding: 3px 10px;
    font-size: 11.5px; font-weight: 600; cursor: pointer;
    transition: background 120ms, opacity 120ms;
  }
  .chip:hover { background: color-mix(in srgb, var(--chip-color, #6366f1) 10%, transparent); }
  .chip--off  { opacity: 0.3; }

  /* ── KPI row ── */
  .kpi-row {
    display: flex; gap: 14px; padding: 16px 28px;
    flex-wrap: wrap;
  }
  .kpi-card {
    background: #fff; border-radius: 10px; padding: 12px 18px;
    box-shadow: 0 1px 3px rgba(0,0,0,.06);
    display: flex; flex-direction: column; gap: 3px;
    min-width: 160px;
  }
  .kpi-card--compare { border: 1.5px dashed #c7d2fe; }
  .kpi-label { font-size: 10.5px; font-weight: 600; color: #9ca3af; text-transform: uppercase; }
  .kpi-value { font-size: 18px; font-weight: 700; color: #1a1a2e; }

  /* ── Charts grid ── */
  .charts-grid {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 18px;
    padding: 0 28px;
  }

  .chart-card { border-radius: 12px; overflow: hidden; }
  .chart-card-head {
    display: flex; justify-content: space-between; align-items: center;
    padding: 12px 16px 8px;
    border-bottom: 1px solid #f1f1f5;
  }
  .chart-title { font-size: 13px; font-weight: 600; color: #374151; }
  .chart-head-controls { display: flex; align-items: center; gap: 6px; }

  .chart-el { width: 100%; height: 280px; }
  .chart-el--heat { height: 260px; }

  .date-select {
    border: 1px solid #e5e7eb; border-radius: 6px;
    padding: 3px 8px; font-size: 11.5px;
    background: #fff; color: #374151; cursor: pointer;
  }

  /* ── Compare banner ── */
  .compare-banner {
    margin: 14px 28px 0;
    background: #eff6ff; border: 1px solid #bfdbfe; border-radius: 8px;
    padding: 8px 14px; font-size: 12.5px; color: #1e40af;
  }

  /* ── Responsive ── */
  @media (max-width: 900px) {
    .charts-grid { grid-template-columns: 1fr; }
  }
</style>
