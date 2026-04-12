<script lang="ts">
  import { datasets as api, type Dataset } from '../api/client';

  // ── State ──────────────────────────────────────────────────────────────────
  let datasetList  = $state<Dataset[]>([]);
  let file         = $state<File | null>(null);
  let fileType     = $state<string>('');          // detected: contracts | branches | zip | …
  let fileHeaders  = $state<string[]>([]);
  let datasetMode  = $state<'new' | 'existing'>('new');
  let dsName       = $state('');
  let dsDesc       = $state('');
  let dsDate       = $state(new Date().toISOString().slice(0, 10));
  let selectedId   = $state('');
  let uploading    = $state(false);
  let result       = $state<{ dataset_id: string; imported: Record<string, number> } | null>(null);
  let uploadError  = $state('');

  // ── CSV type detection ─────────────────────────────────────────────────────
  const TYPE_LABELS: Record<string, { label: string; color: string }> = {
    contracts:      { label: 'Contrats',           color: '#6366f1' },
    branches:       { label: 'Branches',            color: '#0891b2' },
    business_units: { label: 'Business Units',      color: '#0284c7' },
    departments:    { label: 'Départements',        color: '#0369a1' },
    sellers:        { label: 'Vendeurs',            color: '#7c3aed' },
    treasuries:     { label: 'Trésoreries',         color: '#9333ea' },
    rate_curves:    { label: 'Courbes de taux',     color: '#16a34a' },
    rate_series:    { label: 'Séries historiques',  color: '#15803d' },
    unknown:        { label: 'Type inconnu',        color: '#94a3b8' },
    zip:            { label: 'Archive ZIP (multi)', color: '#f59e0b' },
  };

  function detectType(headers: string[]): string {
    const has = (k: string) => headers.includes(k);
    if (has('contract_id') || (has('contract_type') && has('notional'))) return 'contracts';
    if (has('branch_id') && has('branch_name')) return 'branches';
    if (has('bu_id') && has('bu_name')) return 'business_units';
    if (has('dept_id') && has('dept_name')) return 'departments';
    if (has('seller_id') && has('seller_code')) return 'sellers';
    if (has('treasury_id') || has('treasury_name')) return 'treasuries';
    if (has('tenors_json') && has('values_json')) return 'rate_curves';
    if (has('series_name') && has('obs_date')) return 'rate_series';
    return 'unknown';
  }

  async function onFileChange(e: Event) {
    const input = e.target as HTMLInputElement;
    file = input.files?.[0] ?? null;
    fileType = '';
    fileHeaders = [];
    result = null;
    uploadError = '';
    if (!file) return;

    if (file.name.endsWith('.zip')) {
      fileType = 'zip';
      return;
    }

    // Read first line of CSV to detect type
    const text = await file.text();
    const firstLine = text.split('\n')[0] ?? '';
    fileHeaders = firstLine.split(',').map(h => h.trim().replace(/^"|"$/g, ''));
    fileType = detectType(fileHeaders);
  }

  async function upload() {
    if (!file) { uploadError = 'Choisissez un fichier.'; return; }
    if (datasetMode === 'new' && !dsName.trim()) { uploadError = 'Nom du dataset requis.'; return; }
    if (datasetMode === 'existing' && !selectedId) { uploadError = 'Sélectionnez un dataset existant.'; return; }

    const fd = new FormData();
    fd.append('file', file);
    if (datasetMode === 'new') {
      fd.append('dataset_name', dsName.trim());
    } else {
      fd.append('dataset_id', selectedId);
    }

    uploading = true; uploadError = ''; result = null;
    try {
      result = await api.ingest(fd);
      await loadDatasets();
    } catch (e: any) {
      uploadError = e.message;
    } finally { uploading = false; }
  }

  async function loadDatasets() {
    try { datasetList = await api.list(); } catch {}
  }

  function totalImported(): number {
    if (!result) return 0;
    return Object.values(result.imported).reduce((a, b) => a + b, 0);
  }

  function csvSignature(type: string): string {
    const sigs: Record<string, string> = {
      contracts:      'contract_id, contract_type, notional, side, …',
      branches:       'branch_id, branch_code, branch_name, country, …',
      business_units: 'bu_id, bu_name, branch_id, currency, …',
      departments:    'dept_id, dept_name, bu_id, bu_name, …',
      sellers:        'seller_id, seller_code, first_name, bu_id, …',
      treasuries:     'treasury_id, branch_id, treasury_name, …',
      rate_curves:    'id, name, component, tenors_json, values_json, …',
      rate_series:    'series_name, obs_date, tenor, rate',
      zip:            'Archive contenant plusieurs CSV',
    };
    return sigs[type] ?? '';
  }

  $effect(() => { loadDatasets(); });
</script>

