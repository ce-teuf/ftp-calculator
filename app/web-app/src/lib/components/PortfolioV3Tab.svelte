<script lang="ts">
  import { onMount } from 'svelte';
  import { portfoliosV3 } from '../api/client';
  import type { PortfolioV3, PortfolioV3Detail, PortfolioV3Row } from '../api/client';
  import { Plus, Trash2, Upload, ChevronRight } from '@lucide/svelte';

  type View = 'list' | 'detail' | 'create';

  let view          = $state<View>('list');
  let portfolioList = $state<PortfolioV3[]>([]);
  let selected      = $state<PortfolioV3Detail | null>(null);
  let loading       = $state(true);
  let error         = $state('');

  // ── Create form ────────────────────────────────────────────────────────────
  let cName         = $state('');
  let cDescription  = $state('');
  let cType         = $state<'stock_amort' | 'new_prod_amort'>('stock_amort');
  let cSaving       = $state(false);
  let cError        = $state('');

  // ── Row upload ─────────────────────────────────────────────────────────────
  let rowLabel        = $state('');
  let scheduleFile    = $state<File | null>(null);
  let outstandingFile = $state<File | null>(null);
  let uploading       = $state(false);
  let uploadError     = $state('');
  let uploadSuccess   = $state('');

  // ── Preview ─────────────────────────────────────────────────────────────────
  let previewRow  = $state<PortfolioV3Row | null>(null);
  let previewOpen = $state(false);

  const FTP_TENORS = ['1M','3M','6M','12M','24M','36M','60M','84M','120M','180M','240M','360M'];

  async function load() {
    loading = true; error = '';
    try { portfolioList = await portfoliosV3.list(); }
    catch (e: any) { error = e.message; }
    finally { loading = false; }
  }

  async function openDetail(id: string) {
    selected = null;
    view = 'detail';
    try { selected = await portfoliosV3.get(id); }
    catch (e: any) { error = e.message; }
  }

  async function createPortfolio() {
    cError = '';
    if (!cName.trim()) { cError = 'Name required'; return; }
    cSaving = true;
    try {
      await portfoliosV3.create({ name: cName.trim(), description: cDescription || undefined, schedule_type: cType });
      await load();
      view = 'list';
      cName = ''; cDescription = '';
    } catch (e: any) { cError = e.message; }
    finally { cSaving = false; }
  }

  async function deletePortfolio(id: string, name: string) {
    if (!confirm(`Delete "${name}"?`)) return;
    await portfoliosV3.delete(id);
    await load();
    if (selected?.id === id) { selected = null; view = 'list'; }
  }

  async function uploadRow() {
    if (!selected) return;
    uploadError = ''; uploadSuccess = '';
    if (!scheduleFile || !outstandingFile) { uploadError = 'Both files are required.'; return; }
    uploading = true;
    try {
      const form = new FormData();
      form.append('schedule', scheduleFile);
      form.append('outstanding', outstandingFile);
      if (rowLabel.trim()) form.append('label', rowLabel.trim());
      await portfoliosV3.uploadRow(selected.id, form);
      uploadSuccess = 'Line added.';
      rowLabel = ''; scheduleFile = null; outstandingFile = null;
      selected = await portfoliosV3.get(selected.id);
    } catch (e: any) { uploadError = e.message; }
    finally { uploading = false; }
  }

  async function deleteRow(rowId: string) {
    if (!selected || !confirm('Delete this row?')) return;
    await portfoliosV3.deleteRow(rowId);
    selected = await portfoliosV3.get(selected.id);
  }

  async function loadPreview(rowId: string) {
    if (!selected) return;
    try {
      previewRow = await portfoliosV3.getRow(selected.id, rowId);
      previewOpen = true;
    } catch { previewRow = null; }
  }

  // Outstanding chart helpers
  function outstandingPoints(row: PortfolioV3Row): number[] {
    try {
      const arr = JSON.parse(row.outstanding_json) as {date:string;outstanding:number}[];
      return arr.map(r => r.outstanding);
    } catch { return []; }
  }

  function schedulePoints(row: PortfolioV3Row): number[][] {
    try { return JSON.parse(row.schedule_json); } catch { return []; }
  }

  onMount(load);
</script>

