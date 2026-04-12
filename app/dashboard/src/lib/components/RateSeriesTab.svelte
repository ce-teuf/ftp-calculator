<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import * as echarts from 'echarts';
  import { rateSeries, type SeriesInfo } from '../api/client';

  // ── State ──────────────────────────────────────────────────────────────────
  let seriesList = $state<SeriesInfo[]>([]);
  let selectedSeries = $state<string[]>([]);
  let fromDate = $state('2022-01-01');
  let toDate   = $state('2024-12-31');
  let selectedTenor = $state('');
  let loading = $state(false);
  let error = $state('');

  // Chart data
  let chartData = $state<Record<string, { date: string; tenor: string | null; rate: number }[]>>({});
  let totalRows = $state(0);

  // ECharts instances
  let timeChartEl: HTMLDivElement;
  let curveChartEl: HTMLDivElement;
  let timeChart: echarts.ECharts | null = null;
  let curveChart: echarts.ECharts | null = null;

  // Known tenors in display order
  const TENOR_ORDER = ['1M','3M','6M','1Y','2Y','3Y','5Y','7Y','10Y','20Y','30Y'];
  const COLORS = ['#6366f1','#10b981','#f59e0b','#ef4444','#8b5cf6','#06b6d4','#ec4899'];

  // ── Init ───────────────────────────────────────────────────────────────────
  onMount(async () => {
    try {
      const res = await rateSeries.names();
      seriesList = res.series;
      selectedSeries = res.series.map(s => s.name);
    } catch (e) {
      error = String(e);
    }
    timeChart  = echarts.init(timeChartEl,  undefined, { renderer: 'canvas' });
    curveChart = echarts.init(curveChartEl, undefined, { renderer: 'canvas' });
    window.addEventListener('resize', onResize);
    await loadData();
  });

  onDestroy(() => {
    window.removeEventListener('resize', onResize);
    timeChart?.dispose();
    curveChart?.dispose();
  });

  function onResize() {
    timeChart?.resize();
    curveChart?.resize();
  }

  // ── Load data ──────────────────────────────────────────────────────────────
  async function loadData() {
    if (!selectedSeries.length) return;
    loading = true; error = '';
    try {
      const res = await rateSeries.query({
        series: selectedSeries,
        from: fromDate,
        to: toDate,
        tenor: selectedTenor || undefined,
        limit: 50000,
      });
      chartData = res.data;
      totalRows = res.total_rows;
      renderTimeChart();
      renderCurveChart();
    } catch (e) {
      error = String(e);
    } finally {
      loading = false;
    }
  }

  // ── Render time-series chart ───────────────────────────────────────────────
  function renderTimeChart() {
    if (!timeChart) return;

    // Build one series per (series_name × tenor)
    const ecSeries: echarts.SeriesOption[] = [];
    let colorIdx = 0;

    for (const [name, points] of Object.entries(chartData)) {
      // Group by tenor
      const byTenor: Record<string, [string, number][]> = {};
      for (const pt of points) {
        const key = pt.tenor ?? 'ON';
        if (!byTenor[key]) byTenor[key] = [];
        byTenor[key].push([pt.date, pt.rate]);
      }

      // Sort tenors
      const tenors = Object.keys(byTenor).sort((a, b) => {
        const ia = TENOR_ORDER.indexOf(a); const ib = TENOR_ORDER.indexOf(b);
        return (ia < 0 ? 999 : ia) - (ib < 0 ? 999 : ib);
      });

      for (const tenor of tenors) {
        const pts = byTenor[tenor].sort((a, b) => a[0].localeCompare(b[0]));
        const color = COLORS[colorIdx % COLORS.length];
        colorIdx++;
        ecSeries.push({
          name: `${name} ${tenor}`,
          type: 'line',
          data: pts,
          symbol: 'none',
          lineStyle: { width: 1.5, color },
          itemStyle: { color },
          smooth: false,
        });
      }
    }

    timeChart.setOption({
      backgroundColor: '#fff',
      tooltip: {
        trigger: 'axis',
        formatter: (params: any[]) => {
          const date = params[0]?.axisValue ?? '';
          const lines = params.map((p: any) =>
            `<span style="color:${p.color}">●</span> ${p.seriesName}: <b>${Number(p.value[1]).toFixed(3)}%</b>`
          );
          return `<b>${date}</b><br/>${lines.join('<br/>')}`;
        },
      },
      legend: {
        top: 8, type: 'scroll',
        textStyle: { fontSize: 11 },
      },
      grid: { top: 48, bottom: 48, left: 56, right: 20 },
      xAxis: {
        type: 'time',
        axisLabel: { fontSize: 11 },
        splitLine: { show: false },
      },
      yAxis: {
        type: 'value',
        name: 'Rate (%)',
        nameTextStyle: { fontSize: 11 },
        axisLabel: { fontSize: 11, formatter: (v: number) => `${v.toFixed(1)}%` },
        splitLine: { lineStyle: { color: '#f0f0f0' } },
      },
      series: ecSeries,
    }, true);
  }

  // ── Render term structure chart (last available date) ─────────────────────
  function renderCurveChart() {
    if (!curveChart) return;

    const ecSeries: echarts.SeriesOption[] = [];
    let colorIdx = 0;

    for (const [name, points] of Object.entries(chartData)) {
      // Find the latest date
      const latestDate = points.reduce((best, p) => (p.date > best ? p.date : best), '');
      const latestPoints = points.filter(p => p.date === latestDate && p.tenor != null);

      if (!latestPoints.length) continue;

      // Sort by tenor order
      const sorted = latestPoints.sort((a, b) => {
        const ia = TENOR_ORDER.indexOf(a.tenor!); const ib = TENOR_ORDER.indexOf(b.tenor!);
        return (ia < 0 ? 999 : ia) - (ib < 0 ? 999 : ib);
      });

      const color = COLORS[colorIdx % COLORS.length];
      colorIdx++;
      ecSeries.push({
        name: `${name} (${latestDate})`,
        type: 'line',
        data: sorted.map(p => [p.tenor, p.rate]),
        symbol: 'circle',
        symbolSize: 6,
        lineStyle: { width: 2, color },
        itemStyle: { color },
        smooth: true,
      });
    }

    curveChart.setOption({
      backgroundColor: '#fff',
      tooltip: {
        trigger: 'axis',
        formatter: (params: any[]) => {
          const tenor = params[0]?.axisValue ?? '';
          const lines = params.map((p: any) =>
            `<span style="color:${p.color}">●</span> ${p.seriesName}: <b>${Number(p.value[1]).toFixed(3)}%</b>`
          );
          return `<b>${tenor}</b><br/>${lines.join('<br/>')}`;
        },
      },
      legend: {
        top: 8, type: 'scroll',
        textStyle: { fontSize: 11 },
      },
      grid: { top: 48, bottom: 48, left: 56, right: 20 },
      xAxis: {
        type: 'category',
        data: TENOR_ORDER,
        axisLabel: { fontSize: 11 },
        splitLine: { show: false },
      },
      yAxis: {
        type: 'value',
        name: 'Rate (%)',
        nameTextStyle: { fontSize: 11 },
        axisLabel: { fontSize: 11, formatter: (v: number) => `${v.toFixed(1)}%` },
        splitLine: { lineStyle: { color: '#f0f0f0' } },
      },
      series: ecSeries,
    }, true);
  }

  function toggleSeries(name: string) {
    if (selectedSeries.includes(name)) {
      selectedSeries = selectedSeries.filter(s => s !== name);
    } else {
      selectedSeries = [...selectedSeries, name];
    }
  }
