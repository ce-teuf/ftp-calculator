<script lang="ts">
  import { onMount } from 'svelte';
  import { curves, stacks } from '../api/client';
  import type { RateCurve, CurveStack, CurveStackDetail } from '../api/client';
  import { Plus, Trash2, ChevronDown, ChevronUp, Layers, Shuffle } from '@lucide/svelte';

  type View = 'library' | 'builder';
  let view = $state<View>('library');

  // ── Library ────────────────────────────────────────────────────────────────
  let stackList = $state<CurveStack[]>([]);
  let loadingList = $state(true);
  let listError = $state('');

  async function loadList() {
    loadingList = true;
    listError = '';
    try { stackList = await stacks.list(); }
    catch (e: any) { listError = e.message; }
    finally { loadingList = false; }
  }

  async function deleteStack(id: string, name: string) {
    if (!confirm(`Delete the stack "${name}"?`)) return;
    await stacks.delete(id);
    await loadList();
  }

  // ── Builder state ──────────────────────────────────────────────────────────
  let availableCurves = $state<RateCurve[]>([]);
  let bName = $state('');
  let bDescription = $state('');
  let bMode = $state<'simple' | 'combo'>('simple');

  const INTERP_OPTIONS = [
    { value: 'linear',       label: 'Linear',       hint: 'Linear interpolation between points — fast, standard' },
    { value: 'cubic',        label: 'Cubic Spline',  hint: 'Smooth curve — recommended for swap curves' },
    { value: 'flat_forward', label: 'Flat forward',    hint: 'Constant per segment — short-term money market conventions' },
  ];

  interface ComponentRow {
    label: string;
    curve_id: string;          // used in simple mode
    curve_ids: string[];       // used in combo mode
    weight: number;
    interp_method: string;
  }

  let components = $state<ComponentRow[]>([
    { label: '', curve_id: '', curve_ids: [], weight: 1.0, interp_method: 'linear' },
  ]);

  let saving = $state(false);
  let saveError = $state('');
  let saveSuccess = $state('');

  /**
   * Returns the set of component keys that appear more than once across
   * the current stack builder rows (simple mode only).
   */
  const duplicateComponents = $derived.by(() => {
    if (bMode !== 'simple') return new Set<string>();
    const counts = new Map<string, number>();
    for (const row of components) {
      if (!row.curve_id) continue;
      const curve = availableCurves.find(c => c.id === row.curve_id);
      if (!curve?.component) continue;
      counts.set(curve.component, (counts.get(curve.component) ?? 0) + 1);
    }
    return new Set([...counts.entries()].filter(([, n]) => n > 1).map(([k]) => k));
  });

  function componentForRow(row: ComponentRow): string | undefined {
    return availableCurves.find(c => c.id === row.curve_id)?.component;
  }

  function addComponent() {
    components = [...components, { label: '', curve_id: '', curve_ids: [], weight: 1.0, interp_method: 'linear' }];
  }

  function removeComponent(i: number) {
    components = components.filter((_, idx) => idx !== i);
  }

  function moveUp(i: number) {
    if (i === 0) return;
    const arr = [...components];
    [arr[i - 1], arr[i]] = [arr[i], arr[i - 1]];
    components = arr;
  }

  function toggleCurveInCombo(i: number, curveId: string) {
    const c = components[i];
    const has = c.curve_ids.includes(curveId);
    components[i] = {
      ...c,
      curve_ids: has ? c.curve_ids.filter(id => id !== curveId) : [...c.curve_ids, curveId],
    };
  }

  async function save() {
    saveError = '';
    saveSuccess = '';
    if (!bName.trim()) { saveError = 'Name required'; return; }

    saving = true;
    try {
      if (bMode === 'simple') {
        const body = {
          name: bName.trim(),
          description: bDescription || undefined,
          components: components.map(c => ({
            label: c.label || `Component ${components.indexOf(c) + 1}`,
            curve_id: c.curve_id,
            weight: c.weight,
            interp_method: c.interp_method,
          })),
        };
        await stacks.create(body);
        saveSuccess = 'Stack created successfully.';
      } else {
        const body = {
          name_prefix: bName.trim(),
          description: bDescription || undefined,
          components: components.map(c => ({
            label: c.label || `Component ${components.indexOf(c) + 1}`,
            curve_ids: c.curve_ids,
          })),
        };
        const res = await stacks.generateCombinations(body);
        saveSuccess = `${res.created} stack(s) generated.`;
      }
      await loadList();
      view = 'library';
    } catch (e: any) {
      saveError = e.message;
    } finally {
      saving = false;
    }
  }

  // ── Detail view ─────────────────────────────────────────────────────────────
  let selectedStack = $state<CurveStackDetail | null>(null);
  let loadingDetail = $state(false);

  async function openDetail(id: string) {
    loadingDetail = true;
    selectedStack = null;
    try { selectedStack = await stacks.get(id); }
    catch { selectedStack = null; }
    finally { loadingDetail = false; }
  }

  onMount(async () => {
    loadList();
    try { availableCurves = await curves.list(); } catch { /**/ }
  });
