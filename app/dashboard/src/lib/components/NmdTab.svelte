<script lang="ts">
  import { runoff, type RunoffModel } from '../api/client.ts';

  const EBA_WAL_CAP = 60;

  let historicalInput = $state('100,97,94,91,88,85,82,79,76,73,70,67,64,61,58,55,52,50,48,46,44,42,40');
  let lambda = $state(0);
  let wal = $state(0);
  let walCapped = $state(false);
  let profileMonths = $state(60);
  let coreRatio = $state(0.7);
  let calibrationDone = $state(false);
  let calibrationError = $state('');
  let profile = $state<number[]>([]);
  let volatileProfile = $state<number[]>([]);
  let modelName = $state('NMD Retail Core');
  let productType = $state('nmd');
  let saveError = $state('');
  let saveSuccess = $state('');
  let savedModels = $state<RunoffModel[]>([]);

  async function loadModels() {
    try { savedModels = await runoff.list(); } catch { /* ignore */ }
  }

  function calibrate() {
    calibrationError = ''; calibrationDone = false;
    const vols = historicalInput.split(',').map(s => parseFloat(s.trim())).filter(v => !isNaN(v));
    if (vols.length < 3) { calibrationError = 'Saisissez au moins 3 observations.'; return; }
    const v0 = vols[0];
    if (v0 === 0) { calibrationError = 'V(0) ne peut pas être 0.'; return; }
    let sumTY = 0, sumT2 = 0;
    for (let t = 1; t < vols.length; t++) {
      if (vols[t] <= 0) continue;
      const y = Math.log(vols[t] / v0);
      sumTY += t * y;
      sumT2 += t * t;
    }
    lambda = sumT2 > 0 ? Math.max(0, -sumTY / sumT2) : 0;
    wal = lambda > 0 ? 1 / lambda : Infinity;
    walCapped = wal > EBA_WAL_CAP;
    if (walCapped) wal = EBA_WAL_CAP;
    generateProfile();
    calibrationDone = true;
  }

  function generateProfile() {
    const effLambda = lambda > 0 ? lambda : 0.02;
    const n = profileMonths;
    profile = Array.from({ length: n + 1 }, (_, t) => Math.exp(-effLambda * t));
    volatileProfile = Array.from({ length: n + 1 }, (_, t) => Math.max(0, 1 - t / 12));
  }

  const combinedProfile = $derived(() =>
    profile.map((p, t) => coreRatio * p + (1 - coreRatio) * (volatileProfile[t] ?? 0))
  );

  const displayedWal = $derived(() => {
    if (!calibrationDone) return null;
    return walCapped ? `${EBA_WAL_CAP}M (plafonné EBA)` : `${wal.toFixed(1)}M`;
  });

  async function saveModel() {
    saveError = ''; saveSuccess = '';
    const prof = combinedProfile();
    try {
      await runoff.create({
        name: modelName,
        product_type: productType,
        category: 'retail',
        method: 'behavioral_exponential',
        profile_json: JSON.stringify(prof),
        parameters_json: JSON.stringify({
          lambda,
          wal: Math.min(wal, EBA_WAL_CAP),
          core_ratio: coreRatio,
          eba_capped: walCapped,
        }),
      });
      saveSuccess = `Modèle "${modelName}" enregistré.`;
      await loadModels();
    } catch(e: any) { saveError = e.message; }
  }

  async function deleteModel(id: string) {
    if (!confirm('Delete this model?')) return;
    await runoff.delete(id);
    await loadModels();
  }

  $effect(() => { loadModels(); });
  $effect(() => { if (calibrationDone) generateProfile(); });
</script>

<div class="nmd-layout">

  <!-- ── Calibration ── -->
  <div class="card calibration-card">
    <h2>NMD Calibration — exponential model</h2>
    <p class="subtitle">V(t) = V(0) × e<sup>−λt</sup> · Estimation OLS sur série historique</p>

    <label class="field">
Historical volume series (comma-separated)
      <input bind:value={historicalInput} 
            placeholder="100, 97, 94, ..."/>
      <span class="hint">One value per period (month). The first is V(0).</span>
    </label>

    <div class="params-grid">
      <label class="field">
