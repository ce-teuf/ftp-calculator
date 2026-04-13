<script lang="ts">
  import { Download } from '@lucide/svelte';
  import type { AssignmentResult } from '$lib/api/client';
  import { ptLabel, fmtPct, fmtM, fmtAmt } from './utils';

  let { assign }: { assign: AssignmentResult } = $props();

  function exportCSV() {
    const a = assign;
    const headers = ['date','period_type','outstanding','ftp_rate','ftp_interest',...a.bucket_labels];
    const rows = a.time_steps.map(ts => [
      ts.date, ts.period_type,
      ts.kpis.total_outstanding.toFixed(2),
      (ts.kpis.weighted_ftp_rate * 100).toFixed(6),
      ts.kpis.ftp_interest_periodic.toFixed(2),
      ...a.bucket_labels.map((b: string) => ((ts.ftp_by_tenor[b] ?? 0) * 100).toFixed(6)),
    ]);
    const csv = [headers, ...rows].map(r => r.join(',')).join('\n');
    const blob = new Blob([csv], { type: 'text/csv' });
    const url = URL.createObjectURL(blob);
    const anchor = document.createElement('a');
    anchor.href = url; anchor.download = `ftp_${a.assignment_id.slice(0,8)}.csv`; anchor.click();
    URL.revokeObjectURL(url);
  }
</script>

<div class="data-toolbar">
  <span class="data-toolbar-label">
    {assign.time_step_count} pas · {assign.bucket_labels.length} tenors
  </span>
  <button class="btn-sm" onclick={exportCSV}>
    <Download size={12} /> Export CSV
  </button>
</div>
<div class="ts-table-wrap">
  <table class="ts-table">
    <thead>
      <tr>
        <th>Date</th>
        <th>Type</th>
        <th class="num">Encours</th>
        <th class="num">Taux FTP</th>
        <th class="num">Intérêts</th>
        {#each assign.bucket_labels as bl}
          <th class="num tenor">{bl}</th>
        {/each}
      </tr>
    </thead>
    <tbody>
      {#each assign.time_steps as ts}
        <tr class="row-{ts.period_type}">
          <td class="cell-date">{ts.date}</td>
          <td>
            <span class="pt-badge pt-badge--{ts.period_type}">
              {ptLabel(ts.period_type)}
            </span>
          </td>
          <td class="num">{fmtM(ts.kpis.total_outstanding)}</td>
          <td class="num ftp-cell">{fmtPct(ts.kpis.weighted_ftp_rate)}</td>
          <td class="num">{fmtAmt(ts.kpis.ftp_interest_periodic)}</td>
          {#each assign.bucket_labels as bl}
            <td class="num tenor">
              {((ts.ftp_by_tenor[bl] ?? 0) * 100).toFixed(4)}%
            </td>
          {/each}
        </tr>
      {/each}
    </tbody>
  </table>
</div>

<style>
  .data-toolbar {
    display: flex; justify-content: space-between; align-items: center;
    margin-bottom: 8px;
  }
  .data-toolbar-label { font-size: 12px; color: #6b7280; }

  .ts-table-wrap {
    overflow-x: auto; border-radius: 8px; border: 1px solid #e5e7eb;
    max-height: calc(100vh - 340px); overflow-y: auto;
  }
  .ts-table { width: 100%; border-collapse: collapse; font-size: 12px; }
  .ts-table th {
    padding: 7px 12px; background: #f9fafb;
    text-align: left; font-size: 11px; font-weight: 600; color: #6b7280;
    text-transform: uppercase; position: sticky; top: 0; z-index: 1;
    border-bottom: 1px solid #e5e7eb;
  }
  .ts-table th.num   { text-align: right; }
  .ts-table th.tenor { color: #7c3aed; }
  .ts-table td { padding: 6px 12px; border-bottom: 1px solid #f9fafb; vertical-align: middle; }
  .ts-table tr:last-child td { border-bottom: none; }
  .row-projected td      { background: rgba(254,243,199,0.3); }
  .row-contrafactual td  { background: rgba(237,233,254,0.3); }
  .cell-date { font-weight: 600; color: #374151; font-variant-numeric: tabular-nums; }
  .num       { text-align: right; font-variant-numeric: tabular-nums; color: #374151; }
  .ftp-cell  { color: #6366f1; font-weight: 600; }
  .tenor     { color: #7c3aed; }

  .pt-badge {
    display: inline-block; font-size: 10px; font-weight: 700;
    padding: 1px 6px; border-radius: 10px;
  }
  .pt-badge--observed      { background: #eff6ff; color: #1d4ed8; }
  .pt-badge--projected     { background: #fff7ed; color: #c2410c; }
  .pt-badge--contrafactual { background: #f5f3ff; color: #6d28d9; }
</style>
