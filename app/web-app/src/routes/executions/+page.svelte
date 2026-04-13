<script lang="ts">
  import { executions, studies } from '$lib/api/client';
  import type { ExecutionSummary, StudySummary } from '$lib/api/client';
  import { Play, Trash2, X, LayoutDashboard } from '@lucide/svelte';

  let execList    = $state<ExecutionSummary[]>([]);
  let allStudies  = $state<StudySummary[]>([]);
  let loading     = $state(true);
  let error       = $state<string | null>(null);

  let showModal   = $state(false);
  let studyId     = $state('');
  let label       = $state('');
  let modalError  = $state<string | null>(null);
  let launching   = $state(false);

  async function loadAll() {
    loading = true; error = null;
    try {
      const [el, sl] = await Promise.all([executions.list(), studies.list()]);
      execList   = el;
      allStudies = sl;
    } catch (e: any) { error = e.message; }
    finally { loading = false; }
  }

  async function openModal() {
    studyId    = allStudies.find(s => s.status === 'ready')?.id ?? allStudies[0]?.id ?? '';
    label      = '';
    modalError = null;
    showModal  = true;
  }

  async function launch() {
    if (!studyId) { modalError = 'Sélectionner une étude'; return; }
    launching = true; modalError = null;
    try {
      await executions.create({ study_id: studyId, label: label || undefined });
      showModal = false;
      await loadAll();
    } catch (e: any) { modalError = e.message; }
    finally { launching = false; }
  }

  async function del(id: string) {
    if (!confirm('Supprimer cette exécution ?')) return;
    try { await executions.delete(id); await loadAll(); }
    catch (e: any) { alert(e.message); }
  }

  loadAll();

  function statusClass(s: string) {
    return s === 'completed' ? 'badge-active' : s === 'error' ? 'badge-error'
         : s === 'running'   ? 'badge-pending' : 'badge-draft';
  }
  function statusLabel(s: string) {
    return s === 'completed' ? 'Terminée' : s === 'error' ? 'Erreur'
         : s === 'running'   ? 'En cours' : 'En attente';
  }
  function fmtDur(ms?: number) {
    if (!ms) return '—';
    return ms < 1000 ? `${ms} ms` : `${(ms / 1000).toFixed(1)} s`;
  }
  function fmtDate(s: string) {
    return new Date(s).toLocaleString('fr-FR', { dateStyle: 'short', timeStyle: 'short' });
  }
</script>

