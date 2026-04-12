<script lang="ts">
  import { executions, portfolios, analytics,
           type Execution, type Portfolio, type NimHeatmapResponse, type BucketStats } from '../api/client.ts';

  let history       = $state<Execution[]>([]);
  let portfolioList = $state<Portfolio[]>([]);
  let nimData       = $state<NimHeatmapResponse | null>(null);
  let loading       = $state(true);

  interface Kpis { total_outstanding: number; weighted_ftp_rate: number; total_ftp_int_monthly: number; }
  let lastKpis   = $state<Kpis | null>(null);
  let lastMethod = $state('—');
  let heatDim    = $state<'by_branch' | 'by_product' | 'by_seller'>('by_product');

  async function load() {
    loading = true;
    try {
      [history, portfolioList] = await Promise.all([executions.list(), portfolios.list()]);
      const last = history.find(e => e.status === 'completed');
      if (last?.result_json) {
        const parsed = JSON.parse(last.result_json);
        lastKpis   = parsed.kpis ?? null;
        lastMethod = last.method;
      }
      const ptf = portfolioList.find(p =>
        history.some(e => e.portfolio_id === p.id && e.status === 'completed'));
      if (ptf) nimData = await analytics.portfolioNim(ptf.id);
    } catch { /* ignore */ }
    finally { loading = false; }
  }

  function fmtRate(v: number | null | undefined) {
    if (v == null) return '—';
    return (v * 100).toFixed(3) + '%';
  }
  function fmtCcy(v: number) {
    const a = Math.abs(v);
    if (a >= 1e9) return (v / 1e9).toFixed(2) + ' Md';
    if (a >= 1e6) return (v / 1e6).toFixed(2) + ' M';
    if (a >= 1e3) return (v / 1e3).toFixed(1) + ' k';
    return v.toFixed(0);
  }
  function fmtBps(v: number | null | undefined) {
    if (v == null) return '—';
    return Math.round(v * 10000) + ' bps';
  }

  const completedCount = $derived(history.filter(e => e.status === 'completed').length);
  const avgDuration    = $derived(() => {
    const done = history.filter(e => e.status === 'completed' && e.duration_ms != null);
    if (!done.length) return '—';
    return (done.reduce((s, e) => s + (e.duration_ms ?? 0), 0) / done.length).toFixed(0) + ' ms';
  });

  // ── Method stats ────────────────────────────────────────────────────────────
  interface MethodStats { method: string; count: number; avgRate: number; avgOut: number; avgFtpInt: number; }
  const methodStats = $derived((): MethodStats[] => {
    const map = new Map<string, { rates: number[]; outs: number[]; ints: number[] }>();
    for (const ex of history) {
      if (ex.status !== 'completed' || !ex.result_json) continue;
      let kpis: Kpis | null = null;
      try { kpis = JSON.parse(ex.result_json).kpis ?? null; } catch { continue; }
      if (!kpis) continue;
      if (!map.has(ex.method)) map.set(ex.method, { rates: [], outs: [], ints: [] });
      const b = map.get(ex.method)!;
      b.rates.push(kpis.weighted_ftp_rate);
      b.outs.push(kpis.total_outstanding);
      b.ints.push(kpis.total_ftp_int_monthly);
    }
    const avg = (a: number[]) => a.reduce((s, v) => s + v, 0) / a.length;
    return [...map.entries()]
      .map(([m, b]) => ({ method: m, count: b.rates.length, avgRate: avg(b.rates), avgOut: avg(b.outs), avgFtpInt: avg(b.ints) }))
      .sort((a, b) => b.avgRate - a.avgRate);
  });

  function heatColor(v: number, min: number, max: number) {
    if (max === min) return '#eef1ff';
    const t = (v - min) / (max - min);
    return `hsl(${Math.round(234 - t * 180)},70%,${92 - t * 28}%)`;
  }
  const heatRange = $derived((): [number, number] => {
    const s = methodStats();
    if (!s.length) return [0, 0];
    const r = s.map(x => x.avgRate);
    return [Math.min(...r), Math.max(...r)];
  });

  // ── NIM heatmap ─────────────────────────────────────────────────────────────
  const nimBuckets = $derived((): [string, BucketStats][] => {
    if (!nimData) return [];
    const dim = nimData[heatDim] ?? {};
    return Object.entries(dim).sort((a, b) => (b[1].avg_nim ?? 0) - (a[1].avg_nim ?? 0));
  });
  const nimRange = $derived((): [number, number] => {
    const bs = nimBuckets();
    if (!bs.length) return [0, 0];
    const vals = bs.map(([, s]) => s.avg_nim ?? 0);
    return [Math.min(...vals), Math.max(...vals)];
  });
  function nimColor(v: number, min: number, max: number) {
    if (v < 0) return '#fee2e2';
    if (max <= 0) return '#eef1ff';
    const t = Math.min(1, (v - Math.max(0, min)) / (max - Math.max(0, min)));
    return `hsl(${Math.round(142 - t * 0)},${55 + t * 20}%,${92 - t * 28}%)`;
  }

  // ── NIM Waterfall ────────────────────────────────────────────────────────────
  const waterfall = $derived((): { label: string; value: number; type: 'pos' | 'neg' | 'net' }[] | null => {
    if (!lastKpis) return null;
    const r = lastKpis;
    const avgClientRate =
      nimData?.positions.reduce((s, p) => s + (p.client_rate ?? 0) * p.outstanding, 0) /
      (nimData?.positions.reduce((s, p) => s + p.outstanding, 0) || 1) ||
      (r.weighted_ftp_rate + 0.005);
    const clientRevenue = r.total_outstanding * avgClientRate;
    const ftpCost       = r.total_ftp_int_monthly * 12;
    const nimNet        = clientRevenue - ftpCost;
    return [
      { label: 'Revenus clients (annuel)', value: clientRevenue, type: 'pos' },
      { label: 'Coût FTP (annuel)',        value: -ftpCost,      type: 'neg' },
      { label: 'NIM net (annuel)',         value: nimNet,         type: 'net' },
    ];
  });
  const wfMax = $derived(() => {
    const wf = waterfall();
    if (!wf) return 1;
    return Math.max(...wf.map(b => Math.abs(b.value)), 1);
  });

  $effect(() => { load(); });
