<script lang="ts">
  import type { KpiData } from './utils';
  import { fmtPct, fmtM, fmtAmt } from './utils';

  let { kpis }: { kpis: KpiData } = $props();
</script>

<div class="kpi-strip">
  <div class="kpi-group">
    <span class="kpi-group-label">Encours</span>
    <div class="kpi-tile kpi-obs">
      <span class="kt-label">Observé (dernier)</span>
      <span class="kt-val">{fmtM(kpis.obsTotalOut)}</span>
    </div>
    <div class="kpi-tile kpi-proj">
      <span class="kt-label">Projeté (dernier)</span>
      <span class="kt-val">{fmtM(kpis.projTotalOut)}</span>
    </div>
  </div>
  <div class="kpi-group">
    <span class="kpi-group-label">Taux FTP moyen</span>
    <div class="kpi-tile kpi-obs">
      <span class="kt-label">Observé</span>
      <span class="kt-val">{fmtPct(kpis.obsAvgRate)}</span>
    </div>
    <div class="kpi-tile kpi-proj">
      <span class="kt-label">Projeté</span>
      <span class="kt-val">{fmtPct(kpis.projAvgRate)}</span>
    </div>
  </div>
  <div class="kpi-group">
    <span class="kpi-group-label">Intérêts FTP cumulés</span>
    <div class="kpi-tile kpi-obs">
      <span class="kt-label">Observés</span>
      <span class="kt-val">{fmtAmt(kpis.obsInt)}</span>
    </div>
    <div class="kpi-tile kpi-proj">
      <span class="kt-label">Projetés</span>
      <span class="kt-val">{fmtAmt(kpis.projInt)}</span>
    </div>
  </div>
  <div class="kpi-group">
    <span class="kpi-group-label">Caractéristiques</span>
    <div class="kpi-tile">
      <span class="kt-label">WAL (mois)</span>
      <span class="kt-val">{kpis.wal.toFixed(1)}</span>
    </div>
    <div class="kpi-tile">
      <span class="kt-label">Méthode</span>
      <span class="kt-val method-badge">{kpis.method}</span>
    </div>
  </div>
</div>

<style>
  .kpi-strip {
    display: flex; gap: 12px; padding: 12px 24px;
    background: #fff; border-bottom: 1px solid #e5e7eb;
    flex-shrink: 0; flex-wrap: wrap;
  }
  .kpi-group { display: flex; flex-direction: column; gap: 4px; }
  .kpi-group-label {
    font-size: 10px; font-weight: 700; color: #9ca3af;
    text-transform: uppercase; letter-spacing: .04em;
  }
  .kpi-tile {
    background: #f4f5f9; border-radius: 8px; padding: 6px 12px;
    display: flex; flex-direction: column; min-width: 110px;
    border: 1px solid #e5e7eb;
  }
  .kpi-obs  { border-left: 3px solid #6366f1; }
  .kpi-proj { border-left: 3px solid #f97316; }
  .kt-label { font-size: 10px; font-weight: 600; color: #9ca3af; }
  .kt-val   { font-size: 13px; font-weight: 700; color: #1a1a2e; }
  .method-badge {
    font-size: 12px; background: #eff6ff; color: #1d4ed8;
    padding: 1px 6px; border-radius: 4px;
  }
</style>
