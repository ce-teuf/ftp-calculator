<script lang="ts">
  import { tick } from 'svelte';
  import * as echarts from 'echarts';
  import type { AssignmentResult } from '$lib/api/client';
  import { SERIES_COLORS } from './utils';

  let { assign, runoffDates }: {
    assign: AssignmentResult;
    runoffDates: string[];
  } = $props();

  let el = $state<HTMLDivElement | null>(null);
  let chart: echarts.ECharts | null = null;

  function render() {
    if (!el) return;
    chart?.dispose();
    chart = echarts.init(el);
    const a = assign;

    const series: any[] = runoffDates.map((rd, ri) => {
      const ts = a.time_steps.find(t => t.date === rd);
      if (!ts) return null;
      const outstanding = ts.kpis.total_outstanding;
      return {
        name: rd,
        type: 'bar', barGap: '10%',
        data: ts.profile.map((w: number) => +(outstanding * w / 1e6).toFixed(3)),
        itemStyle: { color: SERIES_COLORS[ri % SERIES_COLORS.length] },
        label: { show: false },
      };
    }).filter(Boolean);

    chart.setOption({
      backgroundColor: 'transparent',
      tooltip: {
        trigger: 'axis',
        formatter: (params: any[]) => {
          const bl = a.bucket_labels[params[0]?.dataIndex] ?? '';
          let s = `<b>Bucket ${bl}M</b><br/>`;
          for (const p of params) {
            if (p.value == null) continue;
            s += `${p.marker}${p.seriesName}: <b>${p.value} M€</b><br/>`;
          }
          return s;
        },
      },
      legend: { top: 6, textStyle: { fontSize: 11 } },
      grid: { top: 48, right: 20, bottom: 40, left: 60 },
      xAxis: {
        type: 'category', data: a.bucket_labels, name: 'Horizon (bucket)',
        axisLabel: {
          fontSize: 10, rotate: a.bucket_labels.length > 30 ? 45 : 0,
          interval: Math.floor(a.bucket_labels.length / 20),
        },
      },
      yAxis: {
        type: 'value', name: 'Encours (M€)',
        axisLabel: { fontSize: 11, formatter: (v: number) => v.toFixed(0)+'M' },
      },
      series,
    });
  }

  $effect(() => {
    if (!el || !runoffDates.length) return;
    void [assign, runoffDates.join(), el];
    tick().then(() => { if (el && runoffDates.length) render(); });
    return () => { chart?.dispose(); chart = null; };
  });
</script>

<div class="chart-section-label">Amortissement résiduel par horizon (M€)</div>
<div class="chart-lg" bind:this={el}></div>

<style>
  .chart-lg { width: 100%; height: 280px; }
  .chart-section-label { font-size: 12px; font-weight: 600; color: #6b7280; margin-bottom: 8px; }
</style>