</script>

<div class="tab-content">
  <!-- Header -->
  <div class="tab-header">
    <h2>Dashboard FTP</h2>
    <div class="hdr-actions">
      <button class="btn-sm" onclick={load}>↻ Rafraîchir</button>
      <a class="btn-sm" href="http://localhost:3000/api/export" download>⬇ Export JSON</a>
    </div>
  </div>

  {#if loading}
    <div class="loading">Chargement des données...</div>
  {:else}

    <!-- KPI row -->
    <div class="kpi-grid">
      <div class="kpi-card kpi-primary">
        <div class="kpi-icon">📊</div>
        <div class="kpi-body">
          <div class="kpi-label">Encours total</div>
          <div class="kpi-value">{lastKpis ? fmtCcy(lastKpis.total_outstanding) : '—'}</div>
          <div class="kpi-sub">dernière exécution</div>
        </div>
      </div>
      <div class="kpi-card kpi-green">
        <div class="kpi-icon">📈</div>
        <div class="kpi-body">
          <div class="kpi-label">Taux FTP moyen</div>
          <div class="kpi-value">{lastKpis ? fmtRate(lastKpis.weighted_ftp_rate) : '—'}</div>
          <div class="kpi-sub">pondéré · {lastMethod}</div>
        </div>
      </div>
      <div class="kpi-card kpi-amber">
        <div class="kpi-icon">💶</div>
        <div class="kpi-body">
          <div class="kpi-label">Int. FTP mensuel</div>
          <div class="kpi-value">{lastKpis ? fmtCcy(lastKpis.total_ftp_int_monthly) : '—'}</div>
          <div class="kpi-sub">coût de refinancement</div>
        </div>
      </div>
      <div class="kpi-card kpi-neutral">
        <div class="kpi-icon">⚡</div>
        <div class="kpi-body">
          <div class="kpi-label">Exécutions complètes</div>
          <div class="kpi-value">{completedCount}</div>
          <div class="kpi-sub">durée moy. {avgDuration()}</div>
        </div>
      </div>
    </div>

    <!-- Row 2: waterfall + method heatmap -->
    <div class="two-col">
      {#if waterfall()}
        <div class="card section">
          <div class="section-hdr">
            <span class="section-title">Décomposition NIM</span>
            <span class="section-sub">annualisée</span>
          </div>
          <div class="waterfall">
            {#each waterfall()! as bar}
              <div class="wf-row">
                <div class="wf-label">{bar.label}</div>
                <div class="wf-track">
                  <div
                    class="wf-bar"
                    class:wf-pos={bar.type === 'pos'}
                    class:wf-neg={bar.type === 'neg'}
                    class:wf-net={bar.type === 'net'}
                    style="width:{Math.abs(bar.value) / wfMax() * 100}%"
                  >
                    <span class="wf-val">{fmtCcy(Math.abs(bar.value))}</span>
                  </div>
                </div>
              </div>
            {/each}
          </div>
        </div>
      {:else}
        <div class="card section empty-state"><p>Lancez un calcul pour voir la décomposition NIM.</p></div>
      {/if}

      {#if methodStats().length > 0}
        <div class="card section">
          <div class="section-hdr">
            <span class="section-title">Taux FTP par méthode</span>
            <span class="section-sub">bleu = bas · rouge = élevé</span>
          </div>
          <div class="heat-tiles">
            {#each methodStats() as s}
              {@const [mn, mx] = heatRange()}
              <div class="heat-tile" style="background:{heatColor(s.avgRate, mn, mx)}">
                <div class="heat-method">{s.method.toUpperCase()}</div>
                <div class="heat-rate">{fmtRate(s.avgRate)}</div>
                <div class="heat-meta">{s.count} run{s.count > 1 ? 's' : ''}</div>
                <div class="heat-meta">{fmtCcy(s.avgFtpInt)} €/mois</div>
              </div>
            {/each}
          </div>
        </div>
      {:else}
        <div class="card section empty-state"><p>Aucune exécution complète.</p></div>
      {/if}
    </div>

    <!-- NIM heatmap full width -->
    {#if nimBuckets().length > 0}
      <div class="card section">
        <div class="section-hdr">
          <span class="section-title">Heatmap NIM — {nimData?.method ?? ''}</span>
          <div class="dim-tabs">
            {#each (['by_product', 'by_branch', 'by_seller'] as const) as d}
              <button
                class="dim-btn"
                class:dim-active={heatDim === d}
                onclick={() => heatDim = d}
              >
                {d === 'by_product' ? 'Produit' : d === 'by_branch' ? 'Branche' : 'Vendeur'}
              </button>
            {/each}
          </div>
        </div>
        <div class="heat-tiles nim-tiles">
          {#each nimBuckets() as [key, stats]}
            {@const [mn, mx] = nimRange()}
            <div class="heat-tile" style="background:{nimColor(stats.avg_nim ?? 0, mn, mx)}">
              <div class="heat-method">{key}</div>
              <div class="nim-val" class:nim-pos={(stats.avg_nim ?? 0) > 0} class:nim-neg={(stats.avg_nim ?? 0) < 0}>
                {fmtBps(stats.avg_nim)}
              </div>
              <div class="heat-meta">{fmtRate(stats.avg_ftp_rate)} FTP</div>
              <div class="heat-meta">{stats.count} pos · {fmtCcy(stats.outstanding)}</div>
            </div>
          {/each}
        </div>
      </div>
    {/if}

    <!-- Recent executions -->
    <div class="card section">
      <div class="section-hdr">
        <span class="section-title">Exécutions récentes</span>
      </div>
      {#if history.length === 0}
        <div class="empty-state">
          <p>Aucune exécution. Lancez un calcul FTP dans l'onglet <strong>Exécutions</strong>.</p>
        </div>
      {:else}
        <div class="tbl-wrap">
          <table>
            <thead>
              <tr>
                <th>Label</th><th>Méthode</th><th>Statut</th>
                <th class="num">Durée</th><th class="num">Taux FTP</th><th>Date</th>
              </tr>
            </thead>
            <tbody>
              {#each history.slice(0, 20) as ex}
                {@const kpis = (() => { try { return JSON.parse(ex.result_json ?? '{}').kpis; } catch { return null; } })()}
                <tr>
                  <td>{ex.label || '—'}</td>
                  <td><span class="tag">{ex.method}</span></td>
                  <td><span class="badge badge-{ex.status}">{ex.status}</span></td>
                  <td class="num mono">{ex.duration_ms != null ? ex.duration_ms + ' ms' : '—'}</td>
                  <td class="num mono">{kpis ? fmtRate(kpis.weighted_ftp_rate) : '—'}</td>
                  <td class="muted">{ex.created_at?.slice(0, 16).replace('T', ' ')}</td>
                </tr>
              {/each}
            </tbody>
          </table>
        </div>
      {/if}
    </div>

  {/if}
</div>

<style>
/* ── Header ─────────────────────────────────────────────────────────────────── */
.hdr-actions { display: flex; gap: 8px; }

/* ── KPI grid ───────────────────────────────────────────────────────────────── */
.kpi-grid {
  display: grid;
  grid-template-columns: repeat(4, 1fr);
  gap: 16px;
  margin-bottom: 20px;
}
.kpi-card {
  background: #fff;
  border-radius: 12px;
  padding: 20px;
  display: flex;
  align-items: flex-start;
  gap: 14px;
  box-shadow: 0 1px 3px rgba(0,0,0,.06);
  border-left: 4px solid transparent;
}
.kpi-primary { border-left-color: #6366f1; }
.kpi-green   { border-left-color: #22c55e; }
.kpi-amber   { border-left-color: #f59e0b; }
.kpi-neutral { border-left-color: #94a3b8; }

.kpi-icon { font-size: 22px; line-height: 1; }
.kpi-body { flex: 1; }
.kpi-label { font-size: 12px; font-weight: 500; color: #6b7280; text-transform: uppercase; letter-spacing: .04em; margin-bottom: 4px; }
.kpi-value { font-size: 22px; font-weight: 700; color: #1a1a2e; line-height: 1.2; margin-bottom: 3px; }
.kpi-sub   { font-size: 11.5px; color: #9ca3af; }

/* ── Two-column layout ──────────────────────────────────────────────────────── */
.two-col { display: grid; grid-template-columns: 1fr 1fr; gap: 16px; margin-bottom: 16px; }

/* ── Section card ────────────────────────────────────────────────────────────── */
.section { overflow: hidden; margin-bottom: 0; }
.section-hdr {
  display: flex;
  align-items: baseline;
  gap: 8px;
  padding: 14px 18px;
  border-bottom: 1px solid #f3f4f6;
}
.section-title { font-size: 14px; font-weight: 700; color: #1a1a2e; }
.section-sub   { font-size: 11.5px; color: #9ca3af; }

/* ── Waterfall ───────────────────────────────────────────────────────────────── */
.waterfall { padding: 16px 18px; display: flex; flex-direction: column; gap: 12px; }
.wf-row { display: grid; grid-template-columns: 180px 1fr; gap: 10px; align-items: center; }
.wf-label { font-size: 12.5px; color: #6b7280; text-align: right; }
.wf-track { height: 28px; background: #f1f5f9; border-radius: 6px; overflow: hidden; }
.wf-bar {
  height: 100%;
  border-radius: 6px;
  display: flex;
  align-items: center;
  padding: 0 8px;
  min-width: 40px;
  transition: width .4s ease;
}
.wf-pos { background: #6366f1; }
.wf-neg { background: #f87171; }
.wf-net { background: #22c55e; }
.wf-val { color: #fff; font-size: 11.5px; font-weight: 700; white-space: nowrap; }

/* ── Heat tiles ──────────────────────────────────────────────────────────────── */
.heat-tiles {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
  padding: 14px 18px;
}
.nim-tiles { flex-wrap: wrap; }
.heat-tile {
  flex: 1;
  min-width: 110px;
  border-radius: 10px;
  padding: 12px 14px;
  text-align: center;
  cursor: default;
}
.heat-method { font-size: 11px; font-weight: 700; color: #374151; letter-spacing: .05em; text-transform: uppercase; margin-bottom: 4px; }
.heat-rate   { font-size: 17px; font-weight: 700; color: #1a1a2e; margin-bottom: 2px; }
.heat-meta   { font-size: 11px; color: #6b7280; }

.nim-val { font-size: 17px; font-weight: 700; margin-bottom: 2px; }
.nim-pos { color: #15803d; }
.nim-neg { color: #b91c1c; }

/* ── Dim tabs ─────────────────────────────────────────────────────────────────── */
.dim-tabs { display: flex; gap: 4px; margin-left: auto; }
.dim-btn {
  padding: 4px 10px; border: 1px solid #e5e7eb; border-radius: 6px;
  background: #fff; color: #6b7280; font-size: 12px; cursor: pointer;
  transition: all .12s;
}
.dim-btn:hover  { background: #f1f5f9; color: #374151; }
.dim-active     { background: #6366f1; color: #fff; border-color: #6366f1; font-weight: 600; }

/* ── Table ───────────────────────────────────────────────────────────────────── */
.tbl-wrap { overflow-x: auto; }
table { width: 100%; border-collapse: collapse; font-size: 13px; }
thead { background: #f8fafc; }
th {
  padding: 10px 14px;
  text-align: left;
  font-size: 11.5px;
  font-weight: 600;
  color: #6b7280;
  text-transform: uppercase;
  letter-spacing: .04em;
  border-bottom: 1px solid #f1f5f9;
  white-space: nowrap;
}
td { padding: 10px 14px; border-bottom: 1px solid #f8fafc; color: #374151; }
tr:last-child td { border-bottom: none; }
tr:hover td { background: #fafbff; }
.num  { text-align: right; }
.mono { font-family: 'JetBrains Mono', 'Fira Code', monospace; font-size: 12.5px; }
.muted { color: #9ca3af; font-size: 12px; }
</style>
