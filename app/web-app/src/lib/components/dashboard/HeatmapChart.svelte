<script lang="ts">
  import { tick } from 'svelte';
  import * as echarts from 'echarts';
  import type { AssignmentResult } from '$lib/api/client';

  let { assign }: { assign: AssignmentResult } = $props();

  let el = $state<HTMLDivElement | null>(null);
  let chart: echarts.ECharts | null = null;

  function render() {
    if (!el) return;
    chart?.dispose();
    chart = echarts.init(el);
    const a = assign;

    const dates   = a.time_steps.map(t => t.date);
    const buckets = a.bucket_labels;
    const data: [number, number, number][] = [];
    let minV = Infinity, maxV = -Infinity;

    a.time_steps.forEach((ts, di) => {
      buckets.forEach((bl: string, bi: number) => {
        const v = (ts.ftp_by_tenor[bl] ?? 0) * 100;
        data.push([di, bi, +v.toFixed(4)]);
        if (v < minV) minV = v;
        if (v > maxV) maxV = v;
      });
    });

    chart.setOption({
      backgroundColor: 'transparent',
      tooltip: {
        formatter: (p: any) => {
          const d = dates[p.data[0]]; const bl = buckets[p.data[1]];
          return `<b>${d}</b><br/>Tenor ${bl}M: <b>${p.data[2]}%</b>`;
        },
      },
      grid: { top: 40, right: 100, bottom: 60, left: 60 },
      xAxis: {
        type: 'category', data: dates, name: 'Date',
        axisLabel: {
          fontSize: 9, rotate: 45,
          interval: Math.max(0, Math.floor(dates.length / 20) - 1),
        },
        splitArea: { show: true },
      },
      yAxis: {
        type: 'category', data: buckets, name: 'Tenor (mois)',
        axisLabel: { fontSize: 9, interval: Math.floor(buckets.length / 15) },
        splitArea: { show: true },
      },
      visualMap: {
        min: minV, max: maxV, calculable: true,
        orient: 'vertical', right: 8, top: 40, bottom: 60,
        inRange: { color: ['#dbeafe','#6366f1','#1e1b4b'] },
        textStyle: { fontSize: 10 },
        formatter: (v: number) => v.toFixed(2)+'%',
      },
      series: [{
        type: 'heatmap', data,
        emphasis: { itemStyle: { shadowBlur: 4, shadowColor: 'rgba(0,0,0,0.3)' } },
      }],
    });
  }

  $effect(() => {
    if (!el) return;
    void [assign, el];
    tick().then(() => { if (el) render(); });
    return () => { chart?.dispose(); chart = null; };
  });
</script>

<div class="chart-section-label">
  Taux FTP par date × tenor — {assign.pair_label ?? assign.vector_name}
</div>
<div class="chart-heatmap" bind:this={el}></div>

<style>
  .chart-heatmap { width: 100%; height: 400px; }
  .chart-section-label { font-size: 12px; font-weight: 600; color: #6b7280; margin-bottom: 8px; }
</style>
