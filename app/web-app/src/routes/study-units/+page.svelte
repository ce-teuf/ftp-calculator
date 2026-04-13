<script lang="ts">
  import {
    studyUnits, hypercubes, portfolios, amortSchedules,
  } from '$lib/api/client';
  import type {
    StudyUnitSummary, StudyUnitDetail, AssignmentInfo,
    HypercubeSummary, PortfolioSummary, PortfolioDetail,
    Combination, PairInfo, ScheduleSummary, ValidationReport, ValidationCheck,
  } from '$lib/api/client';
  import MonthPicker from '$lib/components/MonthPicker.svelte';
  import {
    Plus, Trash2, Pencil, X, FlaskConical, CheckCircle, AlertCircle,
  } from '@lucide/svelte';

  // ── État global ───────────────────────────────────────────────────────────────

  let unitList    = $state<StudyUnitSummary[]>([]);
  let loading     = $state(true);
  let error       = $state<string | null>(null);

  // Sélection
  let selectedId    = $state<string | null>(null);
  let detail        = $state<StudyUnitDetail | null>(null);
  let detailLoading = $state(false);

  // Onglet
  let activeTab = $state<'config' | 'assignments' | 'validation'>('config');

  // Listes globales
  let allHypercubes = $state<HypercubeSummary[]>([]);
  let allPortfolios = $state<PortfolioSummary[]>([]);
  let allSchedules  = $state<ScheduleSummary[]>([]);

  // Données chargées pour l'unité sélectionnée
  let hypercubeCombos = $state<Combination[]>([]);
  let portfolioPairs  = $state<PairInfo[]>([]);

  // ── Formulaire study unit ────────────────────────────────────────────────────

  let showUnitForm      = $state(false);
  let editingId         = $state<string | null>(null);
  let fName             = $state('');
  let fDesc             = $state('');
  let fHypercubeId      = $state('');
  let fPortfolioId      = $state('');
  let fStartDate        = $state('');
  let fGranularityRule  = $state('none');
  let unitFormError     = $state<string | null>(null);
  let unitSaving        = $state(false);

  // ── Formulaire assignment ────────────────────────────────────────────────────

  let showAssignForm    = $state(false);
  let editingAssignId   = $state<string | null>(null);
  let aPairId           = $state('');
  let aComboIdx         = $state('-1');    // string pour le select
  let aLabel            = $state('');
  let aIsExistingStock  = $state(false);
  let aMethods          = $state<string[]>(['Stock']);
  let aFtpProfile       = $state<{ tenor: string; rate: string }[]>([]);
  let assignFormError   = $state<string | null>(null);
  let assignSaving      = $state(false);

  const ALL_METHODS = ['Stock', 'Flux'] as const;

  function selectMethod(m: string) {
    aMethods = [m];
  }

  // Buckets du schedule lié à la paire sélectionnée dans le formulaire
  let assignBuckets = $derived((() => {
    const pair = portfolioPairs.find(p => p.id === aPairId);
    if (!pair) return [];
    return allSchedules.find(s => s.id === pair.schedule_id)?.bucket_labels ?? [];
  })());

  // ── Validation ───────────────────────────────────────────────────────────────

  let freshReport = $state<ValidationReport | null>(null);
  let validating  = $state(false);

  // Affiche soit le résultat frais, soit la dernière passe stockée en DB
  function getReport(): ValidationReport | null {
    if (freshReport) return freshReport;
    if (!detail?.validation_log) return null;
    try {
      const checks: ValidationCheck[] = JSON.parse(detail.validation_log);
      return { is_valid: detail.is_valid, checks };
    } catch { return null; }
  }

  // ── Chargement ────────────────────────────────────────────────────────────────

  async function loadAll() {
    loading = true;
    error   = null;
    try {
      const [units, hcs, ports, scheds] = await Promise.all([
        studyUnits.list(),
        hypercubes.list(),
        portfolios.list(),
        amortSchedules.list(),
      ]);
      unitList      = units;
      allHypercubes = hcs;
      allPortfolios = ports;
      allSchedules  = scheds;
    } catch (e: any) {
      error = e.message;
    } finally {
      loading = false;
    }
  }

  async function selectUnit(id: string) {
    if (selectedId === id) return;
    selectedId      = id;
    detail          = null;
    hypercubeCombos = [];
    portfolioPairs  = [];
    freshReport     = null;
    activeTab       = 'config';
    detailLoading   = true;
    try {
      const d = await studyUnits.get(id);
      detail = d;
      const [combos, portDetail] = await Promise.all([
        hypercubes.combinations(d.hypercube_id),
        portfolios.get(d.portfolio_id),
      ]);
      hypercubeCombos = combos;
      portfolioPairs  = portDetail.pairs;
    } catch (e: any) {
      error = e.message;
    } finally {
      detailLoading = false;
    }
  }

  loadAll();

  // ── CRUD study unit ───────────────────────────────────────────────────────────

  function openCreate() {
    editingId        = null;
    fName            = '';
    fDesc            = '';
    fHypercubeId     = '';
    fPortfolioId     = '';
    fStartDate       = '';
    fGranularityRule = 'none';
    unitFormError    = null;
    showUnitForm     = true;
  }

  function openEdit(unit: StudyUnitSummary) {
    editingId        = unit.id;
    fName            = unit.name;
    fDesc            = unit.description ?? '';
    fHypercubeId     = unit.hypercube_id;
    fPortfolioId     = unit.portfolio_id;
    fStartDate       = toYM(unit.start_date);
    fGranularityRule = unit.granularity_rule;
    unitFormError    = null;
    showUnitForm     = true;
  }

  async function saveUnit() {
    if (!fName.trim())            { unitFormError = 'Le nom est requis';               return; }
    if (!editingId && !fHypercubeId) { unitFormError = 'Sélectionner un hypercube';   return; }
    if (!editingId && !fPortfolioId) { unitFormError = 'Sélectionner un portfolio';   return; }
    if (!editingId && !fStartDate)   { unitFormError = 'La date de départ est requise'; return; }
    unitSaving    = true;
    unitFormError = null;
    try {
      if (editingId) {
        await studyUnits.update(editingId, {
          name:             fName.trim(),
          description:      fDesc || undefined,
          start_date:       fStartDate ? toDate(fStartDate) : undefined,
          granularity_rule: fGranularityRule,
        });
      } else {
        await studyUnits.create({
          name:             fName.trim(),
          description:      fDesc || undefined,
          hypercube_id:     fHypercubeId,
          portfolio_id:     fPortfolioId,
          start_date:       toDate(fStartDate),
          granularity_rule: fGranularityRule,
        });
      }
      showUnitForm = false;
      await loadAll();
      if (editingId && selectedId === editingId) await selectUnit(editingId);
    } catch (e: any) {
      unitFormError = e.message;
    } finally {
      unitSaving = false;
    }
  }

  async function deleteUnit(id: string) {
    if (!confirm('Supprimer cette study unit et tous ses assignments ?')) return;
    try {
      await studyUnits.delete(id);
      if (selectedId === id) { selectedId = null; detail = null; }
      await loadAll();
    } catch (e: any) { alert(e.message); }
  }

  // ── CRUD assignments ──────────────────────────────────────────────────────────

  function openAssignCreate(pairId: string) {
    editingAssignId  = null;
    aPairId          = pairId;
    aComboIdx        = '-1';
    aLabel           = '';
    aIsExistingStock = false;
    aMethods         = ['Stock'];
    aFtpProfile      = initProfile(pairId);
    assignFormError  = null;
    showAssignForm   = true;
  }

  function openAssignEdit(a: AssignmentInfo) {
    editingAssignId  = a.id;
    aPairId          = a.pair_id;
    const key = JSON.stringify([...a.combination_matrix_ids].sort());
    const idx = hypercubeCombos.findIndex(
      c => JSON.stringify([...c.matrix_ids].sort()) === key,
    );
    aComboIdx        = String(idx);
    aLabel           = a.label ?? '';
    aIsExistingStock = a.is_existing_stock;
    aMethods         = [...(a.methods?.length ? a.methods : ['Stock'])];
    if (a.initial_ftp_profile_json?.length) {
      aFtpProfile = a.initial_ftp_profile_json.map(e => ({
        tenor: e.tenor, rate: String(e.rate),
      }));
    } else {
      aFtpProfile = initProfile(a.pair_id);
    }
    assignFormError = null;
    showAssignForm  = true;
  }

  function initProfile(pairId: string): { tenor: string; rate: string }[] {
    const pair = portfolioPairs.find(p => p.id === pairId);
    if (!pair) return [];
    const buckets = allSchedules.find(s => s.id === pair.schedule_id)?.bucket_labels ?? [];
    return buckets.map(t => ({ tenor: t, rate: '' }));
  }

  async function saveAssignment() {
    const idx = parseInt(aComboIdx);
    if (idx < 0) { assignFormError = 'Sélectionner une combinaison'; return; }
    const combo = hypercubeCombos[idx];
    if (!combo) { assignFormError = 'Combinaison invalide'; return; }
    const ftpProfile = aIsExistingStock
      ? aFtpProfile.map(e => ({ tenor: e.tenor, rate: parseFloat(e.rate) || 0 }))
      : undefined;
    assignSaving    = true;
    assignFormError = null;
    try {
      if (editingAssignId) {
        await studyUnits.updateAssignment(selectedId!, editingAssignId, {
          combination_matrix_ids:   combo.matrix_ids,
          label:                    aLabel || undefined,
          is_existing_stock:        aIsExistingStock,
          methods:                  aMethods,
          initial_ftp_profile_json: ftpProfile,
        });
      } else {
        await studyUnits.createAssignment(selectedId!, {
          pair_id:                  aPairId,
          combination_matrix_ids:   combo.matrix_ids,
          label:                    aLabel || undefined,
          is_existing_stock:        aIsExistingStock,
          methods:                  aMethods,
          initial_ftp_profile_json: ftpProfile,
        });
      }
      showAssignForm = false;
      detail = await studyUnits.get(selectedId!);
      unitList = await studyUnits.list();
    } catch (e: any) {
      assignFormError = e.message;
    } finally {
      assignSaving = false;
    }
  }

  async function deleteAssignment(aid: string) {
    if (!confirm('Supprimer cet assignment ?')) return;
    try {
      await studyUnits.deleteAssignment(selectedId!, aid);
      detail   = await studyUnits.get(selectedId!);
      unitList = await studyUnits.list();
    } catch (e: any) { alert(e.message); }
  }

  // ── Validation ────────────────────────────────────────────────────────────────

  async function runValidation() {
    if (!selectedId) return;
    validating  = true;
    freshReport = null;
    try {
      freshReport = await studyUnits.validate(selectedId);
      detail      = await studyUnits.get(selectedId);
      unitList    = await studyUnits.list();
    } catch (e: any) { alert(e.message); }
    finally { validating = false; }
  }

  // ── Helpers ───────────────────────────────────────────────────────────────────

  /** "2024-01-01" → "2024-01" */
  function toYM(d: string) { return d ? d.slice(0, 7) : ''; }
  /** "2024-01" → "2024-01-01" */
  function toDate(m: string) { return m ? `${m}-01` : ''; }

  function comboLabel(c: Combination) {
    return c.risks_covered.length
      ? c.risks_covered.join(' · ')
      : c.matrix_names.join(' · ');
  }

  function assignsForPair(pairId: string) {
    return detail?.assignments.filter(a => a.pair_id === pairId) ?? [];
  }

  function matchCombo(ids: string[]): Combination | undefined {
    const key = JSON.stringify([...ids].sort());
    return hypercubeCombos.find(
      c => JSON.stringify([...c.matrix_ids].sort()) === key,
    );
  }

  function hcName(id: string) {
    return allHypercubes.find(h => h.id === id)?.name ?? id;
  }
  function pName(id: string) {
    return allPortfolios.find(p => p.id === id)?.name ?? id;
  }

