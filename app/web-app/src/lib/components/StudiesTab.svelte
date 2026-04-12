<script lang="ts">
  import { onMount } from 'svelte';
  import { studies, linkers } from '../api/client';
  import type { Study, StudyDetail, Linker } from '../api/client';
  import { Plus, Trash2, BookOpen, Link, X } from '@lucide/svelte';

  type View = 'list' | 'detail' | 'create';
  let view     = $state<View>('list');
  let studyList = $state<Study[]>([]);
  let allLinkers = $state<Linker[]>([]);
  let selected  = $state<StudyDetail | null>(null);
  let loading   = $state(true);
  let error     = $state('');

  // ── Create ──────────────────────────────────────────────────────────────────
  let cName        = $state('');
  let cDescription = $state('');
  let cSaving      = $state(false);
  let cError       = $state('');

  // ── Notes editing ───────────────────────────────────────────────────────────
  let editingNotes = $state(false);
  let notesBuffer  = $state('');
  let notesSaving  = $state(false);

  // ── Add linker ──────────────────────────────────────────────────────────────
  let addLinkerId = $state('');
  let addLabel    = $state('');
  let addingLinker = $state(false);

  // Available linkers not yet in study
  let availableLinkers = $derived(
    allLinkers.filter(l => !selected?.linkers.some(sl => sl.linker_id === l.id))
  );

  async function load() {
    loading = true; error = '';
    try {
      [studyList, allLinkers] = await Promise.all([studies.list(), linkers.list()]);
    } catch (e: any) { error = e.message; }
    finally { loading = false; }
  }

  async function openDetail(id: string) {
    selected = null;
    view = 'detail';
    try {
      selected = await studies.get(id);
      notesBuffer = selected.notes ?? '';
    } catch (e: any) { error = e.message; }
  }

  async function createStudy() {
    cError = '';
    if (!cName.trim()) { cError = 'Name required'; return; }
    cSaving = true;
    try {
      const s = await studies.create({ name: cName.trim(), description: cDescription || undefined });
      cName = ''; cDescription = '';
      await load();
      openDetail(s.id);
    } catch (e: any) { cError = e.message; }
    finally { cSaving = false; }
  }

  async function deleteStudy(id: string, name: string) {
    if (!confirm(`Delete "${name}"?`)) return;
    await studies.delete(id);
    if (selected?.id === id) { selected = null; view = 'list'; }
    await load();
  }

  async function saveNotes() {
    if (!selected) return;
    notesSaving = true;
    try {
      selected = await studies.update(selected.id, { notes: notesBuffer });
      editingNotes = false;
    } catch { /**/ }
    finally { notesSaving = false; }
  }

  async function addLinker() {
    if (!selected || !addLinkerId) return;
    addingLinker = true;
    try {
      selected = await studies.addLinker(selected.id, { linker_id: addLinkerId, label: addLabel || undefined });
      addLinkerId = ''; addLabel = '';
    } catch (e: any) { error = e.message; }
    finally { addingLinker = false; }
  }

  async function removeLinker(linkerId: string) {
    if (!selected) return;
    selected = await studies.removeLinker(selected.id, linkerId);
  }

  onMount(load);
</script>