<div class="tab-content">
  <div class="tab-header">
    <div>
      <h2>Datasets Creator</h2>
      <p class="sub">Importez un ZIP (dataset complet) ou un CSV individuel. Le type est détecté automatiquement depuis les colonnes.</p>
    </div>
  </div>

  <div class="layout">
    <!-- ── Upload panel ── -->
    <div class="card upload-card">

      <!-- Drop zone -->
      <label class="dropzone" class:has-file={!!file}>
        <div class="drop-icon">{file ? '📄' : '📂'}</div>
        {#if !file}
          <div class="drop-text">Déposez un <strong>.zip</strong> ou <strong>.csv</strong> ici</div>
          <div class="drop-hint">ou cliquez pour parcourir</div>
        {:else}
          <div class="drop-text">{file.name}</div>
          <div class="drop-hint">{(file.size / 1024).toFixed(1)} Ko</div>
        {/if}
        <input type="file" accept=".csv,.zip,text/csv,application/zip" onchange={onFileChange} />
      </label>

      <!-- Detected type badge -->
      {#if fileType}
        {@const info = TYPE_LABELS[fileType] ?? TYPE_LABELS['unknown']}
        <div class="detected-row">
          <span class="detected-label">Type détecté :</span>
          <span class="type-badge" style="background:{info.color}20;color:{info.color};border-color:{info.color}40">
            {info.label}
          </span>
        </div>
        {#if fileHeaders.length > 0}
          <div class="headers-row">
            Colonnes : {fileHeaders.slice(0, 8).join(', ')}{fileHeaders.length > 8 ? ` … (${fileHeaders.length} total)` : ''}
          </div>
        {/if}
      {/if}

      <hr class="divider" />

      <!-- Dataset target -->
      <div class="mode-tabs">
        <button class="mode-btn" class:active={datasetMode==='new'}    onclick={() => datasetMode='new'}>Nouveau dataset</button>
        <button class="mode-btn" class:active={datasetMode==='existing'} onclick={() => datasetMode='existing'}>Dataset existant</button>
      </div>

      {#if datasetMode === 'new'}
        <div class="form-row">
          <label class="field">
            <span>Nom *</span>
            <input type="text" bind:value={dsName} placeholder="ex: Portfolio Q1 2026" />
          </label>
          <label class="field">
            <span>Date de référence</span>
            <input type="date" bind:value={dsDate} />
          </label>
        </div>
        <label class="field">
          <span>Description</span>
          <input type="text" bind:value={dsDesc} placeholder="Optionnel" />
        </label>
      {:else}
        <label class="field">
          <span>Dataset cible</span>
          <select bind:value={selectedId}>
            <option value="">— Sélectionner —</option>
            {#each datasetList as ds}
              <option value={ds.id}>{ds.name} ({ds.count_contracts} contrats)</option>
            {/each}
          </select>
        </label>
      {/if}

      {#if uploadError}
        <div class="alert-err">{uploadError}</div>
      {/if}

      <button class="btn-primary" onclick={upload} disabled={uploading || !file}>
        {uploading ? 'Import en cours…' : 'Importer'}
      </button>

      {#if result}
        <div class="result-box">
          <div class="result-title">
            <span class="result-ok">✓ {totalImported()} enregistrements importés</span>
          </div>
          <div class="result-breakdown">
            {#each Object.entries(result.imported) as [type, count]}
              {@const info = TYPE_LABELS[type] ?? TYPE_LABELS['unknown']}
              <div class="breakdown-row">
                <span class="breakdown-type" style="color:{info.color}">{info.label}</span>
                <span class="breakdown-count">{count}</span>
              </div>
            {/each}
          </div>
          <div class="result-id">Dataset ID : <code>{result.dataset_id}</code></div>
        </div>
      {/if}
    </div>

    <!-- ── Info panel ── -->
    <div class="info-panel">

      <div class="card info-card">
        <h4>Types de fichiers acceptés</h4>
        <div class="type-list">
          {#each Object.entries(TYPE_LABELS).filter(([k]) => k !== 'unknown') as [type, info]}
            <div class="type-row">
              <span class="type-badge-sm" style="background:{info.color}20;color:{info.color}">{info.label}</span>
              <span class="type-cols">{csvSignature(type)}</span>
            </div>
          {/each}
        </div>
      </div>

      <div class="card info-card">
        <h4>Générer un dataset complet</h4>
        <p>Le script <code>generate_dataset.py</code> produit un ZIP avec tous les types :</p>
        <pre class="code">cd data/datageneration_scripts/datasets
python3 generate_dataset.py \
  --name "Portfolio Q1 2026" \
  --max-contracts 500 \
  --filter-branch FR,ES</pre>
        <p class="hint">Le ZIP généré contient : contracts, entités, courbes, séries historiques.</p>
      </div>

      <div class="card info-card">
        <h4>Chargement direct depuis le serveur</h4>
        <p>Les datasets générés sur le serveur peuvent être chargés directement depuis l'onglet <strong>Explorer</strong> sans upload.</p>
      </div>
    </div>
  </div>
</div>


<style>
.sub { margin: .1rem 0 0; font-size: .83rem; color: #888; }
.layout { display: flex; gap: 1.25rem; align-items: flex-start; }

/* ── Card ── */
.card { background: white; border-radius: 10px; box-shadow: 0 1px 4px rgba(0,0,0,.09); }

/* ── Upload card ── */
.upload-card { flex: 0 0 420px; padding: 1.5rem; display: flex; flex-direction: column; gap: 1rem; }

/* ── Dropzone ── */
.dropzone {
  display: flex; flex-direction: column; align-items: center; gap: .4rem;
  border: 2px dashed #d1d5db; border-radius: 10px; padding: 1.75rem 1rem;
  cursor: pointer; transition: border-color .15s, background .15s;
  background: #fafbff; position: relative;
}
.dropzone:hover { border-color: #6366f1; background: #eef0ff; }
.dropzone.has-file { border-color: #6366f1; background: #eef0ff; border-style: solid; }
.dropzone input[type=file] { position: absolute; inset: 0; opacity: 0; cursor: pointer; }
.drop-icon { font-size: 2rem; }
.drop-text { font-size: .9rem; font-weight: 600; color: #444; text-align: center; }
.drop-hint { font-size: .78rem; color: #aaa; }

/* ── Detected type ── */
.detected-row { display: flex; align-items: center; gap: .6rem; }
.detected-label { font-size: .82rem; color: #777; }
.type-badge {
  display: inline-block; padding: 2px 10px; border-radius: 99px;
  font-size: .78rem; font-weight: 600; border: 1px solid;
}
.headers-row { font-size: .75rem; color: #aaa; font-family: monospace; line-height: 1.4; }

.divider { border: none; border-top: 1px solid #f0f0f0; margin: 0; }

/* ── Mode tabs ── */
.mode-tabs { display: flex; gap: .5rem; }
.mode-btn {
  flex: 1; padding: .42rem .7rem; border: 1px solid #e0e0e0; border-radius: 6px;
  background: #f8f9fa; cursor: pointer; font-size: .83rem; color: #555; transition: all .12s;
}
.mode-btn.active { background: #6366f1; color: white; border-color: #6366f1; font-weight: 600; }

/* ── Form fields ── */
.form-row { display: flex; gap: .75rem; }
.form-row .field { flex: 1; }
.field { display: flex; flex-direction: column; gap: .25rem; font-size: .83rem; color: #555; }
.field input, .field select {
  padding: .4rem .65rem; border: 1px solid #ddd; border-radius: 6px;
  font-size: .84rem; outline: none; transition: border-color .12s;
}
.field input:focus, .field select:focus { border-color: #6366f1; }

/* ── Alerts ── */
.alert-err { background: #fee2e2; color: #991b1b; border-radius: 6px; padding: .55rem .85rem; font-size: .83rem; }

/* ── Result ── */
.result-box {
  background: #f0fdf4; border: 1px solid #bbf7d0; border-radius: 8px;
  padding: 1rem; display: flex; flex-direction: column; gap: .5rem;
}
.result-title { font-size: .88rem; }
.result-ok { color: #16a34a; font-weight: 600; }
.result-breakdown { display: flex; flex-direction: column; gap: .25rem; }
.breakdown-row { display: flex; justify-content: space-between; font-size: .82rem; }
.breakdown-type { font-weight: 500; }
.breakdown-count { font-variant-numeric: tabular-nums; font-weight: 700; color: #374151; }
.result-id { font-size: .76rem; color: #888; }
.result-id code { background: #e5e7eb; border-radius: 3px; padding: 1px 4px; }

/* ── Info panel ── */
.info-panel { flex: 1; display: flex; flex-direction: column; gap: 1rem; }
.info-card { padding: 1.25rem; }
.info-card h4 { margin: 0 0 .75rem; font-size: .9rem; font-weight: 700; }
.info-card p { margin: 0 0 .6rem; font-size: .82rem; color: #555; line-height: 1.5; }
.hint { color: #999 !important; font-size: .78rem !important; }

/* ── Type list ── */
.type-list { display: flex; flex-direction: column; gap: .4rem; }
.type-row { display: flex; align-items: baseline; gap: .75rem; }
.type-badge-sm {
  display: inline-block; padding: 1px 8px; border-radius: 99px;
  font-size: .72rem; font-weight: 600; white-space: nowrap; flex-shrink: 0;
  min-width: 110px; text-align: center;
}
.type-cols { font-size: .74rem; color: #999; font-family: monospace; }

/* ── Code block ── */
.code {
  background: #1a1a2e; color: #a5f3fc; border-radius: 6px;
  padding: .75rem 1rem; font-size: .77rem; line-height: 1.7;
  white-space: pre-wrap; word-break: break-all; margin: 0 0 .6rem;
}

/* ── Button ── */
.btn-primary {
  padding: .5rem 1rem; border-radius: 7px; border: none; cursor: pointer;
  background: #6366f1; color: white; font-weight: 600; font-size: .88rem;
  transition: background .12s;
}
.btn-primary:hover:not(:disabled) { background: #4f52d9; }
.btn-primary:disabled { opacity: .55; cursor: not-allowed; }
</style>
