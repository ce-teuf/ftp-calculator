<script lang="ts">
  import { compute, portfolios, type Portfolio, type ComputeResponse } from '../api/client.ts';

  const SHOCKS: { key: string; label: string; shifts: number[] }[] = [
    { key: 'parallel_up_100',   label: '+100 bps (parallèle)',                   shifts: Array(12).fill(0.01) },
    { key: 'parallel_up_200',   label: '+200 bps (parallèle)',                   shifts: Array(12).fill(0.02) },
    { key: 'parallel_down_100', label: '−100 bps (parallèle)',                   shifts: Array(12).fill(-0.01) },
    { key: 'bear_flattening',   label: 'Bear flattening (+200 court / +50 long)',
      shifts: [0.02,0.018,0.015,0.012,0.009,0.007,0.005,0.004,0.003,0.002,0.001,0.0005] },
    { key: 'bull_steepening',   label: 'Bull steepening (−100 court / +100 long)',
      shifts: [-0.01,-0.008,-0.005,-0.002,0.001,0.003,0.005,0.006,0.007,0.008,0.009,0.01] },
    { key: 'inversion',         label: 'Courbe inversée (+200 court / −50 long)',
      shifts: [0.02,0.015,0.01,0.005,0,-0.001,-0.002,-0.003,-0.004,-0.0045,-0.005,-0.005] },
  ];

  const STANDARD_TENORS = [1, 3, 6, 12, 24, 36, 60, 84, 120, 180, 240, 360];
  const METHODS = [
    { key:'stock',    label:'Stock (MMFTP)' },
    { key:'flux',     label:'Flux (Multi-Vintage)' },
    { key:'duration', label:'Duration' },
  ];

  let portfolioList = $state<Portfolio[]>([]);
  let portfolioId   = $state('');
  let method        = $state('stock');
  let clientRate    = $state(0.045);
  let amount        = $state(1_000_000);
  let maturity      = $state(60);
  let baseRates     = $state(STANDARD_TENORS.map((_, i) => 0.04 + i * 0.001));
  let running       = $state(false);
  let error         = $state('');

  interface ScenarioResult {
    shock: typeof SHOCKS[0];
    result: ComputeResponse | null;
    err: string;
  }
  let results    = $state<ScenarioResult[]>([]);
  let baseResult = $state<ComputeResponse | null>(null);

  async function loadPortfolios() {
    try {
      portfolioList = await portfolios.list();
      if (portfolioList.length) portfolioId = portfolioList[0].id;
    } catch { /* ignore */ }
  }

  function buildProfile(matMonths: number): number[] {
    const ncols = STANDARD_TENORS.filter(t => t <= matMonths).length + 1;
    return Array.from({length: ncols}, (_, j) => Math.max(0, 1 - j / (ncols - 1)));
  }

  function buildRates(shifted: number[], ncols: number): number[] {
    return shifted.slice(0, ncols - 1);
  }

  async function runScenarios() {
    if (!portfolioId) { error = 'Sélectionnez un portefeuille.'; return; }
    running = true; error = ''; results = []; baseResult = null;
    const profile = buildProfile(maturity);
    const ncols = profile.length;
    const baseRateRow = buildRates(baseRates, ncols);
    try {
      baseResult = await compute({
        method, portfolio_id: portfolioId,
        label: `Scénario Base — ${method}`,
        outstanding_json: JSON.stringify([amount]),
        profiles_json: JSON.stringify([profile]),
        rates_json: JSON.stringify([baseRateRow]),
      });
      const promises = SHOCKS.map(async (shock) => {
        const shiftedRates = baseRates.map((r, i) => r + (shock.shifts[i] ?? 0));
        const shiftedRow = buildRates(shiftedRates, ncols);
        try {
          const res = await compute({
            method, portfolio_id: portfolioId,
            label: `Scénario ${shock.label} — ${method}`,
            outstanding_json: JSON.stringify([amount]),
            profiles_json: JSON.stringify([profile]),
            rates_json: JSON.stringify([shiftedRow]),
          });
          return { shock, result: res, err: '' } as ScenarioResult;
        } catch(e: any) {
          return { shock, result: null, err: e.message } as ScenarioResult;
        }
      });
      results = await Promise.all(promises);
    } catch(e: any) { error = e.message; }
    finally { running = false; }
  }

  function fmtRate(v: number | null | undefined) {
    if (v == null) return '—';
    return (v * 100).toFixed(3) + '%';
  }
  function fmtBps(v: number | null | undefined) {
    if (v == null) return '—';
    const bps = Math.round(v * 10000);
    return (bps >= 0 ? '+' : '') + bps + ' bps';
  }
  function nim(r: ComputeResponse | null): number | null {
    if (!r) return null;
    return clientRate - r.weighted_ftp_rate;
  }
  function deltaFtp(r: ComputeResponse | null): number | null {
    if (!r || !baseResult) return null;
    return r.weighted_ftp_rate - baseResult.weighted_ftp_rate;
  }
  function deltaNim(r: ComputeResponse | null): number | null {
    const n = nim(r), nb = nim(baseResult);
    if (n == null || nb == null) return null;
    return n - nb;
  }

  $effect(() => { loadPortfolios(); });