<div class="tab-content">
  <div class="tab-header">
    <h2>Studies</h2>
    <div style="display:flex;gap:8px">
      {#if view !== 'list'}
        <button class="btn-sm" onclick={() => { view = 'list'; selected = null; }}>← Back</button>
      {/if}
      {#if view === 'list'}
        <button class="btn-primary" onclick={() => { view = 'create'; cError = ''; }}>
          <Plus size={14}/> Nouvelle study
        </button>
      {/if}
    </div>
  </div>

  {#if error}<div class="alert-error">{error}</div>{/if}

  <!-- ── List ──────────────────────────────────────────────────────────────── -->
  {#if view === 'list'}
    {#if loading}
<p class="loading">Loading…</p>
      {:else if studyList.length === 0}
        <div class="empty-state">
          <p>No study defined.</p>
          <p>A study groups several linkers to compare FTP analyses.</p>
          <button class="btn-primary" onclick={() => view = 'create'}><Plus size={14}/> Create</button>
      </div>
    {:else}
      <div class="study-grid">
        {#each studyList as s}
          <div class="study-card" onclick={() => openDetail(s.id)}>
            <div class="sc-top">
              <BookOpen size={14}/>
              <span class="sc-name">{s.name}</span>
            </div>
            {#if s.description}<div class="sc-desc">{s.description}</div>{/if}
            <div class="sc-meta">
              <span class="tag"><Link size={11}/> {s.linker_count} linker{s.linker_count !== 1 ? 's' : ''}</span>
              <span class="sc-date">{s.created_at.slice(0,10)}</span>
            </div>
            <div class="sc-actions" onclick={e => e.stopPropagation()}>
              <button class="btn-sm btn-danger" onclick={() => deleteStudy(s.id, s.name)}>
                <Trash2 size={12}/>
              </button>
            </div>
          </div>
        {/each}
      </div>
    {/if}

  <!-- ── Create ─────────────────────────────────────────────────────────────── -->
  {:else if view === 'create'}
    <div class="card form-card">
      <h3 class="form-title">New study</h3>
      <div class="form-grid">
        <label>
          Name
          <input bind:value={cName} placeholder="Ex: ESTR sensitivity analysis"/>
        </label>
        <label>
          Description (optional)
          <input bind:value={cDescription}/>
        </label>
      </div>
      {#if cError}<div class="alert-error">{cError}</div>{/if}
      <div class="form-footer">
        <button class="btn-sm" onclick={() => view = 'list'}>Cancel</button>
        <button class="btn-primary" onclick={createStudy} disabled={cSaving}>
          {cSaving ? 'Creating…' : 'Create'}
        </button>
      </div>
    </div>

  <!-- ── Detail ─────────────────────────────────────────────────────────────── -->
  {:else if view === 'detail'}
    {#if !selected}
      <p class="loading">Chargement…</p>
    {:else}
      <div class="detail-layout">

        <!-- LEFT: info + linkers -->
        <div class="detail-left">
          <div class="card info-panel">
            <div class="ip-top">
              <BookOpen size={15}/>
              <strong>{selected.name}</strong>
            </div>
            {#if selected.description}
              <p class="ip-desc">{selected.description}</p>
            {/if}
            <p class="ip-meta">{selected.linkers.length} linker{selected.linkers.length !== 1 ? 's' : ''} · created {selected.created_at.slice(0,10)}</p>
          </div>

          <!-- Add linker -->
          <div class="card add-panel">
            <h4>Add a linker</h4>
            <select bind:value={addLinkerId}>
              <option value="">-- Choose --</option>
              {#each availableLinkers as l}
                <option value={l.id}>{l.name} ({l.portfolio_name ?? '?'} × {l.cube_name ?? '?'})</option>
              {/each}
            </select>
            {#if availableLinkers.length === 0}
              <span class="hint-warn">All linkers are already in this study.</span>
            {/if}
            <label>
              Alias in this study (optional)
              <input bind:value={addLabel} placeholder="Ex: Base scenario"/>
            </label>
            <button
              class="btn-primary"
              onclick={addLinker}
              disabled={!addLinkerId || addingLinker}
            >
              {addingLinker ? 'Adding…' : 'Add'}
            </button>
          </div>
        </div>

        <!-- RIGHT: linkers list + notes -->
        <div class="detail-right">

          <!-- Linkers -->
          <div class="linkers-section">
            <h3 class="section-title">Linkers</h3>
            {#if selected.linkers.length === 0}
              <div class="empty-state" style="padding:24px">
                <p>No linker — add one from the left panel.</p>
              </div>
            {:else}
              <div class="linker-list">
                {#each selected.linkers as sl, i}
                  <div class="sl-card">
                    <div class="sl-pos">{i + 1}</div>
                    <div class="sl-body">
                      <div class="sl-name">{sl.label ?? sl.linker_name ?? sl.linker_id}</div>
                      <div class="sl-meta">
                        <span>📁 {sl.portfolio_name ?? '—'}</span>
                        <span>📦 {sl.cube_name ?? '—'}</span>
                        <span>📅 {sl.start_date ?? '—'}</span>
                      </div>
                    </div>
                    <button class="sl-remove" onclick={() => removeLinker(sl.linker_id)}>
                      <X size={13}/>
                    </button>
                  </div>
                {/each}
              </div>
            {/if}
          </div>

          <!-- Notes -->
          <div class="card notes-panel">
            <div class="notes-header">
              <h3 class="section-title" style="margin:0">Notes</h3>
              {#if !editingNotes}
                <button class="btn-sm" onclick={() => { editingNotes = true; notesBuffer = selected?.notes ?? ''; }}>
                  Edit
                </button>
              {:else}
                <div style="display:flex;gap:6px">
                  <button class="btn-sm" onclick={() => editingNotes = false}>Cancel</button>
                  <button class="btn-sm btn-success" onclick={saveNotes} disabled={notesSaving}>
                    {notesSaving ? 'Saving…' : 'Save'}
                  </button>
                </div>
              {/if}
            </div>

            {#if editingNotes}
              <textarea
                bind:value={notesBuffer}
                rows={10}
                placeholder="Context, objective, observations…"
                class="notes-editor"
              ></textarea>
            {:else if selected.notes}
              <pre class="notes-display">{selected.notes}</pre>
            {:else}
              <p class="notes-empty">No note. Click Edit to add one.</p>
            {/if}
          </div>

        </div>
      </div>
    {/if}
  {/if}
</div>

<style>
  /* Grid */
  .study-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(260px,1fr));
    gap: 14px;
  }
  .study-card {
    background: #fff;
    border: 1px solid #e5e7eb;
    border-radius: 12px;
    padding: 16px;
    cursor: pointer;
    position: relative;
    transition: border-color 150ms, box-shadow 150ms;
  }
  .study-card:hover { border-color: #6366f1; box-shadow: 0 0 0 3px rgba(99,102,241,.08); }
  .sc-top  { display:flex; align-items:center; gap:8px; margin-bottom:6px; }
  .sc-name { font-weight:700; font-size:14px; color:#1a1a2e; }
  .sc-desc { font-size:12px; color:#6b7280; margin-bottom:8px; }
  .sc-meta { display:flex; align-items:center; justify-content:space-between; gap:8px; }
  .sc-date { font-size:12px; color:#9ca3af; }
  .sc-actions { position:absolute; bottom:12px; right:12px; opacity:0; transition:opacity 150ms; }
  .study-card:hover .sc-actions { opacity:1; }

  /* Create form */
  .form-card  { padding:24px; max-width:560px; }
  .form-title { font-size:16px; font-weight:700; margin-bottom:18px; }
  .form-grid  { display:grid; grid-template-columns:1fr 1fr; gap:12px; margin-bottom:16px; }
  .form-footer{ display:flex; justify-content:flex-end; gap:10px; padding-top:16px; border-top:1px solid #e5e7eb; }

  /* Detail layout */
  .detail-layout { display:grid; grid-template-columns:300px 1fr; gap:20px; align-items:start; }
  .detail-left   { display:flex; flex-direction:column; gap:14px; }

  .info-panel { padding:16px; }
  .ip-top  { display:flex; align-items:center; gap:8px; font-size:15px; margin-bottom:6px; }
  .ip-desc { font-size:12.5px; color:#6b7280; margin-bottom:6px; }
  .ip-meta { font-size:12px; color:#9ca3af; }

  .add-panel { padding:16px; display:flex; flex-direction:column; gap:10px; }
  .add-panel h4 { font-size:14px; font-weight:700; color:#1a1a2e; }

  /* Right column */
  .detail-right   { display:flex; flex-direction:column; gap:16px; }
  .section-title  { font-size:14px; font-weight:700; color:#1a1a2e; margin-bottom:10px; }
  .linkers-section { }

  .linker-list { display:flex; flex-direction:column; gap:8px; }
  .sl-card {
    background:#fff;
    border:1px solid #e5e7eb;
    border-radius:10px;
    padding:12px 14px;
    display:flex;
    align-items:center;
    gap:12px;
  }
  .sl-pos {
    width:26px; height:26px;
    background:#6366f1; color:#fff;
    border-radius:50%;
    font-size:12px; font-weight:700;
    display:flex; align-items:center; justify-content:center;
    flex-shrink:0;
  }
  .sl-body  { flex:1; min-width:0; }
  .sl-name  { font-weight:600; font-size:13.5px; color:#1a1a2e; margin-bottom:4px; }
  .sl-meta  { display:flex; gap:12px; font-size:12px; color:#6b7280; flex-wrap:wrap; }
  .sl-remove {
    background:none; border:none; cursor:pointer;
    color:#9ca3af; padding:4px; border-radius:4px;
    display:flex; align-items:center;
    transition:color 150ms, background 150ms;
  }
  .sl-remove:hover { color:#ef4444; background:#fee2e2; }

  /* Notes */
  .notes-panel  { padding:18px; }
  .notes-header { display:flex; justify-content:space-between; align-items:center; margin-bottom:12px; }
  .notes-editor {
    width:100%; border:1px solid #e5e7eb; border-radius:8px;
    padding:10px; font-size:13px; font-family:inherit;
    resize:vertical; color:#1a1a2e;
  }
  .notes-display {
    font-size:13px; color:#374151; white-space:pre-wrap;
    font-family:inherit; line-height:1.6;
  }
  .notes-empty { font-size:13px; color:#9ca3af; font-style:italic; }

  .hint-warn { font-size:11.5px; color:#d97706; display:block; }
</style>
