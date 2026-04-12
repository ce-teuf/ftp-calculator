<script lang="ts">
  import { onMount } from 'svelte';
  import { linkers, portfoliosV3, cubes } from '../api/client';
  import type { Linker, PortfolioV3, CurveCube } from '../api/client';
  import { Plus, Trash2, Link, Briefcase, Box } from '@lucide/svelte';

  type View = 'list' | 'builder';
  let view = $state<View>('list');

  let linkerList    = $state<Linker[]>([]);
  let portfolioList = $state<PortfolioV3[]>([]);
  let cubeList      = $state<CurveCube[]>([]);
  let loading       = $state(true);
  let error         = $state('');
  let selected      = $state<Linker | null>(null);

  // ── Builder ─────────────────────────────────────────────────────────────────
  let bName        = $state('');
  let bPortfolioId = $state('');
  let bCubeId      = $state('');
  let bStartDate   = $state('');
  let saving       = $state(false);
  let saveError    = $state('');

  // Derived: selected cube object (to check include_proj)
  let selectedCube = $derived(cubeList.find(c => c.id === bCubeId) ?? null);

  async function load() {
    loading = true; error = '';
    try {
      [linkerList, portfolioList, cubeList] = await Promise.all([
        linkers.list(),
        portfoliosV3.list(),
        cubes.list(),
      ]);
    } catch (e: any) { error = e.message; }
    finally { loading = false; }
  }

  async function deleteLinker(id: string, name: string) {
    if (!confirm(`Delete the linker "${name}"?`)) return;
    await linkers.delete(id);
    if (selected?.id === id) selected = null;
    await load();
  }

  function openBuilder() {
    bName = ''; bPortfolioId = ''; bCubeId = ''; bStartDate = '';
    saveError = '';
    view = 'builder';
  }

  async function save() {
    saveError = '';
    if (!bName.trim())    { saveError = 'Name required';          return; }
    if (!bPortfolioId)    { saveError = 'Portfolio required';    return; }
    if (!bCubeId)         { saveError = 'Cube required';         return; }
    if (!bStartDate)      { saveError = 'Start date required'; return; }

    saving = true;
    try {
      await linkers.create({
        name: bName.trim(),
        portfolio_id: bPortfolioId,
        cube_id: bCubeId,
        start_date: bStartDate,
      });
      await load();
      view = 'list';
    } catch (e: any) { saveError = e.message; }
    finally { saving = false; }
  }

  onMount(load);
</script>