<div class="tab-content">
  <div class="tab-header">
    <h2>Portfolios</h2>
    <div style="display:flex;gap:8px">
      {#if view !== 'list'}
        <button class="btn-sm" onclick={() => { view = 'list'; selected = null; }}>
          ← Back
        </button>
      {/if}
      {#if view === 'list'}
        <button class="btn-primary" onclick={() => { view = 'create'; cError = ''; }}>
          <Plus size={14}/> New portfolio
        </button>
      {/if}
    </div>
  </div>

  {#if error}<div class="alert-error">{error}</div>{/if}

  <!-- ── List ──────────────────────────────────────────────────────────────── -->
  {#if view === 'list'}
    {#if loading}
<p class="loading">Loading…</p>
      {:else if portfolioList.length === 0}
        <div class="empty-state">
          <p>No portfolio defined.</p>
          <p>A portfolio contains one or more amortization profiles associated with an outstanding vector.</p>
          <button class="btn-primary" onclick={() => view = 'create'}><Plus size={14}/> Create</button>
      </div>
    {:else}
      <div class="pf-grid">
        {#each portfolioList as p}
          <div class="pf-card" onclick={() => openDetail(p.id)}>
            <div class="pfc-top">
              <span class="pfc-name">{p.name}</span>
              <span class="badge badge-{p.schedule_type === 'stock_amort' ? 'approved' : 'pending'}">
                {p.schedule_type === 'stock_amort' ? 'Stock' : 'New Prod'}
              </span>
            </div>
            {#if p.description}<div class="pfc-desc">{p.description}</div>{/if}
            <div class="pfc-meta">
              <span>{p.row_count} profil{p.row_count !== 1 ? 's' : ''}</span>
              <span>{p.created_at.slice(0,10)}</span>
            </div>
            <div class="pfc-actions" onclick={e => e.stopPropagation()}>
              <button class="btn-sm btn-danger" onclick={() => deletePortfolio(p.id, p.name)}>
                <Trash2 size={12}/>
              </button>
            </div>
          </div>
        {/each}
      </div>
    {/if}

  <!-- ── Create ─────────────────────────────────────────────────────────────── -->
  {:else if view === 'create'}
    <div class="form-card card">
      <h3 class="form-title">New portfolio</h3>

      <div class="form-grid">
        <label>
          Name
          <input bind:value={cName} placeholder="Ex: Real estate loans 2025"/>
        </label>
        <label>
          Description (optional)
          <input bind:value={cDescription}/>
        </label>
        <label style="grid-column:1/-1">
          Schedule type
          <div class="type-toggle">
            <button
              class="type-btn"
              class:active={cType === 'stock_amort'}
              onclick={() => cType = 'stock_amort'}
            >
              Stock amortization
              <span class="type-hint">existing stock profile</span>
            </button>
            <button
              class="type-btn"
              class:active={cType === 'new_prod_amort'}
              onclick={() => cType = 'new_prod_amort'}
            >
              New production
              <span class="type-hint">new production profile</span>
            </button>
          </div>
        </label>
      </div>

      {#if cError}<div class="alert-error">{cError}</div>{/if}

      <div class="form-footer">
        <button class="btn-sm" onclick={() => view = 'list'}>Cancel</button>
        <button class="btn-primary" onclick={createPortfolio} disabled={cSaving}>
          {cSaving ? 'Creating…' : 'Create the portfolio'}
        </button>
      </div>
    </div>

  <!-- ── Detail ─────────────────────────────────────────────────────────────── -->
  {:else if view === 'detail'}
    {#if !selected}
      <p class="loading">Chargement…</p>
    {:else}
      <div class="detail-layout">

        <!-- Left: portfolio info + upload -->
        <div class="detail-left">
          <div class="card pf-info">
            <div class="pfi-row">
              <strong>{selected.name}</strong>
              <span class="badge badge-{selected.schedule_type === 'stock_amort' ? 'approved' : 'pending'}">
                {selected.schedule_type === 'stock_amort' ? 'Stock amort' : 'New production'}
              </span>
            </div>
            {#if selected.description}<p class="pfi-desc">{selected.description}</p>{/if}
            <p class="pfi-meta">{selected.rows.length} profile{selected.rows.length !== 1 ? 's' : ''} loaded{selected.rows.length !== 1 ? 's' : ''}</p>
          </div>

          <!-- Upload new row -->
          <div class="card upload-panel">
            <h4>Add a profile</h4>
            <p class="upload-hint">
              Load two CSV: an amortization profile and an outstanding vector (same number of rows).
            </p>

            <label>
              Label (optional)
              <input bind:value={rowLabel} placeholder="Ex: Residential France"/>
            </label>

            <div class="file-row">
              <div class="file-drop" class:has-file={!!scheduleFile}>
                <Upload size={16}/>
                <span>{scheduleFile ? scheduleFile.name : 'Schedule CSV'}</span>
                <input
                  type="file" accept=".csv"
                  onchange={e => scheduleFile = (e.target as HTMLInputElement).files?.[0] ?? null}
                />
              </div>
              <div class="file-drop" class:has-file={!!outstandingFile}>
                <Upload size={16}/>
                <span>{outstandingFile ? outstandingFile.name : 'Outstanding CSV'}</span>
                <input
                  type="file" accept=".csv"
                  onchange={e => outstandingFile = (e.target as HTMLInputElement).files?.[0] ?? null}
                />
              </div>
            </div>

            <div class="csv-hint">
              <details>
                <summary>Expected format</summary>
                <div class="csv-example">
                  <strong>schedule.csv</strong>
                  <pre>date,m1,m3,m6,m12,m24,m36,m60,m84,m120,m180,m240,m360
2025-01-01,0.02,0.05,0.08,0.12,0.18,0.22,0.30,0.38,0.48,0.62,0.74,1.00
2025-02-01,0.02,0.05,0.08,0.12,...</pre>
                  <strong>outstanding.csv</strong>
                  <pre>date,outstanding
2025-01-01,1500000000
2025-02-01,1480000000</pre>
                </div>
              </details>
            </div>

            {#if uploadError}<div class="alert-error">{uploadError}</div>{/if}
            {#if uploadSuccess}<div class="alert-success">{uploadSuccess}</div>{/if}

            <button class="btn-primary" onclick={uploadRow} disabled={uploading}>
              {uploading ? 'Uploading…' : 'Add this profile'}
            </button>
          </div>
        </div>

        <!-- Right: rows list -->
        <div class="detail-right">
          {#if selected.rows.length === 0}
            <div class="empty-state">
              <p>No profile loaded.</p>
              <p>Utilisez le formulaire à gauche pour importer vos premiers CSV.</p>
            </div>
          {:else}
            <div class="rows-list">
              {#each selected.rows as row}
                <div class="row-card card">
                  <div class="rc-header">
                    <span class="rc-label">{row.label ?? `Profil #${row.row_order + 1}`}</span>
                    <div class="rc-actions">
                      <button class="btn-sm" onclick={() => loadPreview(row.id)}>
                        Preview <ChevronRight size={12}/>
                      </button>
                      <button class="btn-sm btn-danger" onclick={() => deleteRow(row.id)}>
                        <Trash2 size={12}/>
                      </button>
                    </div>
                  </div>

                  <!-- Mini outstanding sparkline -->
                  {#if outstandingPoints(row).length > 0}
                    {@const pts = outstandingPoints(row)}
                    {@const maxV = Math.max(...pts, 1)}
                    <div class="sparkline">
                      {#each pts as v}
                        <div class="spark-bar" style="height:{Math.max(2,(v/maxV)*32)}px"></div>
                      {/each}
                    </div>
                    <div class="rc-meta">
                      {pts.length} periods · max outstanding {(Math.max(...pts)/1e6).toFixed(0)} M
                    </div>
                  {/if}
                </div>
              {/each}
            </div>
          {/if}
        </div>
      </div>
    {/if}
  {/if}
</div>

<!-- ── Preview modal ──────────────────────────────────────────────────────────── -->
{#if previewOpen && previewRow}
  <div class="modal-backdrop" onclick={() => previewOpen = false}>
    <div class="modal" onclick={e => e.stopPropagation()}>
      <div class="modal-header">
        <strong>Preview — {previewRow.label ?? 'Profile'}</strong>
        <button class="btn-sm" onclick={() => previewOpen = false}>✕</button>
      </div>

      <div class="modal-body">
        <!-- Outstanding chart -->
        <h4>Encours</h4>
        {#if previewRow}
          {@const outArr = JSON.parse(previewRow.outstanding_json) as {date:string;outstanding:number}[]}
          {@const maxOut = Math.max(...outArr.map(r => r.outstanding), 1)}
          <div class="out-chart">
            {#each outArr as r, i}
              {#if i % Math.max(1, Math.floor(outArr.length / 40)) === 0}
                <div class="out-col" title="{r.date}: {(r.outstanding/1e6).toFixed(1)}M">
                  <div class="out-bar" style="height:{Math.max(2,(r.outstanding/maxOut)*80)}px"></div>
                </div>
              {/if}
            {/each}
          </div>
          <div class="preview-range">
            {outArr[0]?.date ?? ''} → {outArr[outArr.length-1]?.date ?? ''}
            · {outArr.length} périodes
          </div>

          <!-- Schedule (last row) -->
          <h4 style="margin-top:16px">Schedule (last profile)</h4>
          {@const schedArr = JSON.parse(previewRow.schedule_json) as {date:string;buckets:number[]}[]}
          {@const lastSched = schedArr[schedArr.length - 1]}
          {#if lastSched}
            <div class="sched-bars">
              {#each lastSched.buckets as v, ti}
                <div class="sched-col">
                  <div class="sched-bar" style="height:{Math.max(2, v * 120)}px" title="{FTP_TENORS[ti]}: {(v*100).toFixed(1)}%"></div>
                  <div class="sched-lbl">{FTP_TENORS[ti]}</div>
                </div>
              {/each}
            </div>
            <div class="preview-range">Date: {lastSched.date}</div>
          {/if}
        {/if}
      </div>
    </div>
  </div>
{/if}

<style>
  /* ── Portfolio grid ─────────────────────────────────────────────────────────── */
  .pf-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(260px,1fr));
    gap: 14px;
  }

  .pf-card {
    background: #fff;
    border: 1px solid #e5e7eb;
    border-radius: 12px;
    padding: 16px;
    cursor: pointer;
    position: relative;
    transition: border-color 150ms, box-shadow 150ms;
  }
  .pf-card:hover { border-color: #6366f1; box-shadow: 0 0 0 3px rgba(99,102,241,.08); }
  .pfc-top  { display:flex; justify-content:space-between; align-items:flex-start; gap:8px; margin-bottom:6px; }
  .pfc-name { font-weight:600; font-size:14px; color:#1a1a2e; }
  .pfc-desc { font-size:12px; color:#6b7280; margin-bottom:6px; }
  .pfc-meta { display:flex; justify-content:space-between; font-size:12px; color:#9ca3af; }
  .pfc-actions { position:absolute; bottom:12px; right:12px; opacity:0; transition:opacity 150ms; }
  .pf-card:hover .pfc-actions { opacity:1; }

  /* ── Create form ────────────────────────────────────────────────────────────── */
  .form-card { padding:24px; max-width:600px; }
  .form-title { font-size:16px; font-weight:700; margin-bottom:20px; }
  .form-grid { display:grid; grid-template-columns:1fr 1fr; gap:12px; margin-bottom:16px; }
  .form-footer { display:flex; justify-content:flex-end; gap:10px; padding-top:16px; border-top:1px solid #e5e7eb; }

  .type-toggle { display:flex; gap:10px; margin-top:4px; }
  .type-btn {
    flex:1; padding:12px; border:2px solid #e5e7eb; border-radius:10px;
    background:#fff; cursor:pointer; text-align:left; font-size:13px;
    font-weight:600; color:#374151; transition:border-color 150ms, background 150ms;
    display:flex; flex-direction:column; gap:4px;
  }
  .type-btn:hover { border-color:#6366f1; }
  .type-btn.active { border-color:#6366f1; background:#ede9fe; color:#4f46e5; }
  .type-hint { font-size:11.5px; font-weight:400; color:#9ca3af; }
  .type-btn.active .type-hint { color:#7c70cc; }

  /* ── Detail layout ──────────────────────────────────────────────────────────── */
  .detail-layout { display:grid; grid-template-columns:340px 1fr; gap:20px; align-items:start; }
  .detail-left { display:flex; flex-direction:column; gap:14px; }

  .pf-info { padding:16px; }
  .pfi-row { display:flex; align-items:center; justify-content:space-between; margin-bottom:6px; font-size:15px; font-weight:600; }
  .pfi-desc { font-size:12.5px; color:#6b7280; margin-bottom:4px; }
  .pfi-meta { font-size:12px; color:#9ca3af; }

  /* Upload panel */
  .upload-panel { padding:18px; display:flex; flex-direction:column; gap:10px; }
  .upload-panel h4 { font-size:14px; font-weight:700; color:#1a1a2e; }
  .upload-hint { font-size:12px; color:#6b7280; }

  .file-row { display:flex; gap:8px; }
  .file-drop {
    flex:1; border:2px dashed #e5e7eb; border-radius:8px;
    padding:12px 8px; text-align:center; cursor:pointer;
    position:relative; font-size:12px; color:#9ca3af;
    display:flex; flex-direction:column; align-items:center; gap:4px;
    transition:border-color 150ms;
  }
  .file-drop:hover { border-color:#6366f1; color:#6366f1; }
  .file-drop.has-file { border-color:#10b981; color:#065f46; background:#f0fdf4; }
  .file-drop input[type=file] {
    position:absolute; inset:0; opacity:0; cursor:pointer; width:100%; height:100%;
  }

  .csv-hint { font-size:11.5px; color:#9ca3af; }
  .csv-hint summary { cursor:pointer; }
  .csv-example { margin-top:6px; background:#f9fafb; border-radius:6px; padding:10px; }
  .csv-example pre { font-size:10px; white-space:pre-wrap; color:#374151; margin:4px 0 8px; }
  .csv-example strong { font-size:11px; color:#6b7280; }

  /* ── Rows list ──────────────────────────────────────────────────────────────── */
  .rows-list { display:flex; flex-direction:column; gap:12px; }

  .row-card { padding:14px; }
  .rc-header { display:flex; justify-content:space-between; align-items:center; margin-bottom:8px; }
  .rc-label { font-weight:600; font-size:14px; }
  .rc-actions { display:flex; gap:6px; }
  .rc-meta { font-size:11.5px; color:#9ca3af; margin-top:4px; }

  .sparkline {
    display:flex; align-items:flex-end; gap:1px;
    height:34px; background:#f9fafb; border-radius:4px; padding:2px 4px;
    overflow:hidden;
  }
  .spark-bar { flex:1; background:#6366f1; border-radius:1px; min-width:2px; transition:height 200ms; }

  /* ── Alerts ─────────────────────────────────────────────────────────────────── */
  .alert-success {
    background:#d1fae5; color:#065f46;
    padding:10px 14px; border-radius:8px; font-size:13px;
    border-left:3px solid #10b981;
  }

  /* ── Modal ──────────────────────────────────────────────────────────────────── */
  .modal-backdrop {
    position:fixed; inset:0; background:rgba(0,0,0,.45); z-index:100;
    display:flex; align-items:center; justify-content:center;
  }
  .modal {
    background:#fff; border-radius:16px; width:640px; max-width:95vw;
    max-height:85vh; overflow-y:auto;
    box-shadow:0 20px 60px rgba(0,0,0,.2);
  }
  .modal-header {
    display:flex; justify-content:space-between; align-items:center;
    padding:18px 22px 14px; border-bottom:1px solid #e5e7eb;
    font-size:15px;
  }
  .modal-body { padding:20px 22px; }
  .modal-body h4 { font-size:13px; font-weight:600; color:#6b7280; margin-bottom:8px; }

  .out-chart {
    display:flex; align-items:flex-end; gap:2px;
    height:84px; background:#f9fafb; border-radius:6px; padding:4px 6px;
    margin-bottom:4px;
  }
  .out-col { flex:1; display:flex; align-items:flex-end; }
  .out-bar { width:100%; background:#6366f1; border-radius:1px 1px 0 0; }

  .sched-bars {
    display:flex; align-items:flex-end; gap:4px;
    height:134px; background:#f9fafb; border-radius:6px;
    padding:8px 10px 24px; margin-bottom:4px;
  }
  .sched-col { flex:1; display:flex; flex-direction:column; align-items:center; }
  .sched-bar { width:100%; background:#6366f1; border-radius:2px 2px 0 0; }
  .sched-lbl { font-size:9px; color:#9ca3af; margin-top:4px; text-align:center; }

  .preview-range { font-size:11.5px; color:#9ca3af; }
</style>
