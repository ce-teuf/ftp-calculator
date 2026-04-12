<script lang="ts">
  import { onMount, tick } from 'svelte';
  import * as echarts from 'echarts';
  import { executionsV3 } from '../api/client';
  import type { ExecutionV3, ExecutionV3Detail } from '../api/client';
  import { BarChart2, TrendingUp, RefreshCw } from '@lucide/svelte';

  let execList   = $state<ExecutionV3[]>([]);
  let selectedId = $state('');
  let exec       = $state<ExecutionV3Detail | null>(null);
  let loading    = $state(false);
  let error      = $state('');

  // ECharts instances
  let ftpLineEl:    HTMLElement | undefined;
  let outLineEl:    HTMLElement | undefined;
  let barEl:        HTMLElement | undefined;
  let ftpChart:     echarts.ECharts | null = null;
  let outChart:     echarts.ECharts | null = null;
  let barChart:     echarts.ECharts | null = null;

  // Selected linkers for multi-compare
  let selectedLinkers = $state<Set<string>>(new Set());

  function toggleLinker(id: string) {
    const s = new Set(selectedLinkers);
    if (s.has(id)) s.delete(id); else s.add(id);
    selectedLinkers = s;
    renderCharts();
  }

  function selectAll() {
    if (!exec?.result?.linkers) return;
    selectedLinkers = new Set(exec.result.linkers.map((l: any) => l.linker_id));
    renderCharts();
  }

  async function load() {
    if (!selectedId) { exec = null; return; }
    loading = true; error = '';
    try {
      exec = await executionsV3.get(selectedId);
      if (exec.result?.linkers?.length) {
        selectedLinkers = new Set(exec.result.linkers.map((l: any) => l.linker_id));
      }
      await tick();
      initCharts();
      renderCharts();
    } catch (e: any) { error = e.message; }
    finally { loading = false; }
  }

  function initCharts() {
    if (ftpLineEl && !ftpChart) ftpChart = echarts.init(ftpLineEl, null, { renderer: 'canvas' });
    if (outLineEl  && !outChart)  outChart  = echarts.init(outLineEl,  null, { renderer: 'canvas' });
    if (barEl      && !barChart)  barChart  = echarts.init(barEl,      null, { renderer: 'canvas' });
  }

  function renderCharts() {
    if (!exec?.result?.linkers) return;

    const linkers: any[] = exec.result.linkers.filter(
      (l: any) => selectedLinkers.has(l.linker_id)
    );

    // ── FTP Rate over time ────────────────────────────────────────────────────
    if (ftpChart) {
      const series = linkers.map((l: any) => ({
        name: l.label ?? l.linker_name ?? l.linker_id,
        type: 'line',
        smooth: true,
        symbol: 'circle',
        symbolSize: 5,
        data: (l.analysis_times ?? [])
          .filter((t: any) => t.kpis)
          .map((t: any) => [t.date, +(t.kpis.weighted_ftp_rate * 100).toFixed(5)]),
      }));

      const allDates = linkers
        .flatMap((l: any) =>
          (l.analysis_times ?? []).filter((t: any) => t.kpis).map((t: any) => t.date)
        )
        .filter((v, i, a) => a.indexOf(v) === i)
        .sort();

      ftpChart.setOption({
        backgroundColor: '#fff',
        tooltip: { trigger: 'axis', formatter: (p: any) => {
          if (!Array.isArray(p)) p = [p];
          return p[0].name + '<br/>' + p.map((s: any) => `${s.marker}${s.seriesName}: <b>${s.value[1].toFixed(4)}%</b>`).join('<br/>');
        }},
        legend: { top: 8, type: 'scroll' },
        grid:   { top: 48, bottom: 40, left: 60, right: 20 },
        xAxis:  { type: 'time', axisLabel: { fontSize: 11 } },
        yAxis:  { type: 'value', name: 'FTP Rate (%)', axisLabel: { formatter: '{value}%', fontSize: 11 }, scale: true },
        series,
      }, true);
    }

    // ── Outstanding over time ─────────────────────────────────────────────────
    if (outChart) {
      const series = linkers.map((l: any) => ({
        name: l.label ?? l.linker_name ?? l.linker_id,
        type: 'line',
        smooth: true,
        areaStyle: { opacity: .08 },
        data: (l.analysis_times ?? [])
          .filter((t: any) => t.kpis)
          .map((t: any) => [t.date, t.kpis.total_outstanding]),
      }));

      outChart.setOption({
        backgroundColor: '#fff',
        tooltip: { trigger: 'axis', formatter: (p: any) => {
          if (!Array.isArray(p)) p = [p];
          return p[0].name + '<br/>' + p.map((s: any) => {
            const v = s.value[1];
            const fmt = v >= 1e9 ? (v/1e9).toFixed(2)+' Mrd' : (v/1e6).toFixed(1)+' M';
            return `${s.marker}${s.seriesName}: <b>${fmt}</b>`;
          }).join('<br/>');
        }},
        legend: { top: 8, type: 'scroll' },
        grid:   { top: 48, bottom: 40, left: 80, right: 20 },
        xAxis:  { type: 'time', axisLabel: { fontSize: 11 } },
        yAxis:  { type: 'value', name: 'Outstanding', axisLabel: { fontSize: 11 }, scale: true },
        series,
      }, true);
    }

    // ── Bar chart: last period comparison ─────────────────────────────────────
    if (barChart) {
      const names: string[] = [];
      const ftpVals: number[] = [];
      const outVals: number[] = [];

      for (const l of linkers) {
        const times = (l.analysis_times ?? []).filter((t: any) => t.kpis);
        if (!times.length) continue;
        const last = times[times.length - 1];
        names.push(l.label ?? l.linker_name ?? l.linker_id.slice(0, 8));
        ftpVals.push(+(last.kpis.weighted_ftp_rate * 100).toFixed(4));
        outVals.push(last.kpis.total_outstanding);
      }

      barChart.setOption({
        backgroundColor: '#fff',
        tooltip: { trigger: 'axis' },
        legend: { top: 8 },
        grid:   { top: 48, bottom: 60, left: 60, right: 60 },
        xAxis:  { type: 'category', data: names, axisLabel: { rotate: 20, fontSize: 11 } },
        yAxis: [
          { type: 'value', name: 'FTP Rate (%)', axisLabel: { formatter: '{value}%', fontSize: 10 } },
          { type: 'value', name: 'Outstanding', axisLabel: { fontSize: 10 }, splitLine: { show: false } },
        ],
        series: [
          {
            name: 'Taux FTP',
            type: 'bar',
            data: ftpVals,
            itemStyle: { color: '#6366f1', borderRadius: [4,4,0,0] },
            label: { show: true, position: 'top', formatter: '{c}%', fontSize: 10 },
          },
          {
            name: 'Encours',
            type: 'bar',
            yAxisIndex: 1,
            data: outVals,
            itemStyle: { color: '#10b981', borderRadius: [4,4,0,0], opacity: .7 },
          },
        ],
      }, true);
    }
  }

  function handleResize() {
    ftpChart?.resize(); outChart?.resize(); barChart?.resize();
  }

  function fmt(n: number) { return (n * 100).toFixed(4) + '%'; }
  function fmtM(n: number) {
    if (Math.abs(n) >= 1e9) return (n/1e9).toFixed(2) + ' Mrd';
    if (Math.abs(n) >= 1e6) return (n/1e6).toFixed(1) + ' M';
    return n.toFixed(0);
  }

  onMount(async () => {
    try { execList = await executionsV3.list(); } catch { /**/ }
    window.addEventListener('resize', handleResize);
    return () => {
      window.removeEventListener('resize', handleResize);
      ftpChart?.dispose(); outChart?.dispose(); barChart?.dispose();
    };
  });