</script>

<!-- ── Layout ────────────────────────────────────────────────────────────────── -->
<div class="page-wrap">

  <!-- ── Bannière méthode FTP ──────────────────────────────────────────────────── -->
  <div class="method-info-banner">
    <span class="method-info-title">Méthode : Maturité Appariée</span>
    <div class="method-info-flavours">
      <div class="flavour-card">
        <span class="flavour-tag flavour-tag--stock">Stock</span>
        <span>Encours existant — profil résiduel × courbe FTP</span>
      </div>
      <div class="flavour-card">
        <span class="flavour-tag flavour-tag--flux">Flux</span>
        <span>Nouvelle production — construction anti-diagonale par cohorte</span>
      </div>
    </div>
  </div>

  <!-- ── Panneau principal (liste + détail) ───────────────────────────────────── -->
  <div class="page">

  <!-- Panneau gauche -->
  <aside class="left-panel card">
    <div class="panel-header">
      <h2>Study Units</h2>
      <button class="btn-primary" onclick={openCreate}><Plus size={13} /> Nouvelle</button>
    </div>

    {#if loading}
      <p class="loading" style="padding:16px">Chargement…</p>
    {:else if error}
      <div class="alert-error" style="margin:12px">{error}</div>
    {:else if unitList.length === 0}
      <div class="empty-state" style="margin:16px;padding:32px 16px">
        <p>Aucune study unit</p>
        <p>Créez-en une pour lier un hypercube à un portfolio.</p>
      </div>
    {:else}
      <div class="unit-list">
        {#each unitList as unit}
          <!-- svelte-ignore a11y_click_events_have_key_events a11y_no_static_element_interactions -->
          <div
            class="unit-item"
            class:unit-item--active={selectedId === unit.id}
            onclick={() => selectUnit(unit.id)}
          >
            <div class="unit-row1">
              <span class="unit-name">{unit.name}</span>
              {#if unit.is_valid}
                <span class="badge badge-active">✓ Valide</span>
              {:else}
                <span class="badge badge-draft">Non validée</span>
              {/if}
            </div>
            <div class="unit-row2">
              <span class="unit-meta-item">HC: {unit.hypercube_name}</span>
            </div>
            <div class="unit-row2">
              <span class="unit-meta-item">P: {unit.portfolio_name}</span>
              <span class="unit-count">{unit.assignment_count} assign.</span>
            </div>
            <!-- svelte-ignore a11y_click_events_have_key_events a11y_no_static_element_interactions -->
            <div class="unit-actions" onclick={e => e.stopPropagation()}>
              <button class="btn-sm" onclick={() => openEdit(unit)} title="Modifier">
                <Pencil size={11} />
              </button>
              <button class="btn-sm btn-danger" onclick={() => deleteUnit(unit.id)} title="Supprimer">
                <Trash2 size={11} />
              </button>
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
        <FlaskConical size={32} style="margin:0 auto 12px;opacity:.3" />
        <p>Sélectionnez une study unit pour voir son détail</p>
      </div>

    {:else if detailLoading}
      <p class="loading" style="padding:32px">Chargement…</p>

    {:else if detail}
      <!-- En-tête -->
      <div class="detail-header">
        <div>
          <h2>{detail.name}</h2>
          {#if detail.description}
            <p class="detail-desc">{detail.description}</p>
          {/if}
        </div>
        {#if detail.is_valid}
          <span class="badge badge-active" style="font-size:13px;padding:4px 12px">✓ Valide</span>
        {:else}
          <span class="badge badge-draft" style="font-size:13px;padding:4px 12px">Non validée</span>
        {/if}
      </div>

      <!-- Onglets -->
      <div class="tabs">
        <button class="tab" class:tab--active={activeTab==='config'}
          onclick={() => activeTab='config'}>Configuration</button>
        <button class="tab" class:tab--active={activeTab==='assignments'}
          onclick={() => activeTab='assignments'}>
          Assignments ({detail.assignments.length})
        </button>
        <button class="tab" class:tab--active={activeTab==='validation'}
          onclick={() => activeTab='validation'}>Validation</button>
      </div>

      <!-- ── Onglet Config ──────────────────────────────────────────────────── -->
      {#if activeTab === 'config'}
        <div class="tab-body">
          <div class="info-grid card">
            <div class="info-row">
              <span class="info-label">Hypercube</span>
              <span class="info-value">{hcName(detail.hypercube_id)}</span>
            </div>
            <div class="info-row">
              <span class="info-label">Portfolio</span>
              <span class="info-value">{pName(detail.portfolio_id)}</span>
            </div>
            <div class="info-row">
              <span class="info-label">Date de départ</span>
              <span class="info-value">{toYM(detail.start_date)}</span>
            </div>
            <div class="info-row">
              <span class="info-label">Règle de conversion</span>
              <span class="info-value">{detail.granularity_rule}</span>
            </div>
          </div>

          {#if hypercubeCombos.length > 0}
            <div class="combos-section">
              <h3>Combinaisons disponibles ({hypercubeCombos.length})</h3>
              <div class="combo-chips">
                {#each hypercubeCombos as combo, i}
                  <div class="combo-chip">
                    <span class="combo-num">#{i + 1}</span>
                    <span>{comboLabel(combo)}</span>
                    <span class="combo-detail">({combo.matrix_names.join(' + ')})</span>
                  </div>
                {/each}
              </div>
            </div>
          {:else}
            <p class="loading" style="margin-top:16px">Aucune combinaison dans ce hypercube.</p>
          {/if}
        </div>

      <!-- ── Onglet Assignments ─────────────────────────────────────────────── -->
      {:else if activeTab === 'assignments'}
        <div class="tab-body">
          {#if portfolioPairs.length === 0}
            <div class="empty-state">
              <p>Ce portfolio n'a pas encore de paires.</p>
              <p>Définissez des paires (vecteur × schedule) dans le module Portfolio.</p>
            </div>
          {:else}
            {#each portfolioPairs as pair}
              {@const pairAssigns = assignsForPair(pair.id)}
              {@const pairSchedType = allSchedules.find(s => s.id === pair.schedule_id)?.schedule_type ?? 'stock'}
              <div class="pair-block card">
                <div class="pair-head">
                  <div class="pair-title">
                    <span class="pair-title-line">
                      <strong>{pair.label ?? `${pair.vector_name} × ${pair.schedule_name}`}</strong>
                      <span class="stype-badge stype-badge--{pairSchedType}">
                        {pairSchedType === 'stock' ? 'Stock' : 'Nvl. prod.'}
                      </span>
                    </span>
                    {#if pair.label}
                      <span class="pair-subtitle">{pair.vector_name} × {pair.schedule_name}</span>
                    {/if}
                  </div>
                  <button class="btn-sm btn-success" onclick={() => openAssignCreate(pair.id)}>
                    <Plus size={11} /> Assigner
                  </button>
                </div>

                {#if pairAssigns.length === 0}
                  <p class="no-assign">Aucun assignment — cette paire sera ignorée lors du calcul</p>
                {:else}
                  <div class="assign-rows">
                    {#each pairAssigns as a}
                      {@const combo = matchCombo(a.combination_matrix_ids)}
                      <div class="assign-row">
                        <div class="assign-info">
                          {#if a.label}
                            <span class="assign-label-name">{a.label}</span>
                          {/if}
                          <span class="assign-combo-name">
                            {#if combo}
                              {comboLabel(combo)}
                            {:else}
                              {a.combination_matrix_ids.join(', ')}
                            {/if}
                          </span>
                          <div class="assign-methods">
                            {#each (a.methods ?? ['Stock']) as m}
                              <span class="method-tag">{m}</span>
                            {/each}
                            {#if a.is_existing_stock}
                              <span class="badge badge-frozen" style="font-size:11px">Stock existant</span>
                            {/if}
                          </div>
                        </div>
                        <div class="assign-btns">
                          <button class="btn-sm" onclick={() => openAssignEdit(a)}>
                            <Pencil size={11} />
                          </button>
                          <button class="btn-sm btn-danger" onclick={() => deleteAssignment(a.id)}>
                            <Trash2 size={11} />
                          </button>
                        </div>
                      </div>
                    {/each}
                  </div>
                {/if}
              </div>
            {/each}
          {/if}
        </div>

      <!-- ── Onglet Validation ──────────────────────────────────────────────── -->
      {:else if activeTab === 'validation'}
        {@const report = getReport()}
        <div class="tab-body">
          <div class="validate-bar">
            <button class="btn-primary" onclick={runValidation} disabled={validating}>
              {validating ? 'Validation en cours…' : 'Lancer la validation'}
            </button>
            <p class="validate-hint">
              Vérifie la compatibilité des dates, des combinaisons et des assignments.
            </p>
          </div>

          {#if report}
            <div class="report-card card" class:report-valid={report.is_valid} class:report-invalid={!report.is_valid}>
              <div class="report-status">
                {#if report.is_valid}
                  <CheckCircle size={18} />
                  <span>Study unit valide — prête pour l'exécution</span>
                {:else}
                  <AlertCircle size={18} />
                  <span>Study unit non valide — corrigez les erreurs avant d'exécuter</span>
                {/if}
              </div>
              <div class="check-list">
                {#each report.checks as c}
                  <div class="check-item" class:check-pass={c.passed} class:check-fail={!c.passed}>
                    <span class="check-icon">{c.passed ? '✓' : '✗'}</span>
                    <div class="check-body">
                      <span class="check-name">{c.check}</span>
                      <span class="check-msg">{c.message}</span>
                    </div>
                  </div>
                {/each}
              </div>
            </div>
          {/if}
        </div>
      {/if}
    {/if}
  </main>
  </div> <!-- end .page -->
</div> <!-- end .page-wrap -->

<!-- ── Modal : créer / modifier study unit ──────────────────────────────────── -->
{#if showUnitForm}
  <!-- svelte-ignore a11y_click_events_have_key_events a11y_no_static_element_interactions -->
  <div class="overlay" onclick={() => showUnitForm = false}>
    <!-- svelte-ignore a11y_click_events_have_key_events a11y_no_static_element_interactions -->
    <div class="modal" onclick={e => e.stopPropagation()}>
      <div class="modal-hd">
        <h3>{editingId ? 'Modifier la study unit' : 'Nouvelle study unit'}</h3>
        <button onclick={() => showUnitForm = false}><X size={16} /></button>
      </div>

      <div class="modal-bd">
        {#if unitFormError}
          <div class="alert-error">{unitFormError}</div>
        {/if}

        <label>Nom <input bind:value={fName} placeholder="Ex. Retail EUR 2024" /></label>
        <label>Description <textarea bind:value={fDesc} rows={2}></textarea></label>

        {#if !editingId}
          <label>Hypercube
            <select bind:value={fHypercubeId}>
              <option value="">— Sélectionner —</option>
              {#each allHypercubes as h}
                <option value={h.id}>{h.name} · {h.combination_count} combinaison(s)</option>
              {/each}
            </select>
          </label>

          <label>Portfolio
            <select bind:value={fPortfolioId}>
              <option value="">— Sélectionner —</option>
              {#each allPortfolios as p}
                <option value={p.id}>{p.name} · {p.pair_count} paire(s)</option>
              {/each}
            </select>
          </label>

          <label>Date de départ
            <MonthPicker bind:value={fStartDate} required />
          </label>
        {/if}

        <label>Règle de conversion de granularité
          <select bind:value={fGranularityRule}>
            <option value="none">Aucune (granularités identiques)</option>
            <option value="aggregate">Agrégation (hypercube plus fin → portfolio)</option>
            <option value="interpolate">Interpolation (portfolio plus fin → hypercube)</option>
          </select>
        </label>
      </div>

      <div class="modal-ft">
        <button class="btn-sm" onclick={() => showUnitForm = false}>Annuler</button>
        <button class="btn-primary" onclick={saveUnit} disabled={unitSaving}>
          {unitSaving ? 'Enregistrement…' : 'Enregistrer'}
        </button>
      </div>
    </div>
  </div>
{/if}

<!-- ── Modal : créer / modifier assignment ──────────────────────────────────── -->
{#if showAssignForm}
  {@const pair = portfolioPairs.find(p => p.id === aPairId)}
  <!-- svelte-ignore a11y_click_events_have_key_events a11y_no_static_element_interactions -->
  <div class="overlay" onclick={() => showAssignForm = false}>
    <!-- svelte-ignore a11y_click_events_have_key_events a11y_no_static_element_interactions -->
    <div class="modal modal--wide" onclick={e => e.stopPropagation()}>
      <div class="modal-hd">
        <h3>{editingAssignId ? 'Modifier l\'assignment' : 'Nouvel assignment'}</h3>
        <button onclick={() => showAssignForm = false}><X size={16} /></button>
      </div>

      <div class="modal-bd">
        {#if assignFormError}
          <div class="alert-error">{assignFormError}</div>
        {/if}

        {#if pair}
          <div class="pair-info-box">
            <span class="info-label">Paire</span>
            <strong>{pair.label ?? `${pair.vector_name} × ${pair.schedule_name}`}</strong>
            {#if pair.label}
              <span class="pair-subtitle">{pair.vector_name} × {pair.schedule_name}</span>
            {/if}
          </div>
        {/if}

        <label>Combinaison
          <select bind:value={aComboIdx}>
            <option value="-1">— Sélectionner —</option>
            {#each hypercubeCombos as combo, i}
              <option value={String(i)}>
                #{i + 1} — {comboLabel(combo)}
                ({combo.matrix_names.join(' + ')})
              </option>
            {/each}
          </select>
        </label>

        <label>Label (optionnel)
          <input bind:value={aLabel} placeholder="Ex. Scénario central" />
        </label>

        <div class="methods-pick">
          <span class="methods-pick-label">Variante de calcul *</span>
          <div class="methods-toggle-row">
            {#each ALL_METHODS as m}
              <button
                type="button"
                class="method-flavour-btn"
                class:method-flavour-btn--on={aMethods.includes(m)}
                class:method-flavour-btn--stock={m === 'Stock'}
                class:method-flavour-btn--flux={m === 'Flux'}
                onclick={() => selectMethod(m)}>
                {m === 'Stock' ? 'Stock (encours existant)' : 'Flux (nouvelle production)'}
              </button>
            {/each}
          </div>
        </div>

        <div class="toggle-row">
          <label class="inline-check">
            <input type="checkbox" bind:checked={aIsExistingStock} />
            Stock existant — initialiser avec un profil FTP historique à t=0
          </label>
        </div>

        {#if aIsExistingStock}
          {#if assignBuckets.length === 0}
            <p class="loading">Les buckets du schedule de cette paire sont introuvables.</p>
          {:else}
            <div class="ftp-block">
              <p class="ftp-title">Profil FTP initial (taux par tenor pour t=0)</p>
              <div class="ftp-grid">
                {#each aFtpProfile as row}
                  <span class="ftp-tenor">{row.tenor}</span>
                  <input
                    class="ftp-input"
                    type="number"
                    step="0.0001"
                    min="0"
                    placeholder="0.0000"
                    bind:value={row.rate}
                  />
                  <span class="ftp-unit">%</span>
                {/each}
              </div>
            </div>
          {/if}
        {/if}
      </div>

      <div class="modal-ft">
        <button class="btn-sm" onclick={() => showAssignForm = false}>Annuler</button>
        <button class="btn-primary" onclick={saveAssignment} disabled={assignSaving}>
          {assignSaving ? 'Enregistrement…' : 'Enregistrer'}
        </button>
      </div>
    </div>
  </div>
{/if}

<style>
  /* ── Layout ── */
  .page {
    display: flex;
    flex: 1;
    min-height: 0;
    overflow: hidden;
    gap: 0;
  }

  .left-panel {
    width: 300px;
    flex-shrink: 0;
    display: flex;
    flex-direction: column;
    border-radius: 0;
    border-right: 1px solid #e5e7eb;
    overflow-y: auto;
  }

  .right-panel {
    flex: 1;
    overflow-y: auto;
    background: #f4f5f9;
  }

  /* ── Panel header ── */
  .panel-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 20px 16px 14px;
    border-bottom: 1px solid #f1f1f5;
    flex-shrink: 0;
  }
  .panel-header h2 {
    font-size: 15px;
    font-weight: 700;
    color: #1a1a2e;
  }

  /* ── Unit list ── */
  .unit-list { padding: 8px; }

  .unit-item {
    width: 100%;
    text-align: left;
    background: transparent;
    border: 1px solid transparent;
    border-radius: 8px;
    padding: 10px 12px;
    margin-bottom: 4px;
    cursor: pointer;
    transition: background 100ms, border-color 100ms;
  }
  .unit-item:hover { background: #f4f5f9; }
  .unit-item--active { background: #eef2ff; border-color: #c7d2fe; }

  .unit-row1 {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 3px;
  }
  .unit-name { font-size: 13px; font-weight: 600; color: #1a1a2e; }

  .unit-row2 {
    display: flex;
    justify-content: space-between;
    align-items: center;
    gap: 6px;
  }
  .unit-meta-item { font-size: 11.5px; color: #6b7280; }
  .unit-count     { font-size: 11px; color: #9ca3af; }

  .unit-actions {
    display: flex;
    gap: 4px;
    margin-top: 6px;
    justify-content: flex-end;
  }

  /* ── Right panel ── */
  .detail-header {
    display: flex;
    justify-content: space-between;
    align-items: flex-start;
    padding: 24px 28px 0;
  }
  .detail-header h2 { font-size: 18px; font-weight: 700; color: #1a1a2e; }
  .detail-desc { font-size: 13px; color: #6b7280; margin-top: 3px; }

  /* ── Tabs ── */
  .tabs {
    display: flex;
    gap: 4px;
    padding: 16px 28px 0;
    border-bottom: 1px solid #e5e7eb;
    margin-bottom: 0;
  }
  .tab {
    padding: 7px 14px;
    font-size: 13px;
    font-weight: 500;
    border: none;
    background: transparent;
    color: #6b7280;
    cursor: pointer;
    border-bottom: 2px solid transparent;
    margin-bottom: -1px;
    border-radius: 4px 4px 0 0;
    transition: color 120ms, border-color 120ms;
  }
  .tab:hover   { color: #374151; }
  .tab--active { color: #6366f1; border-bottom-color: #6366f1; font-weight: 600; }

  .tab-body { padding: 20px 28px 32px; }

  /* ── Config tab ── */
  .info-grid { padding: 0; overflow: hidden; }
  .info-row {
    display: flex;
    align-items: center;
    padding: 10px 16px;
    border-bottom: 1px solid #f1f1f5;
    gap: 12px;
  }
  .info-row:last-child { border-bottom: none; }
  .info-label { font-size: 12px; font-weight: 600; color: #6b7280; width: 180px; flex-shrink: 0; }
  .info-value { font-size: 13px; color: #1a1a2e; }

  .combos-section { margin-top: 20px; }
  .combos-section h3 { font-size: 13px; font-weight: 600; color: #374151; margin-bottom: 10px; }
  .combo-chips { display: flex; flex-wrap: wrap; gap: 6px; }
  .combo-chip {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    background: #eef2ff;
    border: 1px solid #c7d2fe;
    border-radius: 8px;
    padding: 4px 10px;
    font-size: 12px;
  }
  .combo-num { font-weight: 700; color: #6366f1; }
  .combo-detail { color: #6b7280; }

  /* ── Assignments tab ── */
  .pair-block {
    padding: 14px 16px;
    margin-bottom: 12px;
    border-radius: 10px;
  }
  .pair-head {
    display: flex;
    justify-content: space-between;
    align-items: flex-start;
    margin-bottom: 10px;
  }
  .pair-title strong { font-size: 13px; font-weight: 600; color: #1a1a2e; }
  .pair-subtitle { display: block; font-size: 11.5px; color: #9ca3af; margin-top: 2px; }

  .no-assign { font-size: 12px; color: #d97706; font-style: italic; }

  .assign-rows { display: flex; flex-direction: column; gap: 6px; }
  .assign-row {
    display: flex;
    justify-content: space-between;
    align-items: center;
    background: #f9fafb;
    border: 1px solid #f1f1f5;
    border-radius: 6px;
    padding: 8px 10px;
  }
  .assign-info { display: flex; align-items: center; gap: 8px; flex-wrap: wrap; }
  .assign-label-name { font-size: 12.5px; font-weight: 600; color: #1a1a2e; }
  .assign-combo-name  { font-size: 12px; color: #4b5563; }
  .assign-btns { display: flex; gap: 4px; flex-shrink: 0; }

  /* ── Validation tab ── */
  .validate-bar {
    display: flex;
    align-items: center;
    gap: 16px;
    margin-bottom: 20px;
  }
  .validate-hint { font-size: 12.5px; color: #9ca3af; }

  .report-card {
    border-radius: 10px;
    overflow: hidden;
    border-left: 4px solid #e5e7eb;
  }
  .report-valid   { border-left-color: #10b981; }
  .report-invalid { border-left-color: #ef4444; }

  .report-status {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 12px 16px;
    font-size: 13px;
    font-weight: 600;
    border-bottom: 1px solid #f1f1f5;
  }
  .report-valid   .report-status { color: #065f46; }
  .report-invalid .report-status { color: #991b1b; }

  .check-list { padding: 8px 0; }
  .check-item {
    display: flex;
    align-items: flex-start;
    gap: 10px;
    padding: 8px 16px;
    border-bottom: 1px solid #f9fafb;
  }
  .check-item:last-child { border-bottom: none; }
  .check-icon { font-size: 13px; font-weight: 700; width: 16px; flex-shrink: 0; margin-top: 1px; }
  .check-pass .check-icon { color: #10b981; }
  .check-fail .check-icon { color: #ef4444; }
  .check-body { display: flex; flex-direction: column; gap: 2px; }
  .check-name { font-size: 12.5px; font-weight: 600; color: #374151; }
  .check-msg  { font-size: 12px; color: #6b7280; }

  /* ── Modals ── */
  .overlay {
    position: fixed; inset: 0;
    background: rgba(0,0,0,.45);
    display: flex; align-items: center; justify-content: center;
    z-index: 100;
  }
  .modal {
    background: #fff;
    border-radius: 14px;
    width: 480px;
    max-width: 95vw;
    max-height: 88vh;
    display: flex;
    flex-direction: column;
    box-shadow: 0 20px 60px rgba(0,0,0,.2);
  }
  .modal--wide { width: 560px; }

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

  /* ── Assignment form extras ── */
  .pair-info-box {
    background: #f9fafb;
    border: 1px solid #e5e7eb;
    border-radius: 8px;
    padding: 10px 14px;
    display: flex;
    flex-direction: column;
    gap: 3px;
  }
  .pair-info-box strong { font-size: 13px; }

  .toggle-row { display: flex; align-items: center; }
  .inline-check {
    display: flex;
    align-items: center;
    gap: 8px;
    font-size: 13px;
    font-weight: 500;
    color: #374151;
    cursor: pointer;
    flex-direction: row;
  }
  .inline-check input[type="checkbox"] {
    width: auto;
    flex-shrink: 0;
  }

  .ftp-block {
    background: #f9fafb;
    border: 1px solid #e5e7eb;
    border-radius: 8px;
    padding: 12px 14px;
  }
  .ftp-title { font-size: 12px; font-weight: 600; color: #374151; margin-bottom: 10px; }
  .ftp-grid {
    display: grid;
    grid-template-columns: 60px 1fr 24px;
    gap: 6px 8px;
    align-items: center;
  }
  .ftp-tenor { font-size: 12.5px; font-weight: 600; color: #4b5563; }
  .ftp-input { width: 100%; }
  .ftp-unit  { font-size: 12px; color: #9ca3af; }

  /* ── Layout ── */
  .page-wrap {
    display: flex;
    flex-direction: column;
    height: 100vh;
    overflow: hidden;
  }

  /* ── Méthode info banner ── */
  .method-info-banner {
    flex-shrink: 0;
    display: flex;
    align-items: center;
    gap: 20px;
    padding: 10px 20px;
    background: #f5f3ff;
    border-bottom: 1px solid #e0d9ff;
  }
  .method-info-title {
    font-size: 12.5px;
    font-weight: 700;
    color: #4338ca;
    white-space: nowrap;
  }
  .method-info-flavours {
    display: flex;
    gap: 12px;
  }
  .flavour-card {
    display: flex;
    align-items: center;
    gap: 8px;
    background: #fff;
    border: 1px solid #c7d2fe;
    border-radius: 8px;
    padding: 5px 12px;
    font-size: 12px;
    color: #374151;
  }
  .flavour-tag {
    font-size: 11px;
    font-weight: 700;
    border-radius: 4px;
    padding: 1px 7px;
  }
  .flavour-tag--stock { background: #eff6ff; color: #1d4ed8; }
  .flavour-tag--flux  { background: #f0fdf4; color: #166534; }

  /* ── Pair header title line ── */
  .pair-title-line { display: flex; align-items: center; gap: 8px; }

  /* ── stype badge ── */
  .stype-badge {
    display: inline-block; padding: 1px 7px;
    border-radius: 4px; font-size: 10px; font-weight: 700;
    letter-spacing: .03em; text-transform: uppercase;
  }
  .stype-badge--stock          { background: #eff6ff; color: #1d4ed8; }
  .stype-badge--new_production { background: #f0fdf4; color: #166534; }

  /* ── Method chips on assignment row ── */
  .assign-methods { display: flex; flex-wrap: wrap; gap: 4px; margin-top: 4px; }
  .method-tag {
    display: inline-block; padding: 1px 7px;
    background: #eef2ff; color: #4338ca;
    border-radius: 4px; font-size: 10.5px; font-weight: 600;
    border: 1px solid #c7d2fe;
  }

  /* ── Method picker in assignment form ── */
  .methods-pick { display: flex; flex-direction: column; gap: 8px; }
  .methods-pick-label { font-size: 12px; font-weight: 600; color: #374151; }
  .methods-toggle-row { display: flex; gap: 8px; }
  .method-flavour-btn {
    flex: 1;
    padding: 9px 16px;
    border-radius: 8px;
    border: 2px solid #e5e7eb;
    background: #f9fafb;
    color: #6b7280;
    font-size: 13px; font-weight: 500;
    cursor: pointer;
    transition: all .12s;
    text-align: center;
  }
  .method-flavour-btn:hover { border-color: #a5b4fc; color: #4338ca; background: #f5f3ff; }
  .method-flavour-btn--on.method-flavour-btn--stock {
    background: #eff6ff; border-color: #3b82f6; color: #1d4ed8; font-weight: 700;
  }
  .method-flavour-btn--on.method-flavour-btn--flux {
    background: #f0fdf4; border-color: #22c55e; color: #166534; font-weight: 700;
  }
</style>
