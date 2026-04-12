<script lang="ts">
  import { studies, studyUnits } from '$lib/api/client';
  import type {
    StudySummary, StudyDetail, StudyUnitRef, StudyUnitSummary,
  } from '$lib/api/client';
  import {
    Plus, Trash2, Pencil, X, BookOpen, CheckCircle, AlertCircle, Minus,
  } from '@lucide/svelte';

  // ── État global ───────────────────────────────────────────────────────────────

  let studyList  = $state<StudySummary[]>([]);
  let loading    = $state(true);
  let error      = $state<string | null>(null);

  let selectedId    = $state<string | null>(null);
  let detail        = $state<StudyDetail | null>(null);
  let detailLoading = $state(false);

  // Toutes les study units disponibles (pour le sélecteur d'ajout)
  let allUnits = $state<StudyUnitSummary[]>([]);

  // ── Formulaire étude ─────────────────────────────────────────────────────────

  let showStudyForm  = $state(false);
  let editingId      = $state<string | null>(null);
  let fName          = $state('');
  let fDesc          = $state('');
  let fStatus        = $state('draft');
  let studyFormError = $state<string | null>(null);
  let studySaving    = $state(false);

  // ── Ajout de study unit ───────────────────────────────────────────────────────

  let showAddUnit   = $state(false);
  let addUnitId     = $state('');
  let addUnitLabel  = $state('');
  let addUnitError  = $state<string | null>(null);
  let addUnitSaving = $state(false);

  // Study units déjà présentes dans l'étude courante
  let alreadyInStudy = $derived(new Set(detail?.units.map(u => u.study_unit_id) ?? []));

  // Study units disponibles (pas encore dans l'étude)
  let availableUnits = $derived(allUnits.filter(u => !alreadyInStudy.has(u.id)));

  // ── Chargement ────────────────────────────────────────────────────────────────

  async function loadAll() {
    loading = true;
    error   = null;
    try {
      const [sl, ul] = await Promise.all([studies.list(), studyUnits.list()]);
      studyList = sl;
      allUnits  = ul;
    } catch (e: any) {
      error = e.message;
    } finally {
      loading = false;
    }
  }

  async function selectStudy(id: string) {
    if (selectedId === id) return;
    selectedId    = id;
    detail        = null;
    showAddUnit   = false;
    detailLoading = true;
    try {
      detail = await studies.get(id);
    } catch (e: any) {
      error = e.message;
    } finally {
      detailLoading = false;
    }
  }

  loadAll();

  // ── CRUD études ───────────────────────────────────────────────────────────────

  function openCreate() {
    editingId      = null;
    fName          = '';
    fDesc          = '';
    fStatus        = 'draft';
    studyFormError = null;
    showStudyForm  = true;
  }

  function openEdit(s: StudySummary) {
    editingId      = s.id;
    fName          = s.name;
    fDesc          = s.description ?? '';
    fStatus        = s.status;
    studyFormError = null;
    showStudyForm  = true;
  }

  async function saveStudy() {
    if (!fName.trim()) { studyFormError = 'Le nom est requis'; return; }
    studySaving    = true;
    studyFormError = null;
    try {
      if (editingId) {
        await studies.update(editingId, {
          name:        fName.trim(),
          description: fDesc || undefined,
          status:      fStatus,
        });
      } else {
        await studies.create({
          name:        fName.trim(),
          description: fDesc || undefined,
          status:      fStatus,
        });
      }
      showStudyForm = false;
      await loadAll();
      if (editingId && selectedId === editingId) await selectStudy(editingId);
    } catch (e: any) {
      studyFormError = e.message;
    } finally {
      studySaving = false;
    }
  }

  async function deleteStudy(id: string) {
    if (!confirm('Supprimer cette étude ?')) return;
    try {
      await studies.delete(id);
      if (selectedId === id) { selectedId = null; detail = null; }
      await loadAll();
    } catch (e: any) { alert(e.message); }
  }

  // ── Statut rapide ─────────────────────────────────────────────────────────────

  async function setStatus(status: string) {
    if (!selectedId) return;
    try {
      detail = await studies.update(selectedId, { status });
      studyList = await studies.list();
    } catch (e: any) { alert(e.message); }
  }

  // ── Ajout / retrait de study units ───────────────────────────────────────────

  function openAddUnit() {
    addUnitId    = '';
    addUnitLabel = '';
    addUnitError = null;
    showAddUnit  = true;
  }

  async function confirmAddUnit() {
    if (!addUnitId) { addUnitError = 'Sélectionner une study unit'; return; }
    addUnitSaving = true;
    addUnitError  = null;
    try {
      detail = await studies.addUnit(selectedId!, {
        study_unit_id: addUnitId,
        label: addUnitLabel || undefined,
      });
      showAddUnit   = false;
      studyList     = await studies.list();
    } catch (e: any) {
      addUnitError = e.message;
    } finally {
      addUnitSaving = false;
    }
  }

  async function removeUnit(unitId: string) {
    if (!confirm('Retirer cette study unit de l\'étude ?')) return;
    try {
      await studies.removeUnit(selectedId!, unitId);
      detail    = await studies.get(selectedId!);
      studyList = await studies.list();
    } catch (e: any) { alert(e.message); }
  }

  // ── Helpers ───────────────────────────────────────────────────────────────────

  function statusLabel(s: string) {
    return s === 'draft' ? 'Brouillon' : s === 'ready' ? 'Prête' : 'Archivée';
  }
  function statusClass(s: string) {
    return s === 'ready' ? 'badge-active' : s === 'archived' ? 'badge-archived' : 'badge-draft';
  }

  function readyToExecute(d: StudyDetail) {
    return d.units.length > 0 && d.units.every(u => u.is_valid);
  }