</script>

<div class="tab-content">
  <div class="tab-header">
    <h2>Curve Stacks</h2>
    <div class="header-actions">
      <button
        class="btn-sm"
        class:active={view === 'library'}
        onclick={() => { view = 'library'; selectedStack = null; }}
      >Library ({stackList.length})</button>
      <button
        class="btn-primary"
        onclick={() => { view = 'builder'; saveError = ''; saveSuccess = ''; }}
      >
        <Plus size={14} /> New stack
      </button>
    </div>
  </div>

  <!-- ── Library ────────────────────────────────────────────────────────────── -->
  {#if view === 'library'}
    {#if listError}
      <div class="alert-error">{listError}</div>
    {/if}

    {#if loadingList}
      <p class="loading">Loading…</p>
    {:else if stackList.length === 0}
      <div class="empty-state">
        <p>No stack defined.</p>
        <p>A stack groups several additive curve components to form an FTP rate.</p>
        <button class="btn-primary" onclick={() => view = 'builder'}>
          <Plus size={14} /> Create a stack
        </button>
      </div>
    {:else}
      <div class="stack-grid">
        {#each stackList as s}
          <div
            class="stack-card"
            class:selected={selectedStack?.id === s.id}
            onclick={() => openDetail(s.id)}
          >
            <div class="sc-header">
              <span class="sc-name">{s.name}</span>
              <span class="badge badge-{s.status}">{s.status}</span>
            </div>
            <div class="sc-meta">
              <span class="sc-count"><Layers size={12} /> {s.component_count} component{s.component_count !== 1 ? 's' : ''}</span>
              <span class="sc-date">{s.created_at.slice(0, 10)}</span>
            </div>
            {#if s.description}
              <div class="sc-desc">{s.description}</div>
            {/if}
            <div class="sc-actions" onclick={e => e.stopPropagation()}>
              <button class="btn-sm btn-danger" onclick={() => deleteStack(s.id, s.name)}>
                <Trash2 size={12} />
              </button>
            </div>
          </div>
        {/each}
      </div>

      <!-- Detail panel -->
      {#if loadingDetail}
        <p class="loading" style="margin-top:16px">Loading details…</p>
      {:else if selectedStack}
        <div class="detail-panel card">
          <div class="dp-header">
            <strong>{selectedStack.name}</strong>
            <span class="badge badge-{selectedStack.status}">{selectedStack.status}</span>
          </div>
          {#if selectedStack.description}
            <p class="dp-desc">{selectedStack.description}</p>
          {/if}
          <table class="comp-table">
            <thead>
              <tr><th>#</th><th>Label</th><th>Curve</th><th>Interpolation</th><th>Weight</th></tr>
            </thead>
            <tbody>
              {#each selectedStack.components as comp}
                <tr>
                  <td class="pos">{comp.position + 1}</td>
                  <td>{comp.label}</td>
                  <td>
                    <span class="curve-pill">{comp.curve_name ?? comp.curve_id}</span>
                    {#if comp.curve_component}
                      <span class="tag">{comp.curve_component}</span>
                    {/if}
                  </td>
                  <td>
                    <span class="interp-badge interp-{comp.interp_method ?? 'linear'}">
                      {comp.interp_method ?? 'linear'}
                    </span>
                  </td>
                  <td>{comp.weight}×</td>
                </tr>
              {/each}
            </tbody>
          </table>
        </div>
      {/if}
    {/if}

  <!-- ── Builder ─────────────────────────────────────────────────────────────── -->
  {:else}
    <div class="builder card">
      <div class="builder-meta">
        <label>
          Stack name
          <input bind:value={bName} placeholder="Ex: FTP Base + Liquidity" />
        </label>
        <label>
          Description (optional)
          <input bind:value={bDescription} placeholder="" />
        </label>
        <div class="mode-toggle">
          <button
            class="btn-sm"
            class:active={bMode === 'simple'}
            onclick={() => bMode = 'simple'}
          >
            <Layers size={13} /> Simple stack
          </button>
          <button
            class="btn-sm"
            class:active={bMode === 'combo'}
            onclick={() => bMode = 'combo'}
          >
            <Shuffle size={13} /> Generate combinations
          </button>
        </div>
      </div>

      {#if bMode === 'combo'}
        <div class="combo-hint">
          Select multiple curves per component → the system generates the Cartesian product (all possible flat stacks).
        </div>
      {/if}

      <div class="comp-list">
        {#each components as comp, i}
          <div class="comp-row">
            <div class="comp-pos">{i + 1}</div>
            <div class="comp-fields">
              <input
                bind:value={comp.label}
                placeholder="Label (e.g., Base rate)"
                class="comp-label-input"
              />

              {#if bMode === 'simple'}
                <select bind:value={comp.curve_id}>
                  <option value="">-- Choose a curve --</option>
                  {#each availableCurves as c}
                    <option value={c.id}>{c.name} ({c.component} / {c.currency})</option>
                  {/each}
                </select>
                {#if comp.curve_id}
                  {@const compKey = componentForRow(comp)}
                  {#if compKey && duplicateComponents.has(compKey)}
                    <div class="dup-warn">
                      ⚠ Component <strong>{compKey}</strong> already present in this stack — economic logic to verify
                    </div>
                  {/if}
                {/if}
                <input
                  type="number"
                  bind:value={comp.weight}
                  min="0"
                  step="0.1"
                  title="Weight"
                  class="weight-input"
                />
                <select bind:value={comp.interp_method} class="interp-select" title="Interpolation method">
                  {#each INTERP_OPTIONS as opt}
                    <option value={opt.value}>{opt.label}</option>
                  {/each}
                </select>
              {:else}
                <!-- Multi-select for combinations -->
                <div class="multi-select">
                  {#each availableCurves as c}
                    <label class="curve-check">
                      <input
                        type="checkbox"
                        checked={comp.curve_ids.includes(c.id)}
                        onchange={() => toggleCurveInCombo(i, c.id)}
                        style="width:auto"
                      />
                      {c.name} <span class="tag">{c.component}</span>
                    </label>
                  {/each}
                  {#if availableCurves.length === 0}
                    <span class="loading">No curve available</span>
                  {/if}
                </div>
                <div class="multi-count">
                  {comp.curve_ids.length} selected{comp.curve_ids.length !== 1 ? 's' : ''}
                </div>
                <select bind:value={comp.interp_method} class="interp-select" title="Interpolation method">
                  {#each INTERP_OPTIONS as opt}
                    <option value={opt.value}>{opt.label}</option>
                  {/each}
                </select>
              {/if}
            </div>

            <div class="comp-actions">
              <button class="btn-sm" onclick={() => moveUp(i)} disabled={i === 0} title="Move up">
                <ChevronUp size={13} />
              </button>
              <button class="btn-sm btn-danger" onclick={() => removeComponent(i)}>
                <Trash2 size={13} />
              </button>
            </div>
          </div>
        {/each}
      </div>

      <button class="btn-sm add-comp-btn" onclick={addComponent}>
        <Plus size={13} /> Add a component
      </button>

      {#if saveError}<div class="alert-error">{saveError}</div>{/if}
      {#if saveSuccess}<div class="alert-success">{saveSuccess}</div>{/if}

      <div class="builder-footer">
        <button class="btn-sm" onclick={() => view = 'library'}>Cancel</button>
        <button class="btn-primary" onclick={save} disabled={saving}>
          {#if bMode === 'simple'}
            {saving ? 'Saving…' : 'Save the stack'}
          {:else}
            {saving ? 'Generating…' : 'Generate combinations'}
          {/if}
        </button>
      </div>
    </div>
  {/if}
</div>

<style>
  .header-actions { display: flex; gap: 8px; align-items: center; }
  .btn-sm.active { background: #e0e7ff; color: #3730a3; }

  /* ── Stack grid ────────────────────────────────────────────────────────────── */
  .stack-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(260px, 1fr));
    gap: 14px;
    margin-bottom: 24px;
  }

  .stack-card {
    background: #fff;
    border: 1px solid #e5e7eb;
    border-radius: 12px;
    padding: 16px;
    cursor: pointer;
    transition: border-color 150ms, box-shadow 150ms;
    position: relative;
  }
  .stack-card:hover { border-color: #6366f1; box-shadow: 0 0 0 3px rgba(99,102,241,.08); }
  .stack-card.selected { border-color: #6366f1; box-shadow: 0 0 0 3px rgba(99,102,241,.14); }

  .sc-header { display: flex; justify-content: space-between; align-items: flex-start; gap: 8px; margin-bottom: 8px; }
  .sc-name { font-weight: 600; font-size: 14px; color: #1a1a2e; }
  .sc-meta { display: flex; justify-content: space-between; font-size: 12px; color: #9ca3af; margin-bottom: 4px; }
  .sc-count { display: flex; align-items: center; gap: 4px; }
  .sc-desc { font-size: 12px; color: #6b7280; margin-top: 4px; }
  .sc-actions { position: absolute; bottom: 12px; right: 12px; opacity: 0; transition: opacity 150ms; }
  .stack-card:hover .sc-actions { opacity: 1; }

  /* ── Detail panel ──────────────────────────────────────────────────────────── */
  .detail-panel { padding: 20px; }
  .dp-header { display: flex; align-items: center; gap: 10px; margin-bottom: 8px; font-size: 15px; }
  .dp-desc { font-size: 13px; color: #6b7280; margin-bottom: 12px; }

  .comp-table { width: 100%; border-collapse: collapse; font-size: 13px; }
  .comp-table th { text-align: left; padding: 6px 10px; font-weight: 600; color: #6b7280; border-bottom: 1px solid #e5e7eb; }
  .comp-table td { padding: 7px 10px; border-bottom: 1px solid #f3f4f6; }
  .comp-table .pos { color: #9ca3af; width: 30px; }
  .curve-pill { background: #f3f4f6; padding: 2px 8px; border-radius: 6px; font-size: 12px; margin-right: 4px; }

  /* ── Builder ───────────────────────────────────────────────────────────────── */
  .builder { padding: 24px; }
  .builder-meta { display: grid; grid-template-columns: 1fr 1fr; gap: 12px; margin-bottom: 16px; }
  .mode-toggle { grid-column: 1 / -1; display: flex; gap: 8px; }

  .combo-hint {
    background: #ede9fe;
    color: #5b21b6;
    padding: 10px 14px;
    border-radius: 8px;
    font-size: 12.5px;
    margin-bottom: 14px;
  }

  .comp-list { display: flex; flex-direction: column; gap: 10px; margin-bottom: 12px; }

  .comp-row {
    display: flex;
    align-items: flex-start;
    gap: 10px;
    background: #f9f9fb;
    border: 1px solid #e5e7eb;
    border-radius: 10px;
    padding: 12px;
  }

  .comp-pos {
    width: 24px;
    height: 24px;
    background: #6366f1;
    color: #fff;
    border-radius: 50%;
    font-size: 12px;
    font-weight: 700;
    display: flex;
    align-items: center;
    justify-content: center;
    flex-shrink: 0;
    margin-top: 4px;
  }

  .comp-fields { flex: 1; display: flex; flex-wrap: wrap; gap: 8px; }
  .comp-label-input { flex: 1; min-width: 160px; }
  .weight-input { width: 70px; flex-shrink: 0; }
  .interp-select { width: 130px; flex-shrink: 0; font-size: 12px; }
  .comp-actions { display: flex; flex-direction: column; gap: 4px; flex-shrink: 0; }

  .multi-select {
    background: #fff;
    border: 1px solid #e5e7eb;
    border-radius: 8px;
    padding: 8px;
    max-height: 160px;
    overflow-y: auto;
    width: 100%;
    display: flex;
    flex-direction: column;
    gap: 4px;
  }
  .curve-check {
    display: flex;
    flex-direction: row;
    align-items: center;
    gap: 6px;
    font-size: 12.5px;
    cursor: pointer;
    padding: 2px 4px;
    border-radius: 4px;
  }
  .curve-check:hover { background: #f3f4f6; }
  .multi-count { font-size: 12px; color: #6b7280; }

  .dup-warn {
    width: 100%;
    background: #fef3c7;
    color: #92400e;
    border: 1px solid #fcd34d;
    border-radius: 6px;
    padding: 5px 10px;
    font-size: 12px;
    margin-top: 4px;
  }

  .add-comp-btn { margin-bottom: 16px; }

  .builder-footer { display: flex; justify-content: flex-end; gap: 10px; padding-top: 16px; border-top: 1px solid #e5e7eb; }

  .alert-success {
    background: #d1fae5;
    color: #065f46;
    padding: 10px 14px;
    border-radius: 8px;
    margin-bottom: 10px;
    font-size: 13px;
    border-left: 3px solid #10b981;
  }
</style>
