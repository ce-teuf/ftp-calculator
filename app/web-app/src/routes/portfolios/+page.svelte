<script lang="ts">
  import { tick } from 'svelte';
  import { portfolios, outstandingVectors, amortSchedules } from '$lib/api/client';
  import type {
    PortfolioSummary, PortfolioDetail,
    VectorSummary, VectorDetail,
    ScheduleSummary, ScheduleDetail,
    PairInfo,
  } from '$lib/api/client';
  import * as echarts from 'echarts';
  import { Plus, Trash2, Pencil, X, Briefcase, ChevronDown, ChevronUp, Link } from '@lucide/svelte';

  // ── État global ───────────────────────────────────────────────────────────────

  let portfolioList = $state<PortfolioSummary[]>([]);
  let loading       = $state(true);
  let error         = $state<string | null>(null);

  // Sélection / détail
  let selectedId    = $state<string | null>(null);
  let detail        = $state<PortfolioDetail | null>(null);
  let detailLoading = $state(false);

  // Onglet actif
  let activeTab = $state<'vectors' | 'schedules' | 'pairs'>('vectors');

  // Formulaire portfolio
  let showPortfolioForm = $state(false);
  let editingPortfolioId = $state<string | null>(null);
  let fName = $state('');
  let fDesc = $state('');
  let portfolioFormError = $state<string | null>(null);
  let portfolioSaving = $state(false);

  // Listes globales (pour "associer existant")
  let allVectors   = $state<VectorSummary[]>([]);
  let allSchedules = $state<ScheduleSummary[]>([]);

  // Panel upload/associer vecteur
  let showVectorPanel = $state(false);
  let vectorPanelMode = $state<'upload' | 'associate'>('upload');
  let vFile = $state<File | null>(null);
  let vName = $state('');
  let vDesc = $state('');
  let vUploading = $state(false);
  let vError = $state<string | null>(null);
  let vFileInput: HTMLInputElement;

  // Panel upload/associer schedule
  let showSchedulePanel = $state(false);
  let schedulePanelMode = $state<'upload' | 'associate'>('schedule');
  let sFile = $state<File | null>(null);
  let sName = $state('');
  let sDesc = $state('');
  let sUploading = $state(false);
  let sError = $state<string | null>(null);
  let sFileInput: HTMLInputElement;

  // Formulaire paire
  let showPairForm    = $state(false);
  let pairVectorId    = $state('');
  let pairScheduleId  = $state('');
  let pairLabel       = $state('');
  let pairError       = $state<string | null>(null);
  let pairSaving      = $state(false);

  // Charts (ECharts sur vecteur/schedule expandé)
  let expandedVectorId   = $state<string | null>(null);
  let expandedScheduleId = $state<string | null>(null);
  let vectorChartData    = $state<VectorDetail | null>(null);
  let scheduleChartData  = $state<ScheduleDetail | null>(null);
  let vectorChartEl:   HTMLDivElement;
  let scheduleChartEl: HTMLDivElement;
  let vchart: echarts.ECharts | null = null;
  let schart: echarts.ECharts | null = null;

  // ── Chargement ────────────────────────────────────────────────────────────────

  async function loadAll() {
    loading = true;
    error   = null;
    const [pRes, vRes, sRes] = await Promise.allSettled([
      portfolios.list(),
      outstandingVectors.list(),
      amortSchedules.list(),
    ]);
    if (pRes.status === 'fulfilled') portfolioList = pRes.value;
    else error = pRes.reason?.message ?? 'Erreur chargement portfolios';
    if (vRes.status === 'fulfilled') allVectors   = vRes.value;
    if (sRes.status === 'fulfilled') allSchedules = sRes.value;
    loading = false;
  }

  loadAll();

  async function selectPortfolio(id: string) {
    if (selectedId === id) return;
    selectedId    = id;
    detail        = null;
    expandedVectorId   = null;
    expandedScheduleId = null;
    vectorChartData    = null;
    scheduleChartData  = null;
    showVectorPanel    = false;
    showSchedulePanel  = false;
    showPairForm       = false;
    detailLoading = true;
    try {
      detail = await portfolios.get(id);
    } catch (e: any) {
      error = e.message;
    } finally {
      detailLoading = false;
    }
  }

  async function reloadDetail() {
    if (!selectedId) return;
    detail = await portfolios.get(selectedId);
  }

  // ── Formulaire portfolio ─────────────────────────────────────────────────────

  function openCreatePortfolio() {
    editingPortfolioId = null;
    fName = ''; fDesc = '';
    portfolioFormError = null;
    showPortfolioForm = true;
    selectedId = null;
    detail = null;
  }

  function openEditPortfolio(p: PortfolioSummary) {
    editingPortfolioId = p.id;
    fName = p.name;
    fDesc = p.description ?? '';
    portfolioFormError = null;
    showPortfolioForm = true;
  }

  async function savePortfolio() {
    if (!fName.trim()) { portfolioFormError = 'Le nom est requis'; return; }
    portfolioSaving = true;
    portfolioFormError = null;
    try {
      if (editingPortfolioId) {
        await portfolios.update(editingPortfolioId, { name: fName.trim(), description: fDesc || undefined });
      } else {
        const p = await portfolios.create({ name: fName.trim(), description: fDesc || undefined });
        selectedId = p.id;
      }
      await loadAll();
      showPortfolioForm = false;
      if (selectedId) await reloadDetail();
    } catch (e: any) {
      portfolioFormError = e.message;
    } finally {
      portfolioSaving = false;
    }
  }

  async function deletePortfolio(id: string, name: string) {
    if (!confirm(`Supprimer le portfolio "${name}" ?`)) return;
    try {
      await portfolios.delete(id);
      if (selectedId === id) { selectedId = null; detail = null; }
      await loadAll();
    } catch (e: any) { error = e.message; }
  }

  // ── Vecteurs ──────────────────────────────────────────────────────────────────

  function openVectorPanel(mode: 'upload' | 'associate') {
    vectorPanelMode = mode;
    vFile = null; vName = ''; vDesc = ''; vError = null;
    showVectorPanel = true;
  }

  async function uploadVector() {
    if (!vName.trim()) { vError = 'Le nom est requis'; return; }
    if (!vFile)        { vError = 'Sélectionnez un fichier'; return; }
    vUploading = true; vError = null;
    try {
      const form = new FormData();
      form.append('file', vFile);
      form.append('name', vName.trim());
      if (vDesc) form.append('description', vDesc);
      if (selectedId) form.append('portfolio_id', selectedId);
      await outstandingVectors.create(form);
      await Promise.all([reloadDetail(), loadAll()]);
      showVectorPanel = false;
    } catch (e: any) {
      vError = e.message;
    } finally {
      vUploading = false;
    }
  }

  async function associateVector(vectorId: string) {
    if (!selectedId) return;
    try {
      await portfolios.addVector(selectedId, vectorId);
      await reloadDetail();
      showVectorPanel = false;
    } catch (e: any) { vError = e.message; }
  }

  async function removeVector(vectorId: string) {
    if (!selectedId) return;
    await portfolios.removeVector(selectedId, vectorId);
    await reloadDetail();
    if (expandedVectorId === vectorId) { expandedVectorId = null; vectorChartData = null; }
  }

  async function toggleVectorChart(v: VectorSummary) {
    if (expandedVectorId === v.id) {
      expandedVectorId = null; vectorChartData = null;
      if (vchart) { vchart.dispose(); vchart = null; }
      return;
    }
    expandedVectorId = v.id;
    vectorChartData = null;
    try {
      vectorChartData = await outstandingVectors.get(v.id);
    } catch (e: any) { error = e.message; }
  }

  $effect(() => {
    const d = vectorChartData;
    if (!d) return;
    tick().then(() => {
      if (!vectorChartEl) return;
      if (vchart) vchart.dispose();
      vchart = echarts.init(vectorChartEl);

      const obs  = d.rows.filter(r => r.period_type !== 'projected');
      const proj = d.rows.filter(r => r.period_type === 'projected');
      const fmt  = (v: number) => v >= 1e9 ? `${(v/1e9).toFixed(2)} Md` : v >= 1e6 ? `${(v/1e6).toFixed(1)} M` : v.toLocaleString();

      vchart.setOption({
        grid:    { left: 70, right: 20, top: 12, bottom: 36 },
        tooltip: { trigger: 'axis', formatter: (p: any[]) => `${p[0].axisValue}<br/>${fmt(p[0].value)}` },
        xAxis:   { type: 'category', data: d.rows.map(r => r.date), axisLabel: { rotate: 30, fontSize: 10 } },
        yAxis:   { type: 'value', axisLabel: { formatter: (v: number) => fmt(v), fontSize: 10 } },
        series: [
          {
            name: 'Réalisé', type: 'line', data: obs.map(r => [r.date, r.value]),
            areaStyle: { opacity: 0.15, color: '#6366f1' }, lineStyle: { color: '#6366f1', width: 1.5 },
            itemStyle: { color: '#6366f1' }, symbol: 'none',
          },
          {
            name: 'Projection', type: 'line', data: proj.map(r => [r.date, r.value]),
            areaStyle: { opacity: 0.08, color: '#f59e0b' },
            lineStyle: { color: '#f59e0b', width: 1.5, type: [6, 4] },
            itemStyle: { color: '#f59e0b' }, symbol: 'none',
          },
        ],
      });
    });
  });

  // ── Schedules ────────────────────────────────────────────────────────────────

  function openSchedulePanel(mode: 'upload' | 'associate') {
    schedulePanelMode = mode;
    sFile = null; sName = ''; sDesc = ''; sError = null;
    showSchedulePanel = true;
  }

  async function uploadSchedule() {
    if (!sName.trim()) { sError = 'Le nom est requis'; return; }
    if (!sFile)        { sError = 'Sélectionnez un fichier'; return; }
    sUploading = true; sError = null;
    try {
      const form = new FormData();
      form.append('file', sFile);
      form.append('name', sName.trim());
      if (sDesc) form.append('description', sDesc);
      if (selectedId) form.append('portfolio_id', selectedId);
      await amortSchedules.create(form);
      await Promise.all([reloadDetail(), loadAll()]);
      showSchedulePanel = false;
    } catch (e: any) {
      sError = e.message;
    } finally {
      sUploading = false;
    }
  }

  async function associateSchedule(scheduleId: string) {
    if (!selectedId) return;
    try {
      await portfolios.addSchedule(selectedId, scheduleId);
      await reloadDetail();
      showSchedulePanel = false;
    } catch (e: any) { sError = e.message; }
  }

  async function removeSchedule(scheduleId: string) {
    if (!selectedId) return;
    await portfolios.removeSchedule(selectedId, scheduleId);
    await reloadDetail();
    if (expandedScheduleId === scheduleId) { expandedScheduleId = null; scheduleChartData = null; }
  }

  async function toggleScheduleChart(s: ScheduleSummary) {
    if (expandedScheduleId === s.id) {
      expandedScheduleId = null; scheduleChartData = null;
      if (schart) { schart.dispose(); schart = null; }
      return;
    }
    expandedScheduleId = s.id;
    scheduleChartData = null;
    try {
      scheduleChartData = await amortSchedules.get(s.id);
    } catch (e: any) { error = e.message; }
  }

  const BUCKET_COLORS = ['#6366f1','#3b82f6','#06b6d4','#10b981','#84cc16','#f59e0b','#ef4444','#8b5cf6','#ec4899'];

  $effect(() => {
    const d = scheduleChartData;
    if (!d) return;
    tick().then(() => {
      if (!scheduleChartEl) return;
      if (schart) schart.dispose();
      schart = echarts.init(scheduleChartEl);

      schart.setOption({
        grid:    { left: 50, right: 20, top: 12, bottom: 56 },
        tooltip: { trigger: 'axis', axisPointer: { type: 'shadow' } },
        legend:  { bottom: 0, textStyle: { fontSize: 10 }, itemHeight: 8 },
        xAxis:   { type: 'category', data: d.rows.map(r => r.date), axisLabel: { rotate: 30, fontSize: 10 } },
        yAxis:   { type: 'value', max: 1, axisLabel: { formatter: (v: number) => `${(v*100).toFixed(0)}%`, fontSize: 10 } },
        series: d.bucket_labels.map((label, i) => ({
          name: label, type: 'bar', stack: 'total',
          data: d.rows.map(r => +(r.buckets[i] ?? 0).toFixed(4)),
          color: BUCKET_COLORS[i % BUCKET_COLORS.length],
          barMaxWidth: 20,
        })),
      });
    });
  });

  // ── Paires ───────────────────────────────────────────────────────────────────

  async function savePair() {
    if (!pairVectorId)   { pairError = 'Sélectionnez un vecteur'; return; }
    if (!pairScheduleId) { pairError = 'Sélectionnez un schedule'; return; }
    pairSaving = true; pairError = null;
    try {
      await portfolios.createPair(selectedId!, {
        vector_id:   pairVectorId,
        schedule_id: pairScheduleId,
        label:       pairLabel || undefined,
      });
      pairVectorId = ''; pairScheduleId = ''; pairLabel = '';
      showPairForm = false;
      await reloadDetail();
    } catch (e: any) {
      pairError = e.message;
    } finally {
      pairSaving = false;
    }
  }

  async function deletePair(pairId: string) {
    if (!selectedId) return;
    await portfolios.deletePair(selectedId, pairId);
    await reloadDetail();
  }

  // ── Helpers ───────────────────────────────────────────────────────────────────

  /** Vecteurs globaux pas encore dans le portfolio courant */
  function unassociatedVectors(): VectorSummary[] {
    if (!detail) return allVectors;
    const ids = new Set(detail.vectors.map(v => v.id));
    return allVectors.filter(v => !ids.has(v.id));
  }

  function unassociatedSchedules(): ScheduleSummary[] {
    if (!detail) return allSchedules;
    const ids = new Set(detail.schedules.map(s => s.id));
    return allSchedules.filter(s => !ids.has(s.id));
  }
