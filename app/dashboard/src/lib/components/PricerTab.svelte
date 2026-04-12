<script lang="ts">
  import { compute, type ComputeResponse } from '../api/client.ts';

  const METHODS = [
    { key:'stock',       label:'Stock (MMFTP)' },
    { key:'flux',        label:'Flux (Multi-Vintage)' },
    { key:'duration',    label:'Duration' },
    { key:'pool',        label:'Pool' },
    { key:'refinancing', label:'Refinancing / Forward Rate' },
    { key:'floating',    label:'Floating-Rate' },
  ];

  const STANDARD_TENORS = [1, 3, 6, 12, 24, 36, 60, 84, 120, 180, 240, 360]; // months

  let method     = $state('stock');
  let amount     = $state(1_000_000);
  let maturity   = $state(60);
  let clientRate = $state(0.045);
  let running    = $state(false);
  let result     = $state<ComputeResponse | null>(null);
  let error      = $state('');

  function buildProfile(matMonths: number): number[] {
    const ncols = STANDARD_TENORS.filter(t => t <= matMonths).length + 1;
    return Array.from({length: ncols}, (_, j) => Math.max(0, 1 - j / (ncols - 1)));
  }

  let rates = $state(
    Array.from({length: STANDARD_TENORS.length}, (_, i) => 0.04 + i * 0.001)
  );

  async function price() {
    running = true; error = ''; result = null;
    const profile = buildProfile(maturity);
    const ncols = profile.length;
    const rateRow = rates.slice(0, ncols - 1);
    try {
      result = await compute({
        method,
        portfolio_id: 'pricer',
        label: `Pricer ${method} — ${amount.toLocaleString()} @ ${maturity}M`,
        outstanding_json: JSON.stringify([amount]),
        profiles_json:    JSON.stringify([profile]),
        rates_json:       JSON.stringify([rateRow]),
      });
    } catch(e: any) { error = e.message; }
    finally { running = false; }
  }

  function fmtRate(v: number) { return (v * 100).toFixed(3) + '%'; }
  function fmtBps(v: number)  { return Math.round(v * 10000) + ' bps'; }

  const nim = $derived(() => {
    if (!result || !result.weighted_ftp_rate) return null;
    return clientRate - result.weighted_ftp_rate;
  });
</script>