</script>

<!-- ── Layout ─────────────────────────────────────────────────────────────── -->
<div class="page">

  <!-- Panneau gauche -->
  <aside class="left-panel card">
    <div class="panel-header">
      <h2>Studies</h2>
      <button class="btn-primary" onclick={openCreate}><Plus size={13} /> Nouvelle</button>
    </div>

    {#if loading}
      <p class="loading" style="padding:16px">Chargement…</p>
    {:else if error}
      <div class="alert-error" style="margin:12px">{error}</div>
    {:else if studyList.length === 0}
      <div class="empty-state" style="margin:16px;padding:32px 16px">
        <p>Aucune étude</p>
        <p>Créez une étude pour regrouper des study units.</p>
      </div>
    {:else}
      <div class="study-list">
        {#each studyList as s}
          <!-- svelte-ignore a11y_click_events_have_key_events a11y_no_static_element_interactions -->
          <div
            class="study-item"
            class:study-item--active={selectedId === s.id}
            onclick={() => selectStudy(s.id)}
          >
            <div class="study-row1">
              <span class="study-name">{s.name}</span>
              <span class="badge {statusClass(s.status)}">{statusLabel(s.status)}</span>
            </div>
            <div class="study-row2">
              {#if s.unit_count > 0}
                <span class="study-meta">
                  {s.valid_unit_count}/{s.unit_count} unité(s) valide(s)
                </span>
              {:else}
                <span class="study-meta empty-meta">Aucune study unit</span>
              {/if}
            </div>
            <!-- svelte-ignore a11y_click_events_have_key_events a11y_no_static_element_interactions -->
            <div class="study-actions" onclick={e => e.stopPropagation()}>
              <button class="btn-sm" onclick={() => openEdit(s)} title="Modifier"><Pencil size={11} /></button>
              <button class="btn-sm btn-danger" onclick={() => deleteStudy(s.id)} title="Supprimer"><Trash2 size={11} /></button>
            </div>
          </div>
        {/each}
      </div>
    {/if}
  </aside>

  <!-- Panneau droit -->
  <main class="right-panel">
    {#if !selectedId}
      <div class="empty-state" style="margin:40px auto;max-width:400px">
        <BookOpen size={32} style="margin:0 auto 12px;opacity:.3" />
        <p>Sélectionnez une étude pour voir son contenu</p>
      </div>

    {:else if detailLoading}
      <p class="loading" style="padding:32px">Chargement…</p>

    {:else if detail}
      <!-- En-tête -->
      <div class="detail-header">
        <div class="detail-title-area">
          <h2>{detail.name}</h2>
          {#if detail.description}
            <p class="detail-desc">{detail.description}</p>
          {/if}
        </div>
        <div class="detail-controls">
          <!-- Sélecteur de statut -->
          <div class="status-control">
            <span class="status-label-sm">Statut</span>
            <select
              class="status-select"
              class:status-select--ready={detail.status === 'ready'}
              class:status-select--archived={detail.status === 'archived'}
              value={detail.status}
              onchange={e => setStatus((e.target as HTMLSelectElement).value)}
            >
              <option value="draft">Brouillon</option>
              <option value="ready">Prête</option>
              <option value="archived">Archivée</option>
            </select>
          </div>
        </div>
      </div>

      <!-- Alerte "prête pour exécution" -->
      {#if readyToExecute(detail) && detail.status !== 'ready'}
        <div class="ready-hint">
          <CheckCircle size={14} />
          Toutes les study units sont valides — vous pouvez passer le statut à « Prête ».
        </div>
      {:else if detail.status === 'ready'}
        <div class="ready-banner">
          <CheckCircle size={14} />
          Étude prête pour l'exécution ({detail.units.length} study unit(s))
        </div>
      {/if}

      <!-- Tableau des study units -->
      <div class="units-section">
        <div class="units-header">
          <h3>Study units ({detail.units.length})</h3>
          <button class="btn-sm btn-success" onclick={openAddUnit}>
            <Plus size={11} /> Ajouter
          </button>
        </div>

        {#if detail.units.length === 0}
          <div class="empty-state" style="padding:32px;margin:0">
            <p>Aucune study unit dans cette étude</p>
            <p>Ajoutez des study units pour composer votre simulation.</p>
          </div>
        {:else}
          <table class="units-table">
            <thead>
              <tr>
                <th>Study unit</th>
                <th>Hypercube</th>
                <th>Portfolio</th>
                <th>Assignments</th>
                <th>Validation</th>
                <th></th>
              </tr>
            </thead>
            <tbody>
              {#each detail.units as u}
                <tr class:row-invalid={!u.is_valid}>
                  <td class="cell-name">
                    <span>{u.name}</span>
                    {#if u.label}
                      <span class="unit-label-badge">{u.label}</span>
                    {/if}
                  </td>
                  <td class="cell-meta">{u.hypercube_name}</td>
                  <td class="cell-meta">{u.portfolio_name}</td>
                  <td class="cell-center">{u.assignment_count}</td>
                  <td class="cell-center">
                    {#if u.is_valid}
                      <span class="valid-chip">
                        <CheckCircle size={13} /> Valide
                      </span>
                    {:else}
                      <span class="invalid-chip">
                        <AlertCircle size={13} /> Non validée
                      </span>
                    {/if}
                  </td>
                  <td class="cell-action">
                    <button
                      class="btn-sm btn-danger"
                      onclick={() => removeUnit(u.study_unit_id)}
                      title="Retirer de l'étude"
                    >
                      <Minus size={11} />
                    </button>
                  </td>
                </tr>
              {/each}
            </tbody>
          </table>
        {/if}

        <!-- Panneau d'ajout inline -->
        {#if showAddUnit}
          <div class="add-unit-panel card">
            <div class="add-unit-header">
              <h4>Ajouter une study unit</h4>
              <button onclick={() => showAddUnit = false}><X size={14} /></button>
            </div>
            {#if addUnitError}
              <div class="alert-error" style="margin:0 0 10px">{addUnitError}</div>
            {/if}
            {#if availableUnits.length === 0}
              <p class="loading">Toutes les study units sont déjà dans cette étude.</p>
            {:else}
              <div class="add-unit-form">
                <label>Study unit
                  <select bind:value={addUnitId}>
                    <option value="">— Sélectionner —</option>
                    {#each availableUnits as u}
                      <option value={u.id}>
                        {u.name}
                        {u.is_valid ? '✓' : '⚠'}
                        — HC: {u.hypercube_name} / P: {u.portfolio_name}
                      </option>
                    {/each}
                  </select>
                </label>
                <label>Label (optionnel)
                  <input bind:value={addUnitLabel} placeholder="Ex. Scénario central" />
                </label>
                <div class="add-unit-actions">
                  <button class="btn-sm" onclick={() => showAddUnit = false}>Annuler</button>
                  <button class="btn-primary" onclick={confirmAddUnit} disabled={addUnitSaving}>
                    {addUnitSaving ? 'Ajout…' : 'Ajouter'}
                  </button>
                </div>
              </div>
            {/if}
          </div>
        {/if}
      </div>
    {/if}
  </main>
</div>

<!-- ── Modal : créer / modifier étude ───────────────────────────────────────── -->
{#if showStudyForm}
  <!-- svelte-ignore a11y_click_events_have_key_events a11y_no_static_element_interactions -->
  <div class="overlay" onclick={() => showStudyForm = false}>
    <!-- svelte-ignore a11y_click_events_have_key_events a11y_no_static_element_interactions -->
    <div class="modal" onclick={e => e.stopPropagation()}>
      <div class="modal-hd">
        <h3>{editingId ? 'Modifier l\'étude' : 'Nouvelle étude'}</h3>
        <button onclick={() => showStudyForm = false}><X size={16} /></button>
      </div>
      <div class="modal-bd">
        {#if studyFormError}
          <div class="alert-error">{studyFormError}</div>
        {/if}
        <label>Nom <input bind:value={fName} placeholder="Ex. Simulation Q4 2024" /></label>
        <label>Description <textarea bind:value={fDesc} rows={2}></textarea></label>
        <label>Statut
          <select bind:value={fStatus}>
            <option value="draft">Brouillon</option>
            <option value="ready">Prête</option>
            <option value="archived">Archivée</option>
          </select>
        </label>
      </div>
      <div class="modal-ft">
        <button class="btn-sm" onclick={() => showStudyForm = false}>Annuler</button>
        <button class="btn-primary" onclick={saveStudy} disabled={studySaving}>
          {studySaving ? 'Enregistrement…' : 'Enregistrer'}
        </button>
      </div>
    </div>
  </div>
{/if}

<style>
  /* ── Layout ── */
  .page { display: flex; height: 100vh; overflow: hidden; }

  .left-panel {
    width: 280px;
    flex-shrink: 0;
    display: flex;
    flex-direction: column;
    border-radius: 0;
    border-right: 1px solid #e5e7eb;
    overflow-y: auto;
  }

  .right-panel { flex: 1; overflow-y: auto; background: #f4f5f9; }

  /* ── Panel header ── */
  .panel-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 20px 16px 14px;
    border-bottom: 1px solid #f1f1f5;
    flex-shrink: 0;
  }
  .panel-header h2 { font-size: 15px; font-weight: 700; color: #1a1a2e; }

  /* ── Study list ── */
  .study-list { padding: 8px; }

  .study-item {
    border: 1px solid transparent;
    border-radius: 8px;
    padding: 10px 12px;
    margin-bottom: 4px;
    cursor: pointer;
    transition: background 100ms, border-color 100ms;
  }
  .study-item:hover { background: #f4f5f9; }
  .study-item--active { background: #eef2ff; border-color: #c7d2fe; }

  .study-row1 {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 3px;
  }
  .study-name { font-size: 13px; font-weight: 600; color: #1a1a2e; }
  .study-row2 { display: flex; align-items: center; }
  .study-meta { font-size: 11.5px; color: #6b7280; }
  .empty-meta { color: #d1d5db; font-style: italic; }
  .study-actions { display: flex; gap: 4px; margin-top: 6px; justify-content: flex-end; }

  /* ── Detail header ── */
  .detail-header {
    display: flex;
    justify-content: space-between;
    align-items: flex-start;
    padding: 24px 28px 16px;
    background: #fff;
    border-bottom: 1px solid #e5e7eb;
  }
  .detail-header h2 { font-size: 18px; font-weight: 700; color: #1a1a2e; }
  .detail-desc { font-size: 13px; color: #6b7280; margin-top: 3px; }

  .detail-controls { display: flex; align-items: center; gap: 12px; flex-shrink: 0; }
  .status-control { display: flex; flex-direction: column; gap: 3px; }
  .status-label-sm { font-size: 11px; font-weight: 600; color: #9ca3af; text-transform: uppercase; }
  .status-select {
    border: 1px solid #e5e7eb;
    border-radius: 6px;
    padding: 5px 10px;
    font-size: 12.5px;
    font-weight: 600;
    color: #92400e;
    background: #fef3c7;
    cursor: pointer;
    width: auto;
  }
  .status-select--ready   { background: #d1fae5; color: #065f46; border-color: #a7f3d0; }
  .status-select--archived { background: #f3f4f6; color: #6b7280; border-color: #e5e7eb; }

  /* ── Alerts ── */
  .ready-hint {
    display: flex;
    align-items: center;
    gap: 6px;
    margin: 0 28px;
    padding: 8px 14px;
    background: #fffbeb;
    border: 1px solid #fde68a;
    border-radius: 8px;
    font-size: 12.5px;
    color: #92400e;
    margin-top: 12px;
  }
  .ready-banner {
    display: flex;
    align-items: center;
    gap: 6px;
    margin: 12px 28px 0;
    padding: 8px 14px;
    background: #d1fae5;
    border: 1px solid #a7f3d0;
    border-radius: 8px;
    font-size: 12.5px;
    color: #065f46;
    font-weight: 600;
  }

  /* ── Units section ── */
  .units-section { padding: 20px 28px 32px; }
  .units-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 14px;
  }
  .units-header h3 { font-size: 14px; font-weight: 700; color: #374151; }

  /* ── Table ── */
  .units-table {
    width: 100%;
    border-collapse: collapse;
    background: #fff;
    border-radius: 10px;
    overflow: hidden;
    box-shadow: 0 1px 3px rgba(0,0,0,.06);
    font-size: 13px;
  }
  .units-table th {
    text-align: left;
    padding: 10px 14px;
    font-size: 11.5px;
    font-weight: 600;
    color: #6b7280;
    text-transform: uppercase;
    letter-spacing: .04em;
    background: #f9fafb;
    border-bottom: 1px solid #f1f1f5;
  }
  .units-table td { padding: 11px 14px; border-bottom: 1px solid #f9fafb; vertical-align: middle; }
  .units-table tr:last-child td { border-bottom: none; }
  .units-table tr.row-invalid { background: #fffbeb; }

  .cell-name { font-weight: 600; color: #1a1a2e; }
  .cell-name span { display: block; }
  .unit-label-badge {
    display: inline-block;
    background: #ede9fe;
    color: #5b21b6;
    border-radius: 4px;
    padding: 1px 6px;
    font-size: 11px;
    font-weight: 500;
    margin-top: 2px;
  }
  .cell-meta   { color: #6b7280; font-size: 12.5px; }
  .cell-center { text-align: center; }
  .cell-action { text-align: right; }

  .valid-chip {
    display: inline-flex;
    align-items: center;
    gap: 4px;
    background: #d1fae5;
    color: #065f46;
    border-radius: 20px;
    padding: 2px 8px;
    font-size: 11.5px;
    font-weight: 600;
  }
  .invalid-chip {
    display: inline-flex;
    align-items: center;
    gap: 4px;
    background: #fef3c7;
    color: #92400e;
    border-radius: 20px;
    padding: 2px 8px;
    font-size: 11.5px;
    font-weight: 600;
  }

  /* ── Add unit panel ── */
  .add-unit-panel {
    margin-top: 14px;
    padding: 14px 16px;
    border-radius: 10px;
    border: 1px dashed #c7d2fe;
    background: #f8f8ff;
  }
  .add-unit-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 12px;
  }
  .add-unit-header h4 { font-size: 13px; font-weight: 600; color: #374151; }
  .add-unit-header button { background: none; border: none; cursor: pointer; color: #6b7280; }
  .add-unit-form { display: flex; flex-direction: column; gap: 10px; }
  .add-unit-actions {
    display: flex;
    justify-content: flex-end;
    gap: 8px;
    margin-top: 4px;
  }

  /* ── Modal ── */
  .overlay {
    position: fixed; inset: 0;
    background: rgba(0,0,0,.45);
    display: flex; align-items: center; justify-content: center;
    z-index: 100;
  }
  .modal {
    background: #fff;
    border-radius: 14px;
    width: 460px;
    max-width: 95vw;
    max-height: 85vh;
    display: flex;
    flex-direction: column;
    box-shadow: 0 20px 60px rgba(0,0,0,.2);
  }
  .modal-hd {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 18px 20px 14px;
    border-bottom: 1px solid #f1f1f5;
  }
  .modal-hd h3 { font-size: 15px; font-weight: 700; }
  .modal-hd button { background: none; border: none; cursor: pointer; color: #6b7280; }
  .modal-bd {
    padding: 16px 20px;
    overflow-y: auto;
    display: flex;
    flex-direction: column;
    gap: 12px;
  }
  .modal-ft {
    display: flex;
    justify-content: flex-end;
    gap: 8px;
    padding: 12px 20px;
    border-top: 1px solid #f1f1f5;
  }
</style>
