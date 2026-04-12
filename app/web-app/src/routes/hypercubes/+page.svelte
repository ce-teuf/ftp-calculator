<script lang="ts">
  import { rateMatrices, hypercubes } from '$lib/api/client';
  import type {
    RateMatrixSummary,
    HypercubeSummary,
    HypercubeDetail,
    Combination,
  } from '$lib/api/client';
  import { Plus, Trash2, Pencil, X, Layers, ChevronRight } from '@lucide/svelte';
  import MonthPicker from '$lib/components/MonthPicker.svelte';

  // ── État global ───────────────────────────────────────────────────────────────

  let list      = $state<HypercubeSummary[]>([]);
  let allMats   = $state<RateMatrixSummary[]>([]);
  let loading   = $state(true);
  let error     = $state<string | null>(null);

  // Panneau de création / édition
  let showForm   = $state(false);
  let editingId  = $state<string | null>(null);
  let saving     = $state(false);
  let formError  = $state<string | null>(null);

  // Champs du formulaire
  let fName      = $state('');
  let fDesc      = $state('');
  let fStartDate = $state('');   // YYYY-MM (input type="month")
  let fEndDate   = $state('');
  let fProjEnd   = $state('');
  let fStatus    = $state<'draft' | 'active' | 'archived'>('draft');
  let fMatrixIds = $state<Set<string>>(new Set());

  // Panneau de détail
  let selectedId    = $state<string | null>(null);
  let detail        = $state<HypercubeDetail | null>(null);
  let combos        = $state<Combination[] | null>(null);
  let detailLoading = $state(false);
  let combosLoading = $state(false);
  let showCombos    = $state(false);

  // ── Couleurs des risques ──────────────────────────────────────────────────────

  const RISK_COLORS: Record<string, string> = {
    base_rate:      '#3b82f6',
    credit_spread:  '#ef4444',
    tlp:            '#8b5cf6',
    clp:            '#06b6d4',
    basis_risk:     '#f59e0b',
    oas:            '#10b981',
    capital_charge: '#64748b',
    xva:            '#ec4899',
    operational:    '#9ca3af',
    country_risk:   '#f97316',
    concentration:  '#84cc16',
    mrel_levy:      '#d97706',
    incentive:      '#0ea5e9',
    rollover:       '#7c3aed',
  };

  function riskColor(key: string): string {
    return RISK_COLORS[key] ?? '#6b7280';
  }

  // ── Chargement initial ────────────────────────────────────────────────────────

  async function loadAll() {
    loading = true;
    error   = null;
    const [hcRes, matRes] = await Promise.allSettled([
      hypercubes.list(),
      rateMatrices.list(),
    ]);
    if (hcRes.status  === 'fulfilled') list    = hcRes.value;
    else error = hcRes.reason?.message ?? 'Erreur chargement hypercubes';
    if (matRes.status === 'fulfilled') allMats = matRes.value;
    else if (!error) error = matRes.reason?.message ?? 'Erreur chargement matrices';
    loading = false;
  }

  loadAll();

  // ── Sélection / détail ────────────────────────────────────────────────────────

  async function selectHypercube(id: string) {
    if (selectedId === id) return;
    selectedId    = id;
    detail        = null;
    combos        = null;
    showCombos    = false;
    detailLoading = true;
    try {
      detail = await hypercubes.get(id);
    } catch (e: any) {
      error = e.message;
    } finally {
      detailLoading = false;
    }
  }

  async function loadCombos() {
    if (!selectedId) return;
    combosLoading = true;
    try {
      combos     = await hypercubes.combinations(selectedId);
      showCombos = true;
    } catch (e: any) {
      error = e.message;
    } finally {
      combosLoading = false;
    }
  }

  // ── Formulaire ────────────────────────────────────────────────────────────────

  /** YYYY-MM-DD (depuis l'API) → YYYY-MM (pour input[type=month]) */
  function toMonthInput(d: string): string {
    return d ? d.slice(0, 7) : '';
  }
  /** YYYY-MM (depuis input) → YYYY-MM-01 (pour l'API qui attend une DATE) */
  function toApiDate(m: string): string {
    return m ? `${m}-01` : '';
  }

  function openCreateForm() {
    editingId  = null;
    fName      = '';
    fDesc      = '';
    fStartDate = '';
    fEndDate   = '';
    fProjEnd   = '';
    fStatus    = 'draft';
    fMatrixIds = new Set();
    formError  = null;
    showForm   = true;
  }

  function openEditForm(hc: HypercubeSummary) {
    editingId  = hc.id;
    fName      = hc.name;
    fDesc      = hc.description ?? '';
    fStartDate = toMonthInput(hc.start_date);
    fEndDate   = toMonthInput(hc.end_date);
    fProjEnd   = hc.proj_end_date ? toMonthInput(hc.proj_end_date) : '';
    fStatus    = hc.status;
    formError    = null;
    // Pré-remplir les matrices si on a le détail
    if (detail && detail.id === hc.id) {
      fMatrixIds = new Set(detail.matrices.map(m => m.id));
    } else {
      fMatrixIds = new Set();
      // Charger le détail pour avoir les matrices sélectionnées
      hypercubes.get(hc.id).then(d => {
        fMatrixIds = new Set(d.matrices.map(m => m.id));
      });
    }
    showForm = true;
  }

  function closeForm() {
    showForm  = false;
    editingId = null;
    formError = null;
  }

  function toggleMatrix(id: string) {
    if (fMatrixIds.has(id)) {
      fMatrixIds.delete(id);
    } else {
      fMatrixIds.add(id);
    }
    fMatrixIds = new Set(fMatrixIds); // trigger reactivity
  }

  async function saveForm() {
    if (!fName.trim()) { formError = 'Le nom est requis'; return; }
    if (!fStartDate)   { formError = 'La date de début est requise'; return; }
    if (!fEndDate)     { formError = 'La date de fin est requise'; return; }
    if (fEndDate < fStartDate) { formError = 'La date de fin doit être >= date de début'; return; }
    if (fProjEnd && fProjEnd < fEndDate) {
      formError = 'La date de fin projection doit être >= date de fin observée';
      return;
    }

    saving    = true;
    formError = null;
    try {
      const payload = {
        name:          fName.trim(),
        description:   fDesc || undefined,
        start_date:    toApiDate(fStartDate),
        end_date:      toApiDate(fEndDate),
        proj_end_date: fProjEnd ? toApiDate(fProjEnd) : undefined,
        status:        fStatus,
        matrix_ids:    [...fMatrixIds],
      };

      if (editingId) {
        await hypercubes.update(editingId, payload);
      } else {
        await hypercubes.create(payload);
      }

      await loadAll();
      closeForm();

      if (editingId) {
        await selectHypercube(editingId);
      }
    } catch (e: any) {
      formError = e.message ?? 'Erreur lors de la sauvegarde';
    } finally {
      saving = false;
    }
  }

  async function deleteHypercube(id: string, name: string) {
    if (!confirm(`Supprimer l'hypercube "${name}" ?`)) return;
    try {
      await hypercubes.delete(id);
      if (selectedId === id) {
        selectedId = null;
        detail     = null;
        combos     = null;
      }
      await loadAll();
    } catch (e: any) {
      error = e.message;
    }
  }

  // ── Helpers d'affichage ───────────────────────────────────────────────────────

  function fmtDate(d: string): string {
    return d ? d.slice(0, 7) : '—';
  }

  /** Tous les types de risque couverts par les combinaisons affichées. */
  function allRisksInCombos(cs: Combination[]): string[] {
    const set = new Set<string>();
    cs.forEach(c => c.risks_covered.forEach(r => set.add(r)));
    return [...set].sort();
  }

  /** Toutes les matrices présentes dans le détail courant, triées par nb de risques desc. */
  function matrixRiskConflicts(): Map<string, string[]> {
    const m = new Map<string, string[]>();
    if (!detail) return m;
    for (const mat of detail.matrices) {
      m.set(mat.id, mat.risks);
    }
    return m;
  }