</script>

<div class="tab-content">
  <div class="tab-header">
    <h2>Dashboard</h2>
    <div class="header-controls">
      <select bind:value={selectedId} onchange={load} class="exec-select">
        <option value="">-- Select an execution --</option>
        {#each execList as e}
          <option value={e.id}>
            {e.label ?? e.id.slice(0,8)} · {e.study_name ?? '?'} · {e.created_at.slice(0,10)}
          </option>
        {/each}
      </select>
      {#if selectedId}
        <button class="btn-sm" onclick={load} title="Refresh">
          <RefreshCw size={13}/>
        </button>
      {/if}
    </div>
  </div>

  {#if error}<div class="alert-error">{error}</div>{/if}

  {#if !selectedId}
    <div class="empty-state" style="margin-top:40px">
      <BarChart2 size={40} color="#d1d5db"/>
      <p style="margin-top:12px">Select an execution to display the dashboard.</p>
    </div>

  {:else if loading}
    <p class="loading">Loading…</p>

  {:else if exec}
    <!-- ── Header strip ────────────────────────────────────────────────────── -->
    <div class="exec-strip">
      <div class="es-item">
        <span class="es-lbl">Study</span>
        <span class="es-val">{exec.study_name ?? '—'}</span>
      </div>
      <div class="es-item">
        <span class="es-lbl">Méthode</span>
        <span class="es-val"><span class="tag">{exec.method}</span></span>
      </div>
      <div class="es-item">
        <span class="es-lbl">Statut</span>
        <span class="es-val">
          <span class="badge badge-{exec.status === 'completed' ? 'approved' : exec.status === 'error' ? 'error' : 'pending'}">
            {exec.status}
          </span>
        </span>
      </div>
      <div class="es-item">
        <span class="es-lbl">Durée</span>
        <span class="es-val mono">{exec.duration_ms ?? '—'} ms</span>
      </div>
      <div class="es-item">
        <span class="es-lbl">Date</span>
        <span class="es-val mono">{exec.created_at.slice(0,16).replace('T',' ')}</span>
      </div>
    </div>

    {#if exec.result?.linkers?.length}
      <!-- ── Linker selector ─────────────────────────────────────────────── -->
      <div class="linker-selector card">
        <div class="ls-head">
          <span class="ls-title">Linkers à afficher</span>
          <button class="btn-sm" onclick={selectAll}>Tous</button>
        </div>
        <div class="ls-pills">
          {#each exec.result.linkers as lnk}
            <button
              class="lnk-pill"
              class:active={selectedLinkers.has(lnk.linker_id)}
              onclick={() => toggleLinker(lnk.linker_id)}
            >
              {lnk.label ?? lnk.linker_name ?? lnk.linker_id.slice(0,8)}
            </button>
          {/each}
        </div>
      </div>

      <!-- ── Global KPIs ─────────────────────────────────────────────────── -->
      {@const visLinkers = exec.result.linkers.filter((l: any) => selectedLinkers.has(l.linker_id))}
      {@const allTimes   = visLinkers.flatMap((l: any) => (l.analysis_times ?? []).filter((t: any) => t.kpis))}
      {#if allTimes.length > 0}
        {@const avgFtp  = allTimes.reduce((s: number, t: any) => s + t.kpis.weighted_ftp_rate, 0) / allTimes.length}
        {@const avgOut  = allTimes.reduce((s: number, t: any) => s + t.kpis.total_outstanding, 0) / allTimes.length}
        {@const avgInt  = allTimes.reduce((s: number, t: any) => s + t.kpis.total_ftp_int_monthly, 0) / allTimes.length}
        {@const nDates  = new Set(allTimes.map((t: any) => t.date)).size}

        <div class="kpi-grid">
          <div class="kpi-card kpi-ftp">
            <div class="kpi-val">{fmt(avgFtp)}</div>
            <div class="kpi-lbl">Taux FTP moy. pondéré</div>
            <TrendingUp size={18} class="kpi-icon"/>
          </div>
          <div class="kpi-card kpi-out">
            <div class="kpi-val">{fmtM(avgOut)}</div>
            <div class="kpi-lbl">Encours moyen</div>
          </div>
          <div class="kpi-card kpi-int">
            <div class="kpi-val">{fmtM(avgInt)}</div>
            <div class="kpi-lbl">Intérêts FTP / mois moy.</div>
          </div>
          <div class="kpi-card kpi-dates">
            <div class="kpi-val">{nDates}</div>
            <div class="kpi-lbl">Dates d'analyse</div>
          </div>
        </div>
      {/if}

      <!-- ── Charts ──────────────────────────────────────────────────────── -->
      <div class="charts-grid">
        <div class="chart-card card">
          <div class="chart-title"><TrendingUp size={14}/> Taux FTP pondéré dans le temps</div>
          <div bind:this={ftpLineEl} class="chart-el"></div>
        </div>

        <div class="chart-card card">
          <div class="chart-title">Encours dans le temps</div>
          <div bind:this={outLineEl} class="chart-el"></div>
        </div>
      </div>

      <div class="card chart-card chart-full">
        <div class="chart-title">Comparaison — dernière période</div>
        <div bind:this={barEl} class="chart-el"></div>
      </div>

      <!-- ── Per-linker summary table ────────────────────────────────────── -->
      <div class="card summary-table-wrap">
        <div class="chart-title">Résumé par linker (dernière période)</div>
        <table class="summary-table">
          <thead>
            <tr>
              <th>Linker</th>
              <th>Dernière date</th>
              <th>Taux FTP</th>
              <th>Encours</th>
              <th>Intérêts FTP / mois</th>
              <th>Tendance</th>
            </tr>
          </thead>
          <tbody>
            {#each exec.result.linkers as lnk}
              {@const times = (lnk.analysis_times ?? []).filter((t: any) => t.kpis)}
              {#if times.length > 0}
                {@const last  = times[times.length - 1]}
                {@const first = times[0]}
                {@const delta = last.kpis.weighted_ftp_rate - first.kpis.weighted_ftp_rate}
                <tr class:dimmed={!selectedLinkers.has(lnk.linker_id)}>
                  <td class="lnk-name">{lnk.label ?? lnk.linker_name ?? lnk.linker_id.slice(0,8)}</td>
                  <td class="mono">{last.date}</td>
                  <td class="td-ftp">{fmt(last.kpis.weighted_ftp_rate)}</td>
                  <td class="mono">{fmtM(last.kpis.total_outstanding)}</td>
                  <td class="mono">{fmtM(last.kpis.total_ftp_int_monthly)}</td>
                  <td class="td-trend">
                    {#if Math.abs(delta) < 0.00005}
                      <span class="trend-flat">→ stable</span>
                    {:else if delta > 0}
                      <span class="trend-up">↑ +{fmt(delta)}</span>
                    {:else}
                      <span class="trend-down">↓ {fmt(delta)}</span>
                    {/if}
                  </td>
                </tr>
              {/if}
            {/each}
          </tbody>
        </table>
      </div>

    {:else if exec.status === 'completed'}
      <div class="empty-state">Aucun résultat dans cette exécution.</div>
    {:else if exec.status === 'error'}
      <div class="alert-error">{exec.error ?? 'Erreur inconnue'}</div>
    {/if}
  {/if}
</div>

<style>
  .header-controls { display: flex; gap: 8px; align-items: center; }
  .exec-select { min-width: 340px; }

  /* Exec strip */
  .exec-strip {
    display: flex; gap: 0; margin-bottom: 20px;
    background: #fff; border: 1px solid #e5e7eb; border-radius: 12px;
    overflow: hidden;
  }
  .es-item {
    flex: 1; padding: 12px 16px;
    border-right: 1px solid #f3f4f6;
    display: flex; flex-direction: column; gap: 3px;
  }
  .es-item:last-child { border-right: none; }
  .es-lbl  { font-size: 11px; font-weight: 600; color: #9ca3af; text-transform: uppercase; letter-spacing: .04em; }
  .es-val  { font-size: 13.5px; font-weight: 600; color: #1a1a2e; }
  .mono    { font-family: monospace; font-size: 12.5px; }

  /* Linker selector */
  .linker-selector { padding: 14px 16px; margin-bottom: 20px; }
  .ls-head  { display: flex; align-items: center; justify-content: space-between; margin-bottom: 10px; }
  .ls-title { font-size: 13px; font-weight: 600; color: #374151; }
  .ls-pills { display: flex; flex-wrap: wrap; gap: 6px; }
  .lnk-pill {
    padding: 5px 14px; border-radius: 20px; font-size: 12.5px; font-weight: 500;
    border: 1.5px solid #e5e7eb; background: #fff; color: #6b7280;
    cursor: pointer; transition: all 120ms;
  }
  .lnk-pill:hover  { border-color: #6366f1; color: #6366f1; }
  .lnk-pill.active { border-color: #6366f1; background: #ede9fe; color: #4f46e5; }

  /* KPI grid */
  .kpi-grid {
    display: grid;
    grid-template-columns: repeat(4, 1fr);
    gap: 14px;
    margin-bottom: 20px;
  }
  .kpi-card {
    background: #fff; border-radius: 12px; padding: 18px 20px;
    border-left: 4px solid #e5e7eb;
    box-shadow: 0 1px 3px rgba(0,0,0,.05);
    position: relative; overflow: hidden;
  }
  .kpi-ftp  { border-left-color: #6366f1; }
  .kpi-out  { border-left-color: #10b981; }
  .kpi-int  { border-left-color: #f59e0b; }
  .kpi-dates{ border-left-color: #3b82f6; }
  .kpi-val  { font-size: 24px; font-weight: 800; color: #1a1a2e; margin-bottom: 4px; }
  .kpi-lbl  { font-size: 11.5px; color: #9ca3af; font-weight: 500; }
  .kpi-icon { position: absolute; right: 16px; top: 16px; color: #e5e7eb; }

  /* Charts */
  .charts-grid {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 16px;
    margin-bottom: 16px;
  }
  .chart-card { padding: 16px; }
  .chart-full { margin-bottom: 16px; }
  .chart-title {
    font-size: 13px; font-weight: 600; color: #374151;
    margin-bottom: 12px; display: flex; align-items: center; gap: 6px;
  }
  .chart-el { height: 240px; width: 100%; }

  /* Summary table */
  .summary-table-wrap { padding: 16px; overflow-x: auto; }
  .summary-table { width: 100%; border-collapse: collapse; font-size: 13px; }
  .summary-table th {
    text-align: left; padding: 8px 12px; font-weight: 600; color: #9ca3af;
    border-bottom: 2px solid #e5e7eb; font-size: 11.5px;
    text-transform: uppercase; letter-spacing: .04em;
  }
  .summary-table td { padding: 9px 12px; border-bottom: 1px solid #f3f4f6; }
  .summary-table tr.dimmed td { opacity: .4; }
  .lnk-name { font-weight: 600; color: #1a1a2e; }
  .td-ftp   { font-weight: 700; color: #6366f1; }
  .td-trend { }
  .trend-flat { color: #9ca3af; font-size: 12px; }
  .trend-up   { color: #ef4444; font-size: 12px; font-weight: 600; }
  .trend-down { color: #10b981; font-size: 12px; font-weight: 600; }
</style>