</script>

<div class="scenarios-layout">

  <!-- ── Paramètres ── -->
  <div class="card params-card">
    <h2>Analyse de scénarios BCBS</h2>
    <p class="subtitle">6 chocs de taux standard — parallèles, bear flattening, bull steepening, inversion</p>

    {#if error}
      <div class="alert-error">{error}</div>
    {/if}

    <div class="form-grid">
      <label class="field">
        Portefeuille
        <select bind:value={portfolioId}>
          <option value="">— Sélectionner —</option>
          {#each portfolioList as p}
            <option value={p.id}>{p.name}</option>
          {/each}
        </select>
      </label>

      <label class="field">
        Méthode FTP
        <select bind:value={method}>
          {#each METHODS as m}
            <option value={m.key}>{m.label}</option>
          {/each}
        </select>
      </label>

      <label class="field">
        Encours (€)
        <input type="number" bind:value={amount} step="10000" min="0" />
      </label>

      <label class="field">
        Maturité (mois)
        <input type="number" bind:value={maturity} step="6" min="1" max="360" />
      </label>

      <label class="field">
        Taux client (pour NIM)
        <input type="number" bind:value={clientRate} step="0.001" min="0" max="1" />
      </label>
    </div>

    <button class="btn-primary" onclick={runScenarios} disabled={running || !portfolioId}>
      {running ? 'Calcul des scénarios…' : '▶ Lancer les 6 scénarios'}
    </button>
  </div>

  <!-- ── Résultats ── -->
  {#if baseResult || results.length > 0}
    <div class="card">
      <h3>Résultats comparatifs</h3>
      <table>
        <thead>
          <tr>
            <th>Scénario</th>
            <th>Taux FTP</th>
            <th>Δ FTP</th>
            <th>NIM</th>
            <th>Δ NIM</th>
            <th>Visualisation</th>
          </tr>
        </thead>
        <tbody>
          <!-- Cas de base -->
          {#if baseResult}
            <tr class="base-row">
              <td><strong>Cas de base</strong></td>
              <td>{fmtRate(baseResult.weighted_ftp_rate)}</td>
              <td>—</td>
              <td>{fmtRate(nim(baseResult) ?? 0)}</td>
              <td>—</td>
              <td>
                <div class="mini-bars">
                  <div class="mini-bar ftp" style="width: {Math.min(100, baseResult.weighted_ftp_rate * 2000)}px" title="FTP"></div>
                  <div class="mini-bar nim-bar" style="width: {Math.max(0, Math.min(100, (nim(baseResult) ?? 0) * 2000))}px" title="NIM"></div>
                </div>
              </td>
            </tr>
          {/if}
          <!-- Scénarios choqués -->
          {#each results as sr}
            {@const df = deltaFtp(sr.result)}
            {@const dn = deltaNim(sr.result)}
            <tr class:shock-neg={df != null && df > 0.005} class:shock-pos={df != null && df < -0.005}>
              <td>{sr.shock.label}</td>
              {#if sr.result}
                <td>{fmtRate(sr.result.weighted_ftp_rate)}</td>
                <td class:delta-up={df != null && df > 0} class:delta-down={df != null && df < 0}>
                  {fmtBps(df)}
                </td>
                <td>{fmtRate(nim(sr.result) ?? 0)}</td>
                <td class:delta-down={dn != null && dn > 0} class:delta-up={dn != null && dn < 0}>
                  {fmtBps(dn)}
                </td>
                <td>
                  <div class="mini-bars">
                    <div class="mini-bar ftp" style="width: {Math.min(100, sr.result.weighted_ftp_rate * 2000)}px"></div>
                    <div class="mini-bar nim-bar" style="width: {Math.max(0, Math.min(100, (nim(sr.result) ?? 0) * 2000))}px"></div>
                  </div>
                </td>
              {:else}
                <td colspan="5" class="err-cell">{sr.err}</td>
              {/if}
            </tr>
          {/each}
        </tbody>
      </table>

      <div class="legend">
        <span class="legend-item ftp-leg">Taux FTP</span>
        <span class="legend-item nim-leg">NIM</span>
      </div>
    </div>
  {/if}

</div>

<style>
.scenarios-layout { display: flex; flex-direction: column; gap: 1.25rem; }
.card { background: #fff; border-radius: 12px; padding: 1.25rem; box-shadow: 0 1px 4px rgba(0,0,0,.08); }
.card h2 { margin: 0 0 .25rem; font-size: 1.05rem; font-weight: 700; }
.card h3 { margin: 0 0 .75rem; font-size: .95rem; font-weight: 600; }
.subtitle { margin: 0 0 1.25rem; font-size: .82rem; color: #666; }
.alert-error { background: #fce8e6; color: #c5221f; padding: .5rem .8rem; border-radius: 6px;
  margin-bottom: .75rem; font-size: .83rem; }

.form-grid { display: grid; grid-template-columns: repeat(3, 1fr); gap: .75rem; margin-bottom: 1rem; }
.field { display: flex; flex-direction: column; gap: .3rem; font-size: .85rem; color: #555; }
.field select, .field input { border: 1px solid #ddd; border-radius: 6px; padding: .4rem .6rem; font-size: .85rem; }

.btn-primary { background: #1a73e8; color: #fff; border: none; border-radius: 8px;
  padding: .5rem 1.4rem; font-size: .9rem; cursor: pointer; }
.btn-primary:hover { background: #1557b0; }
.btn-primary:disabled { background: #b0c4de; cursor: not-allowed; }

table { width: 100%; border-collapse: collapse; font-size: .83rem; }
th, td { border-bottom: 1px solid #f0f0f0; padding: .45rem .7rem; }
th { background: #f8f9fa; font-weight: 600; color: #444; }
.base-row { background: #f8f9fa; font-weight: 500; }
.shock-neg { background: #fff8f8; }
.shock-pos { background: #f8fff8; }
.delta-up { color: #c5221f; font-weight: 600; }
.delta-down { color: #34a853; font-weight: 600; }
.err-cell { color: #c5221f; font-size: .8rem; }

.mini-bars { display: flex; flex-direction: column; gap: 2px; }
.mini-bar { height: 6px; border-radius: 3px; min-width: 2px; }
.mini-bar.ftp { background: #1a73e8; }
.mini-bar.nim-bar { background: #34a853; }

.legend { display: flex; gap: 1rem; margin-top: .75rem; font-size: .78rem; }
.legend-item { display: flex; align-items: center; gap: .4rem; }
.legend-item::before { content: ''; display: inline-block; width: 20px; height: 6px; border-radius: 3px; }
.ftp-leg::before { background: #1a73e8; }
.nim-leg::before { background: #34a853; }
</style>