</script>

<div class="tab-content">
  <div class="tab-header">
    <h2>Rate Series</h2>
    <span class="row-count">{totalRows.toLocaleString()} observations</span>
  </div>

  {#if error}
    <div class="alert-error">{error}</div>
  {/if}

  <!-- Controls -->
  <div class="controls card">
    <!-- Series pills -->
    <div class="control-row">
      <span class="ctrl-label">Series</span>
      <div class="pill-group">
        {#each seriesList as s}
          <button
            class="pill"
            class:pill--active={selectedSeries.includes(s.name)}
            onclick={() => toggleSeries(s.name)}
          >
            {s.name}
            <span class="pill-sub">{s.currency}</span>
          </button>
        {/each}
      </div>
    </div>

    <!-- Date range + tenor -->
    <div class="control-row control-row--inline">
      <label>
        From
        <input type="date" bind:value={fromDate} />
      </label>
      <label>
        To
        <input type="date" bind:value={toDate} />
      </label>
      <label>
        Tenor filter
        <select bind:value={selectedTenor}>
          <option value="">All tenors</option>
          {#each TENOR_ORDER as t}
            <option value={t}>{t}</option>
          {/each}
        </select>
      </label>
      <button class="btn-primary" onclick={loadData} disabled={loading}>
        {loading ? 'Loading…' : 'Apply'}
      </button>
    </div>
  </div>

  <!-- Charts -->
  <div class="charts-grid">
    <div class="card chart-card">
      <div class="chart-title">Historical rates over time</div>
      <div bind:this={timeChartEl} class="chart-area"></div>
    </div>
    <div class="card chart-card">
      <div class="chart-title">Term structure — latest date</div>
      <div bind:this={curveChartEl} class="chart-area"></div>
    </div>
  </div>
</div>

<style>
  .row-count {
    font-size: 12px;
    color: #9ca3af;
    font-weight: 400;
  }

  /* Controls card */
  .controls {
    padding: 16px 20px;
    margin-bottom: 20px;
    display: flex;
    flex-direction: column;
    gap: 14px;
  }

  .control-row {
    display: flex;
    align-items: center;
    gap: 12px;
    flex-wrap: wrap;
  }
  .control-row--inline label {
    flex-direction: row;
    align-items: center;
    gap: 6px;
    white-space: nowrap;
  }
  .control-row--inline input,
  .control-row--inline select {
    width: auto;
    min-width: 130px;
  }

  .ctrl-label {
    font-size: 12.5px;
    font-weight: 600;
    color: #374151;
    min-width: 54px;
  }

  /* Pills */
  .pill-group {
    display: flex;
    gap: 6px;
    flex-wrap: wrap;
  }
  .pill {
    display: inline-flex;
    align-items: center;
    gap: 5px;
    background: #f1f5f9;
    border: 1.5px solid #e2e8f0;
    border-radius: 20px;
    padding: 4px 12px;
    font-size: 12.5px;
    font-weight: 600;
    cursor: pointer;
    transition: background 120ms, border-color 120ms, color 120ms;
    color: #475569;
  }
  .pill:hover { background: #e0e7ff; border-color: #a5b4fc; }
  .pill--active {
    background: rgba(99,102,241,.12);
    border-color: #6366f1;
    color: #4338ca;
  }
  .pill-sub {
    font-size: 10.5px;
    font-weight: 400;
    opacity: .65;
  }

  /* Charts */
  .charts-grid {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 16px;
  }
  @media (max-width: 900px) {
    .charts-grid { grid-template-columns: 1fr; }
  }

  .chart-card {
    padding: 16px;
  }
  .chart-title {
    font-size: 13px;
    font-weight: 600;
    color: #374151;
    margin-bottom: 12px;
  }
  .chart-area {
    height: 320px;
  }
</style>