</script>

<div class="tab-content">
  <!-- ── En-tête ─────────────────────────────────────────────────────────────── -->
  <div class="tab-header">
    <h2>Hypercubes</h2>
    <button class="btn-primary" onclick={openCreateForm}>
      <Plus size={15} /> Nouvel hypercube
    </button>
  </div>

  {#if error}
    <div class="alert-error">{error}</div>
  {/if}

  <!-- ── Layout deux colonnes ─────────────────────────────────────────────────── -->
  <div class="layout">

    <!-- ── Liste ──────────────────────────────────────────────────────────────── -->
    <div class="list-panel card">
      {#if loading}
        <p class="loading" style="padding:20px 16px">Chargement…</p>
      {:else if list.length === 0}
        <div class="empty-state">
          <p>Aucun hypercube créé.</p>
          <button class="btn-primary" onclick={openCreateForm}>
            <Plus size={14} /> Créer le premier
          </button>
        </div>
      {:else}
        {#each list as hc}
          <div
            class="list-item"
            class:list-item--active={selectedId === hc.id}
            role="button"
            tabindex="0"
            onclick={() => selectHypercube(hc.id)}
            onkeydown={e => e.key === 'Enter' && selectHypercube(hc.id)}
          >
            <div class="list-item-header">
              <span class="list-item-name">{hc.name}</span>
              <span class="badge badge-{hc.status}">{hc.status}</span>
            </div>
            <div class="list-item-meta">
              {fmtDate(hc.start_date)} → {fmtDate(hc.end_date)}
              {#if hc.proj_end_date}
                <span class="proj-badge">proj. → {fmtDate(hc.proj_end_date)}</span>
              {/if}
            </div>
            <div class="list-item-stats">
              <span>{hc.matrix_count} matrice{hc.matrix_count !== 1 ? 's' : ''}</span>
              <span class="dot">·</span>
              <span>{hc.combination_count} combinaison{hc.combination_count !== 1 ? 's' : ''}</span>
            </div>
            <div class="list-item-actions" onclick={e => e.stopPropagation()}>
              <button class="btn-sm" title="Modifier" onclick={() => openEditForm(hc)}>
                <Pencil size={12} />
              </button>
              <button class="btn-sm btn-danger" title="Supprimer"
                onclick={() => deleteHypercube(hc.id, hc.name)}>
                <Trash2 size={12} />
              </button>
            </div>
          </div>
        {/each}
      {/if}
    </div>

    <!-- ── Panneau détail ──────────────────────────────────────────────────────── -->
    <div class="detail-panel">

      {#if showForm}
        <!-- ── Formulaire création / édition ───────────────────────────────────── -->
        <div class="card form-card">
          <div class="form-header">
            <h3>{editingId ? 'Modifier l\'hypercube' : 'Nouvel hypercube'}</h3>
            <button class="icon-btn" onclick={closeForm}><X size={16} /></button>
          </div>

          {#if formError}
            <div class="alert-error">{formError}</div>
          {/if}

          <div class="form-grid">
            <label style="grid-column: 1/-1">
              Nom *
              <input bind:value={fName} placeholder="Ex. Scénario Base EUR 2024-2026" />
            </label>
            <label style="grid-column: 1/-1">
              Description
              <textarea bind:value={fDesc} rows={2} placeholder="Description optionnelle"></textarea>
            </label>

            <label>
              Mois de début *
              <MonthPicker bind:value={fStartDate} />
            </label>
            <label>
              Mois de fin (réalisé) *
              <MonthPicker bind:value={fEndDate} />
            </label>
            <label>
              Mois de fin (projection)
              <MonthPicker bind:value={fProjEnd} />
            </label>
            <label>
              Statut
              <select bind:value={fStatus}>
                <option value="draft">Brouillon</option>
                <option value="active">Actif</option>
                <option value="archived">Archivé</option>
              </select>
            </label>
          </div>

          <!-- Sélection des matrices -->
          <div class="matrices-section">
            <div class="matrices-section-header">
              <span class="matrices-section-title">Matrices de taux</span>
              <span class="matrices-count">{fMatrixIds.size} sélectionnée{fMatrixIds.size !== 1 ? 's' : ''}</span>
            </div>
            {#if allMats.length === 0}
              <p class="loading">Aucune matrice disponible</p>
            {:else}
              <div class="matrix-picker">
                {#each allMats as mat}
                  <label class="matrix-option" class:matrix-option--selected={fMatrixIds.has(mat.id)}>
                    <input
                      type="checkbox"
                      checked={fMatrixIds.has(mat.id)}
                      onchange={() => toggleMatrix(mat.id)}
                    />
                    <div class="matrix-option-body">
                      <div class="matrix-option-name">{mat.name}</div>
                      <div class="matrix-option-meta">
                        {mat.currency ?? '—'} ·
                        {mat.date_from ?? '?'} → {mat.date_to ?? '?'}
                      </div>
                      <div class="matrix-option-risks">
                        {#each mat.risks as rk}
                          <span class="risk-badge" style="--c: {riskColor(rk)}">{rk}</span>
                        {/each}
                      </div>
                    </div>
                  </label>
                {/each}
              </div>
            {/if}
          </div>

          <div class="form-actions">
            <button class="btn-sm" onclick={closeForm}>Annuler</button>
            <button class="btn-primary" onclick={saveForm} disabled={saving}>
              {saving ? 'Enregistrement…' : (editingId ? 'Mettre à jour' : 'Créer')}
            </button>
          </div>
        </div>

      {:else if detailLoading}
        <div class="card" style="padding:32px;text-align:center">
          <p class="loading">Chargement du détail…</p>
        </div>

      {:else if detail}
        <!-- ── Vue détail ──────────────────────────────────────────────────────── -->
        <div class="card detail-card">
          <div class="detail-header">
            <div>
              <h3 class="detail-name">{detail.name}</h3>
              {#if detail.description}
                <p class="detail-desc">{detail.description}</p>
              {/if}
            </div>
            <span class="badge badge-{detail.status}">{detail.status}</span>
          </div>

          <!-- Métadonnées -->
          <div class="meta-grid">
            <div class="meta-item">
              <span class="meta-label">Période</span>
              <span class="meta-value">{fmtDate(detail.start_date)} → {fmtDate(detail.end_date)}</span>
            </div>
            {#if detail.proj_end_date}
              <div class="meta-item">
                <span class="meta-label">Projection jusqu'à</span>
                <span class="meta-value">{fmtDate(detail.proj_end_date)}</span>
              </div>
            {/if}
            <div class="meta-item">
              <span class="meta-label">Combinaisons valides</span>
              <span class="meta-value combo-count">{detail.combination_count}</span>
            </div>
          </div>

          <!-- Matrices incluses -->
          <div class="section">
            <h4 class="section-title">
              Matrices incluses
              <span class="section-count">{detail.matrices.length}</span>
            </h4>
            {#if detail.matrices.length === 0}
              <p class="empty-inline">Aucune matrice sélectionnée</p>
            {:else}
              <div class="matrix-list">
                {#each detail.matrices as mat}
                  <div class="matrix-row">
                    <div class="matrix-row-info">
                      <span class="matrix-row-name">{mat.name}</span>
                      <span class="matrix-row-dates">
                        {mat.currency ?? '—'} · {mat.date_from ?? '?'} → {mat.date_to ?? '?'}
                      </span>
                    </div>
                    <div class="matrix-row-risks">
                      {#each mat.risks as rk}
                        <span class="risk-badge" style="--c: {riskColor(rk)}">{rk}</span>
                      {/each}
                    </div>
                  </div>
                {/each}
              </div>
            {/if}
          </div>

          <!-- Combinaisons valides -->
          <div class="section">
            <div class="section-title-row">
              <h4 class="section-title">
                Combinaisons valides
                <span class="section-count">{detail.combination_count}</span>
              </h4>
              {#if !showCombos}
                <button class="btn-sm" onclick={loadCombos} disabled={combosLoading}>
                  {combosLoading ? 'Calcul…' : 'Afficher'}
                </button>
              {:else}
                <button class="btn-sm" onclick={() => { showCombos = false; combos = null; }}>
                  Masquer
                </button>
              {/if}
            </div>

            {#if showCombos && combos}
              {#if combos.length === 0}
                <p class="empty-inline">
                  Aucune combinaison valide — vérifiez les conflits de risques entre matrices.
                </p>
              {:else}
                {@const riskCols = allRisksInCombos(combos)}
                <div class="combos-wrapper">
                  <table class="combos-table">
                    <thead>
                      <tr>
                        <th class="th-combo"># Combinaison</th>
                        {#each riskCols as rk}
                          <th class="th-risk">
                            <span class="risk-dot" style="background:{riskColor(rk)}"></span>
                            {rk}
                          </th>
                        {/each}
                      </tr>
                    </thead>
                    <tbody>
                      {#each combos as combo, i}
                        <tr>
                          <td class="td-combo">
                            <div class="combo-names">
                              {#each combo.matrix_names as n, ni}
                                {#if ni > 0}<span class="combo-sep">+</span>{/if}
                                <span class="combo-name">{n}</span>
                              {/each}
                            </div>
                          </td>
                          {#each riskCols as rk}
                            <td class="td-risk">
                              {#if combo.risks_covered.includes(rk)}
                                <span class="check" style="color:{riskColor(rk)}">✓</span>
                              {/if}
                            </td>
                          {/each}
                        </tr>
                      {/each}
                    </tbody>
                  </table>
                </div>

                <!-- Avertissement couverture -->
                {@const uncovered = detail.matrices.flatMap(m => m.risks)
                  .filter((r, i, arr) => arr.indexOf(r) === i)
                  .filter(r => !combos.some(c => c.matrix_ids.length === detail.matrices.length && c.risks_covered.includes(r)))}
                {#if detail.matrices.length > 0}
                  <p class="combos-note">
                    {combos.filter(c => c.matrix_ids.length === detail.matrices.length).length} combinaison(s) utilisent toutes les matrices.
                  </p>
                {/if}
              {/if}
            {/if}
          </div>
        </div>

      {:else if !selectedId}
        <div class="card empty-detail">
          <Layers size={40} color="#d1d5db" />
          <p>Sélectionnez un hypercube dans la liste<br>ou créez-en un nouveau.</p>
        </div>
      {/if}
    </div>
  </div>
</div>

<style>
  .layout {
    display: grid;
    grid-template-columns: 320px 1fr;
    gap: 20px;
    align-items: start;
  }

  /* ── Liste ── */
  .list-panel {
    display: flex;
    flex-direction: column;
    overflow: hidden;
  }

  .list-item {
    padding: 12px 14px;
    border-bottom: 1px solid #f3f4f6;
    cursor: pointer;
    transition: background 120ms;
    position: relative;
  }
  .list-item:last-child { border-bottom: none; }
  .list-item:hover { background: #f9fafb; }
  .list-item--active { background: #ede9fe; }
  .list-item--active:hover { background: #ddd6fe; }

  .list-item-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 8px;
    margin-bottom: 3px;
  }
  .list-item-name { font-size: 13.5px; font-weight: 600; color: #1a1a2e; }
  .list-item-meta {
    font-size: 11.5px;
    color: #6b7280;
    margin-bottom: 2px;
    display: flex;
    align-items: center;
    gap: 6px;
    flex-wrap: wrap;
  }
  .proj-badge {
    background: #fef3c7;
    color: #92400e;
    padding: 1px 5px;
    border-radius: 4px;
    font-size: 10.5px;
    font-weight: 600;
  }
  .list-item-stats {
    font-size: 11px;
    color: #9ca3af;
    display: flex;
    align-items: center;
    gap: 4px;
  }
  .dot { opacity: .5; }
  .granularity { font-style: italic; }

  .list-item-actions {
    display: none;
    gap: 4px;
    position: absolute;
    right: 10px;
    top: 10px;
  }
  .list-item:hover .list-item-actions { display: flex; }

  /* ── Détail / formulaire ── */
  .detail-panel { min-width: 0; }

  .form-card, .detail-card {
    padding: 22px 24px;
  }

  .form-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    margin-bottom: 18px;
  }
  .form-header h3 { font-size: 15px; font-weight: 700; color: #1a1a2e; }

  .icon-btn {
    background: none;
    border: none;
    cursor: pointer;
    color: #9ca3af;
    padding: 4px;
    border-radius: 6px;
    display: flex;
    align-items: center;
  }
  .icon-btn:hover { background: #f3f4f6; color: #374151; }

  .form-grid {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 12px;
    margin-bottom: 18px;
  }

  /* ── Sélecteur de matrices ── */
  .matrices-section { margin-bottom: 18px; }
  .matrices-section-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    margin-bottom: 8px;
  }
  .matrices-section-title {
    font-size: 12.5px;
    font-weight: 600;
    color: #374151;
  }
  .matrices-count {
    font-size: 11.5px;
    color: #6b7280;
    background: #f3f4f6;
    padding: 2px 8px;
    border-radius: 12px;
  }

  .matrix-picker {
    display: flex;
    flex-direction: column;
    gap: 6px;
    max-height: 280px;
    overflow-y: auto;
    border: 1px solid #e5e7eb;
    border-radius: 8px;
    padding: 8px;
    background: #fafafa;
  }

  .matrix-option {
    display: flex;
    align-items: flex-start;
    gap: 10px;
    padding: 8px 10px;
    border-radius: 6px;
    cursor: pointer;
    transition: background 100ms;
    border: 1px solid transparent;
  }
  .matrix-option:hover { background: #f3f4f6; }
  .matrix-option--selected {
    background: #ede9fe;
    border-color: #c4b5fd;
  }

  .matrix-option input[type="checkbox"] {
    width: auto;
    margin-top: 2px;
    flex-shrink: 0;
    accent-color: #6366f1;
  }

  .matrix-option-body { flex: 1; min-width: 0; }
  .matrix-option-name {
    font-size: 13px;
    font-weight: 600;
    color: #1a1a2e;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .matrix-option-meta { font-size: 11px; color: #9ca3af; margin: 2px 0; }
  .matrix-option-risks { display: flex; flex-wrap: wrap; gap: 3px; }

  /* ── Badges de risque ── */
  .risk-badge {
    display: inline-block;
    padding: 1px 6px;
    border-radius: 4px;
    font-size: 10.5px;
    font-weight: 600;
    background: color-mix(in srgb, var(--c) 15%, white);
    color: var(--c);
  }

  .form-actions {
    display: flex;
    justify-content: flex-end;
    gap: 8px;
    padding-top: 16px;
    border-top: 1px solid #f3f4f6;
  }

  /* ── Détail ── */
  .detail-header {
    display: flex;
    align-items: flex-start;
    justify-content: space-between;
    gap: 12px;
    margin-bottom: 16px;
  }
  .detail-name { font-size: 17px; font-weight: 700; color: #1a1a2e; }
  .detail-desc { font-size: 12.5px; color: #6b7280; margin-top: 4px; }

  .meta-grid {
    display: flex;
    flex-wrap: wrap;
    gap: 16px;
    padding: 12px 14px;
    background: #f9fafb;
    border-radius: 8px;
    margin-bottom: 20px;
  }
  .meta-item { display: flex; flex-direction: column; gap: 2px; }
  .meta-label { font-size: 10.5px; font-weight: 600; color: #9ca3af; text-transform: uppercase; letter-spacing: .04em; }
  .meta-value { font-size: 13.5px; font-weight: 600; color: #1a1a2e; }
  .combo-count { color: #6366f1; }

  /* ── Sections ── */
  .section { margin-bottom: 20px; }
  .section-title-row {
    display: flex;
    align-items: center;
    justify-content: space-between;
    margin-bottom: 10px;
  }
  .section-title {
    font-size: 13px;
    font-weight: 700;
    color: #374151;
    display: flex;
    align-items: center;
    gap: 6px;
    margin-bottom: 10px;
  }
  .section-count {
    background: #e0e7ff;
    color: #3730a3;
    padding: 1px 7px;
    border-radius: 10px;
    font-size: 11px;
    font-weight: 700;
  }
  .empty-inline { font-size: 12.5px; color: #9ca3af; padding: 8px 0; }

  /* ── Liste matrices dans le détail ── */
  .matrix-list {
    display: flex;
    flex-direction: column;
    gap: 6px;
  }
  .matrix-row {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 12px;
    padding: 8px 12px;
    background: #f9fafb;
    border-radius: 6px;
    border: 1px solid #f3f4f6;
  }
  .matrix-row-info { display: flex; flex-direction: column; gap: 2px; }
  .matrix-row-name { font-size: 13px; font-weight: 600; color: #1a1a2e; }
  .matrix-row-dates { font-size: 11px; color: #9ca3af; }
  .matrix-row-risks { display: flex; flex-wrap: wrap; gap: 3px; justify-content: flex-end; }

  /* ── Table des combinaisons ── */
  .combos-wrapper {
    overflow-x: auto;
    border: 1px solid #e5e7eb;
    border-radius: 8px;
  }
  .combos-table {
    border-collapse: collapse;
    width: 100%;
    font-size: 12px;
    min-width: 400px;
  }
  .combos-table th {
    background: #f9fafb;
    border-bottom: 1px solid #e5e7eb;
    padding: 7px 10px;
    text-align: left;
    font-weight: 600;
    color: #374151;
    white-space: nowrap;
  }
  .combos-table td {
    padding: 7px 10px;
    border-bottom: 1px solid #f3f4f6;
    vertical-align: top;
  }
  .combos-table tr:last-child td { border-bottom: none; }
  .combos-table tr:hover td { background: #fafafa; }

  .th-combo { min-width: 200px; }
  .th-risk { text-align: center; min-width: 90px; }

  .risk-dot {
    display: inline-block;
    width: 6px;
    height: 6px;
    border-radius: 50%;
    margin-right: 4px;
    vertical-align: middle;
  }

  .td-combo { }
  .td-risk { text-align: center; }

  .combo-names {
    display: flex;
    flex-direction: column;
    gap: 2px;
    line-height: 1.4;
  }
  .combo-name { font-size: 11.5px; color: #374151; }
  .combo-sep { font-size: 10px; color: #9ca3af; }
  .check { font-size: 14px; font-weight: 700; }

  .combos-note {
    font-size: 11.5px;
    color: #6b7280;
    margin-top: 8px;
    padding-left: 2px;
  }

  /* ── Empty state détail ── */
  .empty-detail {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 14px;
    padding: 60px 32px;
    text-align: center;
    color: #9ca3af;
    font-size: 13px;
  }
</style>