Profile horizon (months)
        <input type="number" bind:value={profileMonths} min={12} max={360} />
        Core ratio (0–1)
        <input type="number" bind:value={coreRatio} min={0} max={1} step={0.01} />
        <span class="hint">Stable fraction of deposit (ALCO)</span>
      </label>
    </div>

    {#if calibrationError}
      <div class="alert-error">{calibrationError}</div>
    {/if}

    <button class="btn-primary" onclick={calibrate}>Calibrate λ</button>

    {#if calibrationDone}
      <div class="results-row">
        <div class="result-item">
          <span class="result-label">λ (taux de décroissance)</span>
          <span class="result-value">{lambda.toFixed(5)}</span>
        </div>
        <div class="result-item">
          <span class="result-label">WAL</span>
          <span class="result-value" class:warn={walCapped}>{displayedWal()}</span>
        </div>
        <div class="result-item">
          <span class="result-label">Ratio non-core</span>
          <span class="result-value">{((1 - coreRatio) * 100).toFixed(0)}%</span>
        </div>
        {#if walCapped}
          <div class="eba-warning">⚠ WAL capped at {EBA_WAL_CAP}M according to EBA NMD guidelines</div>
        {/if}
      </div>
    {/if}
  </div>

  <!-- ── Visualisation du profil ── -->
  {#if profile.length > 0}
    <div class="card">
      <h3>Profil combiné (core + volatile)</h3>
      <div class="profile-chart">
        {#each combinedProfile().slice(0, Math.min(61, combinedProfile().length)) as v, t}
          <div class="profile-col" title="t={t}: {(v*100).toFixed(1)}%">
            <div class="bar-core" style="height: {Math.round(coreRatio * profile[t] * 100)}px"></div>
            <div class="bar-volatile" style="height: {Math.round((1-coreRatio) * (volatileProfile[t] ?? 0) * 100)}px"></div>
          </div>
        {/each}
      </div>
      <div class="legend">
        <span class="legend-item core">Core ({(coreRatio*100).toFixed(0)}%)</span>
        <span class="legend-item volatile">Non-core ({((1-coreRatio)*100).toFixed(0)}%)</span>
      </div>
    </div>

    <!-- Sauvegarde -->
    <div class="card save-card">
      <h3>Save the runoff model</h3>
      {#if saveError}   <div class="alert-error">{saveError}</div> {/if}
      {#if saveSuccess} <div class="alert-success">{saveSuccess}</div> {/if}
      <div class="save-form">
        <label class="field">
          Model name
          <input type="text" bind:value={modelName} />
        </label>
        <label class="field">
          Product type
          <select bind:value={productType}>
            <option value="nmd">NMD (demand deposits)</option>
            <option value="savings">Regulated savings</option>
            <option value="current_account">Current account</option>
          </select>
        </label>
      </div>
      <button class="btn-primary" onclick={saveModel}>Save</button>
    </div>
  {/if}

  <!-- ── Modèles existants ── -->
  <div class="card">
<h3>Saved runoff models</h3>
      {#if savedModels.length === 0}
        <div class="empty">No model saved.</div>
    {:else}
      <table>
        <thead>
          <tr><th>Nom</th><th>Produit</th><th>Méthode</th><th>Statut</th><th>λ</th><th>WAL</th><th></th></tr>
        </thead>
        <tbody>
          {#each savedModels as m}
            {@const params = m.parameters_json ? JSON.parse(m.parameters_json) : {}}
            <tr>
              <td class="name">{m.name}</td>
              <td>{m.product_type}</td>
              <td>{m.method}</td>
              <td><span class="status-badge" class:approved={m.status==='approved'} class:draft={m.status==='draft'}>{m.status}</span></td>
              <td class="mono">{params.lambda ? params.lambda.toFixed(5) : '—'}</td>
              <td class="mono">{params.wal ? params.wal.toFixed(1)+'M' : '—'}</td>
              <td><button class="btn-del" onclick={() => deleteModel(m.id)}>✕</button></td>
            </tr>
          {/each}
        </tbody>
      </table>
    {/if}
  </div>

</div>

<style>
.nmd-layout { display: flex; flex-direction: column; gap: 1.25rem; }
.card { background: #fff; border-radius: 12px; padding: 1.25rem; box-shadow: 0 1px 4px rgba(0,0,0,.08); }
.card h2 { margin: 0 0 .25rem; font-size: 1.05rem; font-weight: 700; }
.card h3 { margin: 0 0 .75rem; font-size: .95rem; font-weight: 600; }
.subtitle { margin: 0 0 1.25rem; font-size: .82rem; color: #666; }

.field { display: flex; flex-direction: column; gap: .3rem; font-size: .85rem; color: #555; margin-bottom: .75rem; }
.field input, .field select { border: 1px solid #ddd; border-radius: 6px; padding: .4rem .6rem; font-size: .85rem; }
.hint { font-size: .75rem; color: #999; }
.params-grid { display: grid; grid-template-columns: 1fr 1fr; gap: .75rem; }

.alert-error { background: #fce8e6; color: #c5221f; padding: .5rem .8rem; border-radius: 6px;
  margin-bottom: .75rem; font-size: .83rem; }
.alert-success { background: #e8f5e9; color: #1b5e20; padding: .5rem .8rem; border-radius: 6px;
  margin-bottom: .75rem; font-size: .83rem; }

.btn-primary { background: #1a73e8; color: #fff; border: none; border-radius: 8px;
  padding: .45rem 1.2rem; font-size: .9rem; cursor: pointer; }
.btn-primary:hover { background: #1557b0; }

.results-row { display: flex; gap: 1rem; margin-top: 1rem; flex-wrap: wrap; align-items: flex-start; }
.result-item { background: #f8f9fa; border-radius: 8px; padding: .6rem 1rem; display: flex;
  flex-direction: column; gap: .2rem; }
.result-label { font-size: .72rem; color: #666; }
.result-value { font-size: 1.1rem; font-weight: 700; color: #333; }
.result-value.warn { color: #ea8600; }
.eba-warning { background: #fff3e0; color: #e65100; border: 1px solid #ffcc02;
  border-radius: 6px; padding: .4rem .8rem; font-size: .8rem; width: 100%; }

.profile-chart { display: flex; align-items: flex-end; gap: 2px; height: 120px; overflow-x: auto;
  padding-bottom: .3rem; }
.profile-col { display: flex; flex-direction: column-reverse; width: 12px; flex-shrink: 0; }
.bar-core { background: #1a73e8; }
.bar-volatile { background: #8ab4f8; }
.legend { display: flex; gap: 1rem; margin-top: .5rem; font-size: .78rem; }
.legend-item { display: flex; align-items: center; gap: .4rem; }
.legend-item::before { content: ''; display: inline-block; width: 12px; height: 12px; border-radius: 2px; }
.legend-item.core::before { background: #1a73e8; }
.legend-item.volatile::before { background: #8ab4f8; }

.save-form { display: grid; grid-template-columns: 1fr 1fr; gap: .75rem; margin-bottom: .75rem; }
.empty { color: #999; font-size: .85rem; padding: .5rem; text-align: center; }

table { width: 100%; border-collapse: collapse; font-size: .83rem; }
th, td { border-bottom: 1px solid #f0f0f0; padding: .4rem .7rem; }
th { background: #f8f9fa; font-weight: 600; color: #444; }
.name { font-weight: 500; }
.mono { font-family: monospace; font-size: .8rem; }
.status-badge { border-radius: 20px; padding: .15rem .6rem; font-size: .75rem; font-weight: 500; }
.status-badge.draft { background: #fff3e0; color: #e65100; }
.status-badge.approved { background: #e8f5e9; color: #1b5e20; }
.btn-del { background: none; border: none; color: #ea4335; cursor: pointer; padding: .1rem .4rem; }
.btn-del:hover { background: #fce8e6; border-radius: 4px; }
</style>