</script>

<div class="tab-content">
  <div class="tab-header">
    <h2>Portfolios</h2>
    <button class="btn-primary" onclick={openCreatePortfolio}>
      <Plus size={15} /> Nouveau portfolio
    </button>
  </div>

  {#if error}
    <div class="alert-error">{error}</div>
  {/if}

  <div class="layout">

    <!-- ── Liste portfolios ──────────────────────────────────────────────────── -->
    <div class="list-panel card">
      {#if loading}
        <p class="loading" style="padding:20px 16px">Chargement…</p>
      {:else if portfolioList.length === 0 && !showPortfolioForm}
        <div class="empty-state">
          <p>Aucun portfolio créé.</p>
          <button class="btn-primary" onclick={openCreatePortfolio}>
            <Plus size={14} /> Créer le premier
          </button>
        </div>
      {:else}
        {#each portfolioList as p}
          <div
            class="list-item"
            class:list-item--active={selectedId === p.id && !showPortfolioForm}
            role="button" tabindex="0"
            onclick={() => { showPortfolioForm = false; selectPortfolio(p.id); }}
            onkeydown={e => e.key === 'Enter' && selectPortfolio(p.id)}
          >
            <div class="list-item-header">
              <span class="list-item-name">{p.name}</span>
            </div>
            <div class="list-item-stats">
              <span>{p.vector_count} vecteur{p.vector_count !== 1 ? 's' : ''}</span>
              <span class="dot">·</span>
              <span>{p.schedule_count} schedule{p.schedule_count !== 1 ? 's' : ''}</span>
              <span class="dot">·</span>
              <span>{p.pair_count} paire{p.pair_count !== 1 ? 's' : ''}</span>
            </div>
            {#if p.description}
              <div class="list-item-desc">{p.description}</div>
            {/if}
            <div class="list-item-actions" onclick={e => e.stopPropagation()}>
              <button class="btn-sm" onclick={() => openEditPortfolio(p)}><Pencil size={12} /></button>
              <button class="btn-sm btn-danger" onclick={() => deletePortfolio(p.id, p.name)}><Trash2 size={12} /></button>
            </div>
          </div>
        {/each}
      {/if}
    </div>

    <!-- ── Panneau droit ─────────────────────────────────────────────────────── -->
    <div class="detail-panel">

      {#if showPortfolioForm}
        <!-- Formulaire portfolio -->
        <div class="card form-card">
          <div class="form-header">
            <h3>{editingPortfolioId ? 'Modifier' : 'Nouveau portfolio'}</h3>
            <button class="icon-btn" onclick={() => showPortfolioForm = false}><X size={16} /></button>
          </div>
          {#if portfolioFormError}<div class="alert-error">{portfolioFormError}</div>{/if}
          <div class="form-grid">
            <label style="grid-column:1/-1">
              Nom *
              <input bind:value={fName} placeholder="Ex. Retail France 2024" />
            </label>
            <label style="grid-column:1/-1">
              Description
              <textarea bind:value={fDesc} rows={2} placeholder="Description optionnelle"></textarea>
            </label>
          </div>
          <div class="form-actions">
            <button class="btn-sm" onclick={() => showPortfolioForm = false}>Annuler</button>
            <button class="btn-primary" onclick={savePortfolio} disabled={portfolioSaving}>
              {portfolioSaving ? 'Enregistrement…' : (editingPortfolioId ? 'Mettre à jour' : 'Créer')}
            </button>
          </div>
        </div>

      {:else if detailLoading}
        <div class="card" style="padding:40px;text-align:center"><p class="loading">Chargement…</p></div>

      {:else if detail}
        <!-- Détail portfolio -->
        <div class="card detail-card">
          <div class="detail-header">
            <div>
              <h3 class="detail-name">{detail.name}</h3>
              {#if detail.description}<p class="detail-desc">{detail.description}</p>{/if}
            </div>
          </div>

          <!-- Onglets -->
          <div class="tabs">
            <button class="tab" class:tab--active={activeTab === 'vectors'}
              onclick={() => activeTab = 'vectors'}>
              Vecteurs <span class="tab-count">{detail.vectors.length}</span>
            </button>
            <button class="tab" class:tab--active={activeTab === 'schedules'}
              onclick={() => activeTab = 'schedules'}>
              Schedules <span class="tab-count">{detail.schedules.length}</span>
            </button>
            <button class="tab" class:tab--active={activeTab === 'pairs'}
              onclick={() => activeTab = 'pairs'}>
              Paires <span class="tab-count">{detail.pairs.length}</span>
            </button>
          </div>

          <!-- ── Tab Vecteurs ──────────────────────────────────────────────── -->
          {#if activeTab === 'vectors'}
            <div class="tab-body">
              <div class="tab-toolbar">
                <button class="btn-primary" onclick={() => openVectorPanel('upload')}>
                  <Plus size={13} /> Uploader
                </button>
                <button class="btn-sm" onclick={() => openVectorPanel('associate')}>
                  <Link size={12} /> Associer existant
                </button>
              </div>

              {#if showVectorPanel}
                <div class="sub-panel">
                  <div class="sub-panel-header">
                    <div class="mode-toggle">
                      <button class="mode-btn" class:mode-btn--active={vectorPanelMode === 'upload'}
                        onclick={() => vectorPanelMode = 'upload'}>Upload</button>
                      <button class="mode-btn" class:mode-btn--active={vectorPanelMode === 'associate'}
                        onclick={() => vectorPanelMode = 'associate'}>Associer existant</button>
                    </div>
                    <button class="icon-btn" onclick={() => showVectorPanel = false}><X size={14} /></button>
                  </div>
                  {#if vError}<div class="alert-error">{vError}</div>{/if}

                  {#if vectorPanelMode === 'upload'}
                    <div class="upload-form">
                      <label>Nom * <input bind:value={vName} placeholder="Nom du vecteur" /></label>
                      <label>Description <input bind:value={vDesc} placeholder="Optionnelle" /></label>
                      <label>
                        Fichier (.xlsx, .ods) *
                        <input type="file" accept=".xlsx,.ods,.xlsm" bind:this={vFileInput}
                          onchange={e => vFile = (e.target as HTMLInputElement).files?.[0] ?? null} />
                      </label>
                      <div class="file-hint">
                        Format attendu : <code>date_month | period_type | value</code>
                      </div>
                      <button class="btn-primary" onclick={uploadVector} disabled={vUploading}>
                        {vUploading ? 'Envoi…' : 'Uploader et associer'}
                      </button>
                    </div>
                  {:else}
                    {@const candidates = unassociatedVectors()}
                    {#if candidates.length === 0}
                      <p class="empty-inline">Tous les vecteurs sont déjà associés.</p>
                    {:else}
                      <div class="associate-list">
                        {#each candidates as v}
                          <div class="associate-row">
                            <div>
                              <div class="assoc-name">{v.name}</div>
                              <div class="assoc-meta">{v.date_from ?? '?'} → {v.date_to ?? '?'} · {v.row_count} lignes</div>
                            </div>
                            <button class="btn-sm btn-success" onclick={() => associateVector(v.id)}>
                              <Link size={11} /> Associer
                            </button>
                          </div>
                        {/each}
                      </div>
                    {/if}
                  {/if}
                </div>
              {/if}

              {#if detail.vectors.length === 0}
                <p class="empty-inline">Aucun vecteur associé à ce portfolio.</p>
              {:else}
                <div class="item-list">
                  {#each detail.vectors as v}
                    <div class="item-row" class:item-row--expanded={expandedVectorId === v.id}>
                      <div class="item-row-main">
                        <div class="item-info">
                          <span class="item-name">{v.name}</span>
                          <span class="item-meta">{v.date_from ?? '?'} → {v.date_to ?? '?'} · {v.row_count} lignes</span>
                        </div>
                        <div class="item-actions">
                          <button class="btn-sm" onclick={() => toggleVectorChart(v)} title="Graphique">
                            {#if expandedVectorId === v.id}<ChevronUp size={12} />{:else}<ChevronDown size={12} />{/if}
                          </button>
                          <button class="btn-sm btn-danger" onclick={() => removeVector(v.id)} title="Dissocier">
                            <X size={11} />
                          </button>
                        </div>
                      </div>
                      {#if expandedVectorId === v.id}
                        <div class="chart-row">
                          {#if !vectorChartData}
                            <p class="loading">Chargement…</p>
                          {:else}
                            <div bind:this={vectorChartEl} style="height:180px;width:100%"></div>
                          {/if}
                        </div>
                      {/if}
                    </div>
                  {/each}
                </div>
              {/if}
            </div>

          <!-- ── Tab Schedules ─────────────────────────────────────────────── -->
          {:else if activeTab === 'schedules'}
            <div class="tab-body">
              <div class="tab-toolbar">
                <button class="btn-primary" onclick={() => openSchedulePanel('upload')}>
                  <Plus size={13} /> Uploader
                </button>
                <button class="btn-sm" onclick={() => openSchedulePanel('associate')}>
                  <Link size={12} /> Associer existant
                </button>
              </div>

              {#if showSchedulePanel}
                <div class="sub-panel">
                  <div class="sub-panel-header">
                    <div class="mode-toggle">
                      <button class="mode-btn" class:mode-btn--active={schedulePanelMode === 'upload'}
                        onclick={() => schedulePanelMode = 'upload'}>Upload</button>
                      <button class="mode-btn" class:mode-btn--active={schedulePanelMode === 'associate'}
                        onclick={() => schedulePanelMode = 'associate'}>Associer existant</button>
                    </div>
                    <button class="icon-btn" onclick={() => showSchedulePanel = false}><X size={14} /></button>
                  </div>
                  {#if sError}<div class="alert-error">{sError}</div>{/if}

                  {#if schedulePanelMode === 'upload'}
                    <div class="upload-form">
                      <label>Nom * <input bind:value={sName} placeholder="Nom du schedule" /></label>
                      <label>Description <input bind:value={sDesc} placeholder="Optionnelle" /></label>
                      <label>
                        Fichier (.xlsx, .ods) *
                        <input type="file" accept=".xlsx,.ods,.xlsm" bind:this={sFileInput}
                          onchange={e => sFile = (e.target as HTMLInputElement).files?.[0] ?? null} />
                      </label>
                      <div class="file-hint">
                        Format attendu : <code>date_month | period_type | bucket_1 | … | bucket_n</code>
                        (somme des buckets ≈ 1.0 par ligne)
                      </div>
                      <button class="btn-primary" onclick={uploadSchedule} disabled={sUploading}>
                        {sUploading ? 'Envoi…' : 'Uploader et associer'}
                      </button>
                    </div>
                  {:else}
                    {@const candidates = unassociatedSchedules()}
                    {#if candidates.length === 0}
                      <p class="empty-inline">Tous les schedules sont déjà associés.</p>
                    {:else}
                      <div class="associate-list">
                        {#each candidates as s}
                          <div class="associate-row">
                            <div>
                              <div class="assoc-name">{s.name}</div>
                              <div class="assoc-meta">
                                {s.date_from ?? '?'} → {s.date_to ?? '?'} ·
                                buckets : {s.bucket_labels.join(', ')}
                              </div>
                            </div>
                            <button class="btn-sm btn-success" onclick={() => associateSchedule(s.id)}>
                              <Link size={11} /> Associer
                            </button>
                          </div>
                        {/each}
                      </div>
                    {/if}
                  {/if}
                </div>
              {/if}

              {#if detail.schedules.length === 0}
                <p class="empty-inline">Aucun schedule associé à ce portfolio.</p>
              {:else}
                <div class="item-list">
                  {#each detail.schedules as s}
                    <div class="item-row" class:item-row--expanded={expandedScheduleId === s.id}>
                      <div class="item-row-main">
                        <div class="item-info">
                          <span class="item-name">{s.name}</span>
                          <span class="item-meta">
                            {s.date_from ?? '?'} → {s.date_to ?? '?'} · {s.row_count} lignes ·
                            <span class="buckets-hint">{s.bucket_labels.join(' | ')}</span>
                          </span>
                        </div>
                        <div class="item-actions">
                          <button class="btn-sm" onclick={() => toggleScheduleChart(s)} title="Graphique">
                            {#if expandedScheduleId === s.id}<ChevronUp size={12} />{:else}<ChevronDown size={12} />{/if}
                          </button>
                          <button class="btn-sm btn-danger" onclick={() => removeSchedule(s.id)} title="Dissocier">
                            <X size={11} />
                          </button>
                        </div>
                      </div>
                      {#if expandedScheduleId === s.id}
                        <div class="chart-row">
                          {#if !scheduleChartData}
                            <p class="loading">Chargement…</p>
                          {:else}
                            <div bind:this={scheduleChartEl} style="height:200px;width:100%"></div>
                          {/if}
                        </div>
                      {/if}
                    </div>
                  {/each}
                </div>
              {/if}
            </div>

          <!-- ── Tab Paires ─────────────────────────────────────────────────── -->
          {:else if activeTab === 'pairs'}
            <div class="tab-body">
              <div class="tab-toolbar">
                <button class="btn-primary" onclick={() => { showPairForm = !showPairForm; pairError = null; }}>
                  <Plus size={13} /> Créer une paire
                </button>
              </div>

              {#if showPairForm}
                <div class="sub-panel">
                  <div class="sub-panel-header">
                    <span style="font-weight:600;font-size:13px">Nouvelle paire</span>
                    <button class="icon-btn" onclick={() => showPairForm = false}><X size={14} /></button>
                  </div>
                  {#if pairError}<div class="alert-error">{pairError}</div>{/if}
                  <div class="pair-form">
                    <label>
                      Vecteur d'outstandings *
                      <select bind:value={pairVectorId}>
                        <option value="">— Choisir —</option>
                        {#each detail.vectors as v}
                          <option value={v.id}>{v.name}</option>
                        {/each}
                      </select>
                    </label>
                    <label>
                      Schedule d'amortissement *
                      <select bind:value={pairScheduleId}>
                        <option value="">— Choisir —</option>
                        {#each detail.schedules as s}
                          <option value={s.id}>{s.name}</option>
                        {/each}
                      </select>
                    </label>
                    <label>
                      Label (optionnel)
                      <input bind:value={pairLabel} placeholder="Ex. Retail EUR — amort. linéaire" />
                    </label>
                    <button class="btn-primary" onclick={savePair} disabled={pairSaving}>
                      {pairSaving ? 'Création…' : 'Créer la paire'}
                    </button>
                  </div>
                </div>
              {/if}

              {#if detail.pairs.length === 0}
                <p class="empty-inline">Aucune paire définie. Ajoutez d'abord des vecteurs et des schedules.</p>
              {:else}
                <table class="pairs-table">
                  <thead>
                    <tr>
                      <th>Vecteur</th>
                      <th>Schedule</th>
                      <th>Label</th>
                      <th></th>
                    </tr>
                  </thead>
                  <tbody>
                    {#each detail.pairs as pair}
                      <tr>
                        <td>{pair.vector_name}</td>
                        <td>{pair.schedule_name}</td>
                        <td class="td-label">{pair.label ?? '—'}</td>
                        <td class="td-action">
                          <button class="btn-sm btn-danger" onclick={() => deletePair(pair.id)}>
                            <Trash2 size={11} />
                          </button>
                        </td>
                      </tr>
                    {/each}
                  </tbody>
                </table>
              {/if}
            </div>
          {/if}
        </div>

      {:else}
        <div class="card empty-detail">
          <Briefcase size={40} color="#d1d5db" />
          <p>Sélectionnez un portfolio<br>ou créez-en un nouveau.</p>
        </div>
      {/if}
    </div>
  </div>
</div>

<style>
  .layout {
    display: grid;
    grid-template-columns: 280px 1fr;
    gap: 20px;
    align-items: start;
  }

  /* ── Liste ── */
  .list-panel { display: flex; flex-direction: column; overflow: hidden; }

  .list-item {
    padding: 11px 14px;
    border-bottom: 1px solid #f3f4f6;
    cursor: pointer;
    transition: background 120ms;
    position: relative;
  }
  .list-item:last-child { border-bottom: none; }
  .list-item:hover { background: #f9fafb; }
  .list-item--active { background: #ede9fe; }
  .list-item--active:hover { background: #ddd6fe; }

  .list-item-header { display: flex; align-items: center; justify-content: space-between; margin-bottom: 2px; }
  .list-item-name { font-size: 13.5px; font-weight: 600; color: #1a1a2e; }
  .list-item-stats { font-size: 11px; color: #9ca3af; display: flex; align-items: center; gap: 4px; }
  .list-item-desc { font-size: 11.5px; color: #6b7280; margin-top: 2px; white-space: nowrap; overflow: hidden; text-overflow: ellipsis; }
  .dot { opacity: .5; }

  .list-item-actions {
    display: none; gap: 4px;
    position: absolute; right: 10px; top: 10px;
  }
  .list-item:hover .list-item-actions { display: flex; }

  /* ── Détail ── */
  .detail-panel { min-width: 0; }
  .form-card, .detail-card { padding: 20px 22px; }

  .form-header { display: flex; align-items: center; justify-content: space-between; margin-bottom: 16px; }
  .form-header h3 { font-size: 15px; font-weight: 700; color: #1a1a2e; }

  .icon-btn { background: none; border: none; cursor: pointer; color: #9ca3af; padding: 4px; border-radius: 6px; display: flex; align-items: center; }
  .icon-btn:hover { background: #f3f4f6; color: #374151; }

  .form-grid { display: grid; grid-template-columns: 1fr 1fr; gap: 10px; margin-bottom: 16px; }
  .form-actions { display: flex; justify-content: flex-end; gap: 8px; padding-top: 14px; border-top: 1px solid #f3f4f6; }

  .detail-header { display: flex; align-items: flex-start; justify-content: space-between; gap: 12px; margin-bottom: 16px; }
  .detail-name { font-size: 17px; font-weight: 700; color: #1a1a2e; }
  .detail-desc { font-size: 12.5px; color: #6b7280; margin-top: 4px; }

  /* ── Onglets ── */
  .tabs { display: flex; gap: 2px; border-bottom: 2px solid #f3f4f6; margin-bottom: 16px; }
  .tab {
    background: none; border: none; cursor: pointer;
    padding: 8px 16px; font-size: 13px; font-weight: 500; color: #6b7280;
    border-bottom: 2px solid transparent; margin-bottom: -2px;
    transition: color 120ms, border-color 120ms;
    display: flex; align-items: center; gap: 6px;
  }
  .tab:hover { color: #374151; }
  .tab--active { color: #6366f1; border-bottom-color: #6366f1; font-weight: 600; }
  .tab-count {
    background: #f3f4f6; color: #6b7280; font-size: 10.5px; font-weight: 700;
    padding: 1px 6px; border-radius: 10px;
  }
  .tab--active .tab-count { background: #e0e7ff; color: #3730a3; }

  .tab-body { }

  /* ── Toolbar ── */
  .tab-toolbar { display: flex; gap: 8px; margin-bottom: 12px; }

  /* ── Sub-panel (upload / associer) ── */
  .sub-panel {
    background: #f9fafb;
    border: 1px solid #e5e7eb;
    border-radius: 10px;
    padding: 14px 16px;
    margin-bottom: 14px;
  }
  .sub-panel-header { display: flex; align-items: center; justify-content: space-between; margin-bottom: 12px; }

  .mode-toggle { display: flex; background: #f1f1f5; border-radius: 7px; padding: 3px; gap: 2px; }
  .mode-btn {
    border: none; background: transparent; border-radius: 5px;
    padding: 4px 12px; font-size: 12px; font-weight: 500; color: #6b7280;
    cursor: pointer; transition: background 120ms, color 120ms;
  }
  .mode-btn--active { background: #fff; color: #4f46e5; box-shadow: 0 1px 3px rgba(0,0,0,.08); font-weight: 600; }

  .upload-form { display: flex; flex-direction: column; gap: 10px; }
  .file-hint { font-size: 11.5px; color: #6b7280; }
  .file-hint code { background: #e5e7eb; padding: 1px 5px; border-radius: 4px; font-size: 11px; }

  /* ── Liste d'items ── */
  .item-list { display: flex; flex-direction: column; gap: 4px; }
  .item-row {
    border: 1px solid #e5e7eb;
    border-radius: 8px;
    overflow: hidden;
  }
  .item-row--expanded { border-color: #c4b5fd; }

  .item-row-main {
    display: flex; align-items: center; justify-content: space-between;
    padding: 9px 12px;
    background: #fff;
  }
  .item-row--expanded .item-row-main { background: #faf5ff; }

  .item-info { display: flex; flex-direction: column; gap: 2px; min-width: 0; }
  .item-name { font-size: 13px; font-weight: 600; color: #1a1a2e; }
  .item-meta { font-size: 11px; color: #9ca3af; }
  .buckets-hint { font-family: monospace; font-size: 10.5px; }

  .item-actions { display: flex; gap: 4px; flex-shrink: 0; }

  .chart-row { padding: 12px; background: #fefefe; border-top: 1px solid #f3f4f6; }

  /* ── Table paires ── */
  .pairs-table { width: 100%; border-collapse: collapse; font-size: 13px; }
  .pairs-table th {
    text-align: left; padding: 7px 12px;
    background: #f9fafb; border-bottom: 1px solid #e5e7eb;
    font-size: 11.5px; font-weight: 600; color: #6b7280; text-transform: uppercase; letter-spacing: .04em;
  }
  .pairs-table td { padding: 9px 12px; border-bottom: 1px solid #f3f4f6; }
  .pairs-table tr:last-child td { border-bottom: none; }
  .pairs-table tr:hover td { background: #fafafa; }
  .td-label { color: #6b7280; font-size: 12px; }
  .td-action { width: 40px; }

  /* ── Pair form ── */
  .pair-form { display: flex; flex-direction: column; gap: 10px; }

  /* ── Associate list ── */
  .associate-list { display: flex; flex-direction: column; gap: 6px; }
  .associate-row {
    display: flex; align-items: center; justify-content: space-between; gap: 12px;
    padding: 8px 10px; background: #fff; border-radius: 6px; border: 1px solid #e5e7eb;
  }
  .assoc-name { font-size: 13px; font-weight: 600; color: #1a1a2e; }
  .assoc-meta { font-size: 11px; color: #9ca3af; }

  .empty-inline { font-size: 12.5px; color: #9ca3af; padding: 8px 0; }

  /* ── Empty state ── */
  .empty-detail {
    display: flex; flex-direction: column; align-items: center;
    justify-content: center; gap: 14px;
    padding: 60px 32px; text-align: center; color: #9ca3af; font-size: 13px;
  }
</style>