<div class="tab-content">
  <div class="tab-header">
    <h2>Exécutions</h2>
    <div class="header-actions">
      <a class="btn-sm" href="/dashboard">
        <LayoutDashboard size={13} /> Voir le dashboard
      </a>
      <button class="btn-primary" onclick={openModal}>
        <Play size={13} /> Lancer
      </button>
    </div>
  </div>

  {#if loading}
    <p class="loading">Chargement…</p>
  {:else if error}
    <div class="alert-error">{error}</div>
  {:else if execList.length === 0}
    <div class="empty-state">
      <p>Aucune exécution.</p>
      <p>Lancez votre première simulation depuis le bouton ci-dessus.</p>
    </div>
  {:else}
    <div class="card">
      <table class="exec-table">
        <thead>
          <tr>
            <th>Étude</th>
            <th>Label</th>
            <th>Méthode</th>
            <th>Statut</th>
            <th>Durée</th>
            <th>Date</th>
            <th></th>
          </tr>
        </thead>
        <tbody>
          {#each execList as ex}
            <tr>
              <td class="cell-study">{ex.study_name ?? '—'}</td>
              <td class="cell-label">{#if ex.label}{ex.label}{:else}<span class="muted">—</span>{/if}</td>
              <td><span class="method-tag">{ex.method}</span></td>
              <td><span class="badge {statusClass(ex.status)}">{statusLabel(ex.status)}</span></td>
              <td class="cell-num">{fmtDur(ex.duration_ms)}</td>
              <td class="cell-date">{fmtDate(ex.created_at)}</td>
              <td class="cell-actions">
                <a class="btn-sm" href="/dashboard" title="Voir dans le dashboard">
                  <LayoutDashboard size={11} />
                </a>
                <button class="btn-sm btn-danger" onclick={() => del(ex.id)} title="Supprimer">
                  <Trash2 size={11} />
                </button>
              </td>
            </tr>
          {/each}
        </tbody>
      </table>
    </div>
  {/if}
</div>

{#if showModal}
  <!-- svelte-ignore a11y_click_events_have_key_events a11y_no_static_element_interactions -->
  <div class="overlay" onclick={() => showModal = false}>
    <!-- svelte-ignore a11y_click_events_have_key_events a11y_no_static_element_interactions -->
    <div class="modal" onclick={e => e.stopPropagation()}>
      <div class="modal-hd">
        <h3>Lancer une exécution</h3>
        <button onclick={() => showModal = false}><X size={16} /></button>
      </div>
      <div class="modal-bd">
        {#if modalError}<div class="alert-error">{modalError}</div>{/if}
        <label>Étude
          <select bind:value={studyId}>
            <option value="">— Sélectionner —</option>
            {#each allStudies as s}
              <option value={s.id}>
                {s.name} ({s.status === 'ready' ? '✓ Prête' : s.status}) · {s.unit_count} unité(s)
              </option>
            {/each}
          </select>
        </label>
        <label>Label (optionnel)
          <input bind:value={label} placeholder="Ex. Run Q4 2024 baseline" />
        </label>
        {#if launching}
          <div class="launch-progress">⏳ Calcul en cours…</div>
        {/if}
      </div>
      <div class="modal-ft">
        <button class="btn-sm" onclick={() => showModal = false} disabled={launching}>Annuler</button>
        <button class="btn-primary" onclick={launch} disabled={launching || !studyId}>
          {#if launching}Calcul…{:else}<Play size={13} /> Lancer{/if}
        </button>
      </div>
    </div>
  </div>
{/if}

<style>
  .header-actions { display: flex; gap: 8px; align-items: center; }

  .exec-table {
    width: 100%; border-collapse: collapse; font-size: 13px;
  }
  .exec-table th {
    padding: 10px 14px; text-align: left;
    font-size: 11px; font-weight: 700; color: #9ca3af;
    text-transform: uppercase; letter-spacing: .04em;
    background: #f9fafb; border-bottom: 1px solid #e5e7eb;
  }
  .exec-table td {
    padding: 10px 14px; border-bottom: 1px solid #f3f4f6;
    vertical-align: middle;
  }
  .exec-table tr:last-child td { border-bottom: none; }
  .exec-table tr:hover td { background: #fafafa; }

  .cell-study { font-weight: 600; color: #1a1a2e; }
  .cell-label { color: #6366f1; font-style: italic; font-size: 12px; }
  .cell-num   { text-align: right; font-variant-numeric: tabular-nums; color: #6b7280; }
  .cell-date  { color: #6b7280; font-size: 12px; white-space: nowrap; }
  .cell-actions { display: flex; gap: 6px; justify-content: flex-end; }
  .muted { color: #9ca3af; }

  .method-tag {
    display: inline-block; font-size: 11px; font-weight: 600;
    background: #eff6ff; color: #1d4ed8;
    padding: 2px 8px; border-radius: 6px;
  }

  .launch-progress {
    background: #eff6ff; border: 1px solid #bfdbfe;
    border-radius: 8px; padding: 10px 14px; font-size: 13px; color: #1e40af;
  }

  .overlay {
    position: fixed; inset: 0; background: rgba(0,0,0,.45);
    display: flex; align-items: center; justify-content: center; z-index: 100;
  }
  .modal {
    background: #fff; border-radius: 14px; width: 460px; max-width: 95vw;
    display: flex; flex-direction: column;
    box-shadow: 0 20px 60px rgba(0,0,0,.2);
  }
  .modal-hd {
    display: flex; justify-content: space-between; align-items: center;
    padding: 18px 20px 14px; border-bottom: 1px solid #f1f1f5;
  }
  .modal-hd h3 { font-size: 15px; font-weight: 700; }
  .modal-hd button { background: none; border: none; cursor: pointer; color: #6b7280; }
  .modal-bd { padding: 16px 20px; display: flex; flex-direction: column; gap: 12px; }
  .modal-ft {
    display: flex; justify-content: flex-end; gap: 8px;
    padding: 12px 20px; border-top: 1px solid #f1f1f5;
  }
</style>
