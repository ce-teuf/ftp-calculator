<script lang="ts">
  import { tick } from 'svelte';
  import * as echarts from 'echarts';
  import type { AssignmentResult } from '$lib/api/client';
  import { SERIES_COLORS, C_FLUX, hexToRgb, buildMarkArea, buildMarkLine } from './utils';

  let { assignments, projBoundary }: {
    assignments: AssignmentResult[];
    projBoundary: string | null;
  } = $props();

  let el = $state<HTMLDivElement | null>(null);
  let chart: echarts.ECharts | null = null;

  function render() {
    if (!el) return;
    chart?.dispose();
    chart = echarts.init(el);

    const allDates = [...new Set(assignments.flatMap(a => a.time_steps.map(t => t.date)))].sort();
    const series: any[] = [];

    assignments.forEach((a, ai) => {
      const color = SERIES_COLORS[ai % SERIES_COLORS.length];
      const methodColor = a.method.includes('Flux') ? C_FLUX : color;
      const label = a.pair_label ?? `${a.vector_name} × ${a.schedule_name}`;
      const tsMap = new Map(a.time_steps.map(t => [t.date, t]));

      series.push(
        {
          name: `Encours ${label}`,
          type: 'bar', yAxisIndex: 1, barMaxWidth: 18,
          data: allDates.map(d => {
            const ts = tsMap.get(d);
            return ts ? { value: +(ts.kpis.total_outstanding / 1e6).toFixed(2), pt: ts.period_type } : null;
          }),
          itemStyle: {
            color: (p: any) => p.data?.pt === 'projected'
              ? `rgba(${hexToRgb(color)},0.35)` : `rgba(${hexToRgb(color)},0.7)`,
          },
          markArea: buildMarkArea(allDates, assignments),
        },
        {
          name: `FTP ${label}`,
          type: 'line', yAxisIndex: 0, smooth: true,
          data: allDates.map(d => {
            const ts = tsMap.get(d);
            return ts ? +((ts.kpis.weighted_ftp_rate * 100).toFixed(4)) : null;
          }),
          lineStyle: { color: methodColor, width: 2, type: 'solid' },
          itemStyle: { color: methodColor },
          symbol: 'none',
          markLine: ai === 0 ? buildMarkLine(projBoundary) : { data: [] },
        },
      );
    });

    chart.setOption({
      backgroundColor: 'transparent',
      tooltip: {
        trigger: 'axis',
        formatter: (params: any[]) => {
          const d = params[0]?.axisValue;
          let s = `<b>${d}</b><br/>`;
          for (const p of params) {
            if (p.value == null) continue;
            const unit = p.seriesName.startsWith('Encours') ? ' M€' : ' %';
            s += `${p.marker}${p.seriesName}: <b>${p.value}${unit}</b><br/>`;
          }
          return s;
        },
        axisPointer: { type: 'cross', crossStyle: { color: '#aaa' } },
      },
      legend: { top: 6, type: 'scroll', textStyle: { fontSize: 11 } },
      grid: { top: 52, right: 72, bottom: 40, left: 64 },
      xAxis: {
        type: 'category', data: allDates,
        axisLabel: { fontSize: 11, rotate: allDates.length > 30 ? 35 : 0 },
      },
      yAxis: [
        { type: 'value', name: 'FTP (%)', nameTextStyle: { fontSize: 11 },
          axisLabel: { fontSize: 11, formatter: (v: number) => v.toFixed(2)+'%' } },
        { type: 'value', name: 'Encours (M€)', nameTextStyle: { fontSize: 11 },
          axisLabel: { fontSize: 11, formatter: (v: number) => v.toFixed(0)+'M' } },
      ],
      series,
    });
  }

  $effect(() => {
    if (!el || !assignments.length) return;
    void [assignments, projBoundary, el];
    tick().then(() => { if (el && assignments.length) render(); });
    return () => { chart?.dispose(); chart = null; };
  });
</script>

<div class="chart-legend-row">
  <div class="legend-pill legend-obs">■ Observé</div>
  <div class="legend-pill legend-proj">■ Projeté</div>
  {#if assignments.some(a => a.method.includes('Flux'))}
    <div class="legend-pill legend-flux">■ Flux</div>
  {/if}
  {#if projBoundary}
    <div class="legend-pill legend-boundary">↕ Frontière : {projBoundary}</div>
  {/if}
</div>
<div class="chart-xl" bind:this={el}></div>

<style>
  .chart-xl { width: 100%; height: 320px; }
  .chart-legend-row {
    display: flex; gap: 12px; align-items: center; margin-bottom: 8px; flex-wrap: wrap;
  }
  .legend-pill   { font-size: 11px; font-weight: 600; }
  .legend-obs    { color: #6366f1; }
  .legend-proj   { color: #f97316; }
  .legend-flux   { color: #16a34a; }
  .legend-boundary { color: #9ca3af; font-size: 11px; font-weight: 400; }
</style>