<div class="tab-content">
  <div class="tab-header">
    <h2>Linkers</h2>
    <div style="display:flex;gap:8px">
      {#if view === 'builder'}
        <button class="btn-sm" onclick={() => view = 'list'}>← Back</button>
      {:else}
        <button class="btn-primary" onclick={openBuilder}>
          <Plus size={14}/> New linker
        </button>
      {/if}
    </div>
  </div>

  {#if error}<div class="alert-error">{error}</div>{/if}

  <!-- ── List ──────────────────────────────────────────────────────────────── -->
  {#if view === 'list'}
    {#if loading}
<p class="loading">Loading…</p>
      {:else if linkerList.length === 0}
        <div class="empty-state">
          <p>No linker defined.</p>
          <p>A linker associates a portfolio with a cube and defines the start date of the analysis.</p>
          <button class="btn-primary" onclick={openBuilder}><Plus size={14}/> Create</button>
      </div>
    {:else}
      <div class="linker-grid">
        {#each linkerList as l}
          <div
            class="linker-card"
            class:selected={selected?.id === l.id}
            onclick={() => selected = selected?.id === l.id ? null : l}
          >
            <div class="lc-name">{l.name}</div>

            <div class="lc-row">
              <span class="lc-icon"><Briefcase size={12}/></span>
              <span class="lc-val">{l.portfolio_name ?? l.portfolio_id}</span>
            </div>
            <div class="lc-row">
              <span class="lc-icon"><Box size={12}/></span>
              <span class="lc-val">{l.cube_name ?? l.cube_id}</span>
            </div>
            <div class="lc-row">
              <span class="lc-icon"><Link size={12}/></span>
              <span class="lc-val">Start: {l.start_date}</span>
            </div>

            <div class="lc-actions" onclick={e => e.stopPropagation()}>
              <button class="btn-sm btn-danger" onclick={() => deleteLinker(l.id, l.name)}>
                <Trash2 size={12}/>
              </button>
            </div>
          </div>
        {/each}
      </div>

      <!-- Detail -->
      {#if selected}
        <div class="detail-panel card">
          <div class="dp-header">
            <Link size={16}/>
            <strong>{selected.name}</strong>
          </div>
          <div class="dp-grid">
            <div class="dp-item">
              <span class="dp-lbl">Portfolio</span>
              <span class="dp-val">{selected.portfolio_name ?? selected.portfolio_id}</span>
            </div>
            <div class="dp-item">
              <span class="dp-lbl">Cube</span>
              <span class="dp-val">{selected.cube_name ?? selected.cube_id}</span>
            </div>
            <div class="dp-item">
              <span class="dp-lbl">Start date</span>
              <span class="dp-val">{selected.start_date}</span>
            </div>
            <div class="dp-item">
              <span class="dp-lbl">Created</span>
              <span class="dp-val">{selected.created_at.slice(0,10)}</span>
            </div>
          </div>
        </div>
      {/if}
    {/if}

  <!-- ── Builder ─────────────────────────────────────────────────────────────── -->
  {:else}
    <div class="builder card">
      <div class="builder-grid">

        <label style="grid-column:1/-1">
          Linker name
          <input bind:value={bName} placeholder="Ex: Real estate credits × ESTR Cube 2025"/>
        </label>

        <label>
          Portfolio
          <select bind:value={bPortfolioId}>
            <option value="">-- Choose a portfolio --</option>
            {#each portfolioList as p}
              <option value={p.id}>
                {p.name}
                ({p.schedule_type === 'stock_amort' ? 'Stock' : 'New Prod'}
                · {p.row_count} profil{p.row_count !== 1 ? 's' : ''})
              </option>
            {/each}
          </select>
          {#if portfolioList.length === 0}
            <span class="hint-warn">No portfolio — create one in the Portfolios tab.</span>
          {/if}
        </label>

        <label>
          Cube
          <select bind:value={bCubeId}>
            <option value="">-- Choose a cube --</option>
            {#each cubeList as c}
              <option value={c.id}>
                {c.name} ({c.n_analysis_times} dates · pas {c.step_months}M)
              </option>
            {/each}
          </select>
          {#if cubeList.length === 0}
            <span class="hint-warn">No cube — create one in the Cubes tab.</span>
          {/if}
        </label>

        <label>
          Date de début de l'analyse
          <input type="date" bind:value={bStartDate}/>
          <span class="hint">Premier point dans la dimension temporelle du cube.</span>
        </label>

        <!-- Summary box -->
        {#if bPortfolioId && bCubeId && bStartDate}
          {@const pf = portfolioList.find(p => p.id === bPortfolioId)}
          {@const cb = cubeList.find(c => c.id === bCubeId)}
          {#if pf && cb}
            <div class="summary-box" style="grid-column:1/-1">
              <div class="sb-row">
                <Briefcase size={14}/>
                <strong>{pf.name}</strong>
                <span class="tag">{pf.schedule_type === 'stock_amort' ? 'Stock amort' : 'New Prod'}</span>
                <span class="sb-dot">·</span>
                {pf.row_count} profil{pf.row_count !== 1 ? 's' : ''}
              </div>
              <div class="sb-arrow">↕</div>
              <div class="sb-row">
                <Box size={14}/>
                <strong>{cb.name}</strong>
                <span class="tag">{cb.n_analysis_times} dates</span>
                <span class="sb-dot">·</span>
                stack : {cb.stack_name ?? cb.stack_id}
                {#if cb.include_proj}<span class="tag tag-purple">proj.</span>{/if}
                {#if cb.mc_scenarios > 0}<span class="tag tag-orange">MC×{cb.mc_scenarios}</span>{/if}
              </div>
              <div class="sb-start">Début : <strong>{bStartDate}</strong></div>
            </div>
          {/if}
        {/if}

      </div>

      {#if saveError}<div class="alert-error">{saveError}</div>{/if}

      <div class="builder-footer">
        <button class="btn-sm" onclick={() => view = 'list'}>Annuler</button>
        <button class="btn-primary" onclick={save} disabled={saving}>
          {saving ? 'Création…' : 'Créer le linker'}
        </button>
      </div>
    </div>
  {/if}
</div>

<style>
  .linker-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(260px,1fr));
    gap: 14px;
    margin-bottom: 24px;
  }

  .linker-card {
    background: #fff;
    border: 1px solid #e5e7eb;
    border-radius: 12px;
    padding: 16px;
    cursor: pointer;
    position: relative;
    transition: border-color 150ms, box-shadow 150ms;
  }
  .linker-card:hover    { border-color: #6366f1; box-shadow: 0 0 0 3px rgba(99,102,241,.08); }
  .linker-card.selected { border-color: #6366f1; box-shadow: 0 0 0 3px rgba(99,102,241,.14); }

  .lc-name { font-weight: 700; font-size: 14px; color: #1a1a2e; margin-bottom: 10px; }
  .lc-row  { display: flex; align-items: center; gap: 6px; font-size: 12.5px; color: #6b7280; margin-bottom: 4px; }
  .lc-icon { color: #9ca3af; display: flex; }
  .lc-val  { flex: 1; }
  .lc-actions { position: absolute; bottom: 12px; right: 12px; opacity: 0; transition: opacity 150ms; }
  .linker-card:hover .lc-actions { opacity: 1; }

  /* Detail */
  .detail-panel { padding: 20px; }
  .dp-header { display: flex; align-items: center; gap: 8px; margin-bottom: 14px; font-size: 15px; }
  .dp-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(180px,1fr));
    gap: 10px;
  }
  .dp-item { background: #f9fafb; border-radius: 8px; padding: 10px 12px; }
  .dp-lbl  { display: block; font-size: 11px; color: #9ca3af; font-weight: 600; text-transform: uppercase; letter-spacing: .04em; margin-bottom: 3px; }
  .dp-val  { font-size: 14px; font-weight: 600; color: #1a1a2e; }

  /* Builder */
  .builder { padding: 24px; }
  .builder-grid {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 14px;
    margin-bottom: 16px;
  }
  .builder-footer { display: flex; justify-content: flex-end; gap: 10px; padding-top: 16px; border-top: 1px solid #e5e7eb; }

  .hint      { font-size: 11.5px; color: #9ca3af; display: block; margin-top: 3px; }
  .hint-warn { font-size: 11.5px; color: #d97706; display: block; margin-top: 3px; }

  /* Summary box */
  .summary-box {
    background: #f0fdf4;
    border: 1px solid #bbf7d0;
    border-radius: 10px;
    padding: 14px 16px;
    display: flex;
    flex-direction: column;
    gap: 6px;
    font-size: 13px;
  }
  .sb-row   { display: flex; align-items: center; gap: 8px; flex-wrap: wrap; color: #1a1a2e; }
  .sb-arrow { text-align: center; color: #9ca3af; font-size: 16px; }
  .sb-start { font-size: 12px; color: #6b7280; padding-top: 4px; border-top: 1px solid #d1fae5; }
  .sb-dot   { color: #9ca3af; }

  .tag-purple { background: #ede9fe; color: #5b21b6; }
  .tag-orange { background: #fff7ed; color: #c2410c; }
</style>
