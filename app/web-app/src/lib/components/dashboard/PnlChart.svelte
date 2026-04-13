<script lang="ts">
  import { tick } from 'svelte';
  import * as echarts from 'echarts';
  import type { AssignmentResult } from '$lib/api/client';
  import { SERIES_COLORS, hexToRgb, buildMarkArea, fmtAmt } from './utils';

  let { assignments }: { assignments: AssignmentResult[] } = $props();

  let el = $state<HTMLDivElement | null>(null);
  let chart: echarts.ECharts | null = null;

  function render() {
    if (!el) return;
    chart?.dispose();
    chart = echarts.init(el);

    const allDates = [...new Set(assignments.flatMap(a => a.time_steps.map(t => t.date)))].sort();
    let cumul = 0;
    const barSeries: any[] = [];

    assignments.forEach((a, ai) => {
      const color = SERIES_COLORS[ai % SERIES_COLORS.length];
      const label = a.pair_label ?? `${a.vector_name}`;
      const tsMap = new Map(a.time_steps.map(t => [t.date, t]));
      barSeries.push({
        name: label,
        type: 'bar', stack: 'pnl', yAxisIndex: 0,
        data: allDates.map(d => {
          const ts = tsMap.get(d);
          const v = ts?.kpis.ftp_interest_periodic ?? 0;
          return { value: Math.round(v), pt: ts?.period_type ?? 'observed' };
        }),
        itemStyle: {
          color: (p: any) => p.data?.pt === 'projected'
            ? `rgba(${hexToRgb(color)},0.4)` : `rgba(${hexToRgb(color)},0.8)`,
        },
        markArea: ai === 0 ? buildMarkArea(allDates, assignments) : undefined,
      });
    });

    const cumulSeries = {
      name: 'Cumulé',
      type: 'line', yAxisIndex: 1, smooth: false,
      data: allDates.map(d => {
        const v = assignments.reduce((s, a) => {
          const ts = a.time_steps.find(t => t.date === d);
          return s + (ts?.kpis.ftp_interest_periodic ?? 0);
        }, 0);
        cumul += v;
        return Math.round(cumul);
      }),
      lineStyle: { color: '#6b7280', width: 2, type: 'dashed' },
      itemStyle: { color: '#6b7280' }, symbol: 'none',
    };

    chart.setOption({
      backgroundColor: 'transparent',
      tooltip: {
        trigger: 'axis', axisPointer: { type: 'cross' },
        formatter: (params: any[]) => {
          const d = params[0]?.axisValue;
          let s = `<b>${d}</b><br/>`;
          for (const p of params) {
            if (p.value == null) continue;
            s += `${p.marker}${p.seriesName}: <b>${fmtAmt(p.value)}</b><br/>`;
          }
          return s;
        },
      },
      legend: { top: 6, textStyle: { fontSize: 11 } },
      grid: { top: 52, right: 80, bottom: 40, left: 72 },
      xAxis: {
        type: 'category', data: allDates,
        axisLabel: { fontSize: 11, rotate: allDates.length > 30 ? 35 : 0 },
      },
      yAxis: [
        { type: 'value', name: 'Intérêts FTP (€)',
          axisLabel: { fontSize: 11, formatter: (v: number) => fmtAmt(v) } },
        { type: 'value', name: 'Cumulé (€)',
          axisLabel: { fontSize: 11, formatter: (v: number) => fmtAmt(v) } },
      ],
      series: [...barSeries, cumulSeries],
    });
  }

  $effect(() => {
    if (!el || !assignments.length) return;
    void [assignments, el];
    tick().then(() => { if (el && assignments.length) render(); });
    return () => { chart?.dispose(); chart = null; };
  });
</script>

<div class="chart-legend-row">
  <div class="legend-pill legend-obs">■ Observé (plein)</div>
  <div class="legend-pill legend-proj">■ Projeté (pâle)</div>
  <div class="legend-pill legend-cumul">--- Cumulé</div>
</div>
<div class="chart-xl" bind:this={el}></div>

<style>
  .chart-xl { width: 100%; height: 320px; }
  .chart-legend-row {
    display: flex; gap: 12px; align-items: center; margin-bottom: 8px; flex-wrap: wrap;
  }
  .legend-pill  { font-size: 11px; font-weight: 600; }
  .legend-obs   { color: #6366f1; }
  .legend-proj  { color: #f97316; }
  .legend-cumul { color: #6b7280; font-weight: 400; }
</style>