<div class="pricer-layout">
  <div class="pricer-form card">
    <h2>Pricer — position unique</h2>
    <p class="subtitle">Calcul FTP à la volée pour une position type</p>

    {#if error}
      <div class="alert-error">{error}</div>
    {/if}

    <div class="form-grid">
      <label>
        Méthode
        <select bind:value={method}>
          {#each METHODS as m}
            <option value={m.key}>{m.label}</option>
          {/each}
        </select>
      </label>

      <label>
        Encours (€)
        <input type="number" bind:value={amount} step="10000" min="0" />
      </label>

      <label>
        Maturité (mois)
        <input type="number" bind:value={maturity} step="6" min="1" max="360" />
      </label>

      <label>
        Taux client (%)
        <input type="number" bind:value={clientRate} step="0.001" min="0" max="1" />
      </label>
    </div>

    <!-- Courbe de taux (valeurs par tenor) -->
    <div class="rates-section">
      <h4>Courbe de taux de marché</h4>
      <div class="rates-grid">
        {#each STANDARD_TENORS as tenor, i}
          <label class="rate-input">
            <span class="tenor-label">{tenor < 12 ? tenor+'M' : (tenor/12)+'Y'}</span>
            <input type="number" bind:value={rates[i]} step="0.001" min="0" max="0.2" />
          </label>
        {/each}
      </div>
    </div>

    <button class="btn-primary" onclick={price} disabled={running}>
      {running ? 'Calcul…' : '▶ Pricer'}
    </button>
  </div>

  <!-- Résultats -->
  {#if result}
    <div class="results-panel">
      <div class="result-kpis card">
        <h3>Résultats</h3>
        <div class="kpi-grid">
          <div class="kpi green">
            <div class="kpi-label">Taux FTP</div>
            <div class="kpi-value">{fmtRate(result.weighted_ftp_rate)}</div>
          </div>
          <div class="kpi blue">
            <div class="kpi-label">Taux client</div>
            <div class="kpi-value">{fmtRate(clientRate)}</div>
          </div>
          {#if nim() != null}
            <div class="kpi" class:kpi-pos={nim()! > 0} class:kpi-neg={nim()! < 0}>
              <div class="kpi-label">NIM</div>
              <div class="kpi-value">{fmtBps(nim()!)}</div>
            </div>
          {/if}
          <div class="kpi">
            <div class="kpi-label">Intérêts FTP/mois</div>
            <div class="kpi-value">{result.total_ftp_int_monthly.toLocaleString('fr-FR', {maximumFractionDigits:0})} €</div>
          </div>
          <div class="kpi">
            <div class="kpi-label">Durée calcul</div>
            <div class="kpi-value">{result.duration_ms} ms</div>
          </div>
        </div>
      </div>

      <!-- Profil d'amortissement -->
      {#if result.ftp_rate && result.ftp_rate[0]}
        <div class="card">
          <h3>Profil de taux FTP par tranche</h3>
          <div class="bar-chart">
            {#each result.ftp_rate[0] as v, j}
              <div class="bar-col">
                <div class="bar-fill" style="height: {Math.round(v * 5000)}px"
                     title="{fmtRate(v)}">
                </div>
                <div class="bar-label">T{j}</div>
                <div class="bar-val">{(v*100).toFixed(2)}%</div>
              </div>
            {/each}
          </div>
        </div>
      {/if}
    </div>
  {/if}
</div>

<style>
.pricer-layout { display: flex; gap: 1.25rem; flex-wrap: wrap; }
.pricer-form { flex: 1; min-width: 340px; }
.results-panel { flex: 1; min-width: 340px; display: flex; flex-direction: column; gap: 1rem; }

.card { background: #fff; border-radius: 12px; padding: 1.25rem; box-shadow: 0 1px 4px rgba(0,0,0,.08); }
.card h2 { margin: 0 0 .25rem; font-size: 1.05rem; font-weight: 700; }
.card h3 { margin: 0 0 .75rem; font-size: .95rem; font-weight: 600; }
.subtitle { margin: 0 0 1.25rem; font-size: .82rem; color: #666; }

.alert-error { background: #fce8e6; color: #c5221f; padding: .6rem .9rem;
  border-radius: 6px; margin-bottom: .8rem; font-size: .85rem; }

.form-grid { display: grid; grid-template-columns: 1fr 1fr; gap: .75rem; margin-bottom: 1rem; }
.form-grid label { display: flex; flex-direction: column; gap: .3rem; font-size: .85rem; color: #555; }
.form-grid select, .form-grid input { border: 1px solid #ddd; border-radius: 6px;
  padding: .4rem .6rem; font-size: .85rem; }

.rates-section { margin-bottom: 1rem; }
.rates-section h4 { font-size: .85rem; color: #555; margin: 0 0 .5rem; font-weight: 600; }
.rates-grid { display: grid; grid-template-columns: repeat(6, 1fr); gap: .4rem; }
.rate-input { display: flex; flex-direction: column; align-items: center; gap: .2rem; }
.tenor-label { font-size: .72rem; color: #777; font-weight: 600; }
.rate-input input { width: 100%; border: 1px solid #ddd; border-radius: 4px;
  padding: .25rem .3rem; font-size: .78rem; text-align: center; }

.btn-primary { background: #1a73e8; color: #fff; border: none; border-radius: 8px;
  padding: .55rem 1.4rem; font-size: .9rem; cursor: pointer; width: 100%; }
.btn-primary:hover { background: #1557b0; }
.btn-primary:disabled { background: #b0c4de; cursor: not-allowed; }

.kpi-grid { display: grid; grid-template-columns: repeat(3, 1fr); gap: .6rem; }
.kpi { background: #f8f9fa; border-radius: 8px; padding: .65rem; text-align: center; }
.kpi.green { background: #e8f5e9; }
.kpi.blue  { background: #e8f0fe; }
.kpi.kpi-pos { background: #e8f5e9; }
.kpi.kpi-neg { background: #fce8e6; }
.kpi-label { font-size: .72rem; color: #555; margin-bottom: .25rem; }
.kpi-value { font-size: 1.1rem; font-weight: 700; color: #333; }

.bar-chart { display: flex; align-items: flex-end; gap: 4px; overflow-x: auto;
  padding-bottom: .5rem; min-height: 120px; }
.bar-col { display: flex; flex-direction: column; align-items: center; gap: 2px; flex: 1; min-width: 30px; }
.bar-fill { background: linear-gradient(to top, #1a73e8, #8ab4f8); border-radius: 3px 3px 0 0;
  width: 100%; min-height: 4px; transition: height .3s; }
.bar-label { font-size: .65rem; color: #777; }
.bar-val { font-size: .62rem; color: #444; font-weight: 600; }
</style>
