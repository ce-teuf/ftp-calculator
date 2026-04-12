<script lang="ts">
  import { onMount, tick } from 'svelte';
  import { riskTypes, rateMatrices } from '$lib/api/client';
  import type { RateMatrixSummary, RateMatrixDetail, RiskType } from '$lib/api/client';
  import * as echarts from 'echarts';
  import { Upload, Trash2, Eye, Pencil, X, Plus, ChevronDown, ChevronUp } from '@lucide/svelte';

  // ── État global ───────────────────────────────────────────────────────────────

  let matrices  = $state<RateMatrixSummary[]>([]);
  let allRisks  = $state<RiskType[]>([]);
  let loading   = $state(true);
  let error     = $state<string | null>(null);

  // Panneau d'upload
  let showUpload   = $state(false);
  let uploading    = $state(false);
  let uploadError  = $state<string | null>(null);

  // Formulaire d'upload
  let uName         = $state('');
  let uDesc         = $state('');
  let uCurrency     = $state('EUR');
  let uInterp       = $state<'linear' | 'cubic' | 'flat_forward'>('linear');
  let uStatus       = $state<'draft' | 'active'>('draft');
  let uRisks        = $state<Set<string>>(new Set());
  let uFile         = $state<File | null>(null);
  let uFileInput:     HTMLInputElement;

  // Panneau de détail
  let selectedId   = $state<string | null>(null);
  let detail       = $state<RateMatrixDetail | null>(null);
  let detailLoading = $state(false);

  // Panneau d'édition
  let showEdit     = $state(false);
  let editRisks    = $state<Set<string>>(new Set());
  let editStatus   = $state('');
  let editInterp   = $state('');
  let saving       = $state(false);

  // Chart
  let chartEl: HTMLDivElement;
  let chart: echarts.ECharts | null = null;
  let chartView = $state<'tenor' | 'fan'>('tenor');

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

  function riskLabel(key: string): string {
    return allRisks.find(r => r.key === key)?.label ?? key;
  }

  // ── Chargement initial ────────────────────────────────────────────────────────

  onMount(async () => {
    try {
      [matrices, allRisks] = await Promise.all([
        rateMatrices.list(),
        riskTypes.list(),
      ]);
    } catch (e: any) {
      error = e.message;
    } finally {
      loading = false;
    }
  });

  // ── Sélection / détail ────────────────────────────────────────────────────────

  async function selectMatrix(id: string) {
    // Détruire l'instance ECharts pour éviter les références à un nœud DOM retiré
    if (chart) { chart.dispose(); chart = null; }

    if (selectedId === id) {
      selectedId = null;
      detail = null;
      return;
    }
    selectedId = id;
    detailLoading = true;
    showEdit = false;
    try {
      detail = await rateMatrices.get(id);
    } catch (e: any) {
      error = e.message;
    } finally {
      detailLoading = false;
    }
  }

  // Mettre à jour le chart quand le détail change.
  // On attend un tick pour s'assurer que le DOM est rendu (le panneau est conditionnel).
  $effect(() => {
    const d = detail;
    const v = chartView;
    if (!d) return;
    tick().then(() => {
      if (chartEl) renderChart(d, v);
    });
  });

  // ── Helpers chart (partagés) ──────────────────────────────────────────────────

  function tenorToMonths(t: string): number {
    const m = t.match(/^(\d+)([MY])$/i);
    if (!m) return 0;
    return m[2].toUpperCase() === 'M' ? +m[1] : +m[1] * 12;
  }

  function addMonths(ym: string, months: number): number {
    const [y, mo] = ym.split('-').map(Number);
    const total = y * 12 + (mo - 1) + months;
    return Date.UTC(Math.floor(total / 12), total % 12, 1);
  }

  function fmtTs(ts: number): string {
    const dt = new Date(ts);
    return `${dt.getUTCFullYear()}-${String(dt.getUTCMonth() + 1).padStart(2, '0')}`;
  }

  function curveColor(i: number, n: number): string {
    const t = n > 1 ? i / (n - 1) : 0;
    return `rgb(${Math.round(59 + t * 190)},${Math.round(130 - t * 15)},${Math.round(246 - t * 224)})`;
  }

  // ── Vue "tenor" : X = tenors (catégorie), une courbe par mois ─────────────────

  function buildTenorSeries(d: RateMatrixDetail): any[] {
    const n = d.rows.length;
    const firstProjIdx = d.rows.findIndex(r => r.period_type === 'projected');
    return d.rows.map((row, i) => {
      const isProj = row.period_type === 'projected';
      const color  = curveColor(i, n);
      return {
        name: row.date,
        type: 'line',
        data: row.values.map(v => +(v * 100).toFixed(4)),
        lineStyle: { width: 1.2, type: isProj ? [6, 4] : 'solid', color, opacity: 0.45 },
        itemStyle:  { color },
        symbol: 'circle', symbolSize: 4, showSymbol: false,
        emphasis: {
          focus: 'series',
          lineStyle: { width: 2.5, opacity: 1, type: isProj ? [6, 4] : 'solid' },
          showSymbol: true, symbolSize: 5,
          label: { show: true, formatter: row.date, position: 'insideTopRight' as const,
                   fontSize: 10, color, fontWeight: 'bold' as const },
        },
        blur: { lineStyle: { opacity: 0.06, width: 0.8 }, itemStyle: { opacity: 0 } },
        ...(isProj && i === firstProjIdx
          ? { markArea: { silent: true, itemStyle: { color: 'rgba(251,191,36,0.07)' },
                          label: { show: false },
                          data: [[{ xAxis: d.tenors[0] }, { xAxis: d.tenors[d.tenors.length - 1] }]] } }
          : {}),
      };
    });
  }

  // ── Vue "fan" : X = date calendaire, point de départ = mois d'observation ────

  function buildFanSeries(d: RateMatrixDetail): any[] {
    const n = d.rows.length;
    const firstProjRow = d.rows.find(r => r.period_type === 'projected');
    const projLineTs   = firstProjRow ? addMonths(firstProjRow.date, 0) : null;
    return d.rows.map((row, i) => {
      const isProj = row.period_type === 'projected';
      const color  = curveColor(i, n);
      const data   = d.tenors.map((tenor, ti) => ({
        value:   [addMonths(row.date, tenorToMonths(tenor)), +(row.values[ti] * 100).toFixed(4)],
        tenor,
      }));
      return {
        name: row.date,
        type: 'line',
        data,
        lineStyle: { width: 1.2, type: isProj ? [6, 4] : 'solid', color, opacity: 0.45 },
        itemStyle:  { color },
        symbol: 'circle', symbolSize: 3, showSymbol: false,
        emphasis: {
          focus: 'series',
          lineStyle: { width: 2.5, opacity: 1, type: isProj ? [6, 4] : 'solid' },
          showSymbol: true, symbolSize: 5,
          label: { show: true, formatter: row.date, position: 'insideTopLeft' as const,
                   fontSize: 10, color, fontWeight: 'bold' as const,
                   backgroundColor: 'rgba(255,255,255,0.8)', padding: [2, 4] as any, borderRadius: 3 },
        },
        blur: { lineStyle: { opacity: 0.06, width: 0.8 }, itemStyle: { opacity: 0 } },
        ...(i === 0 && projLineTs
          ? { markLine: { silent: true, symbol: 'none',
                          lineStyle: { color: '#f59e0b', type: [4, 3], width: 1.5 },
                          label: { show: true, formatter: 'Projection →', position: 'insideStartTop', fontSize: 9, color: '#92400e' },
                          data: [{ xAxis: projLineTs }] } }
          : {}),
      };
    });
  }

  // ── Dispatch selon la vue active ──────────────────────────────────────────────

  function renderChart(d: RateMatrixDetail, view: 'tenor' | 'fan' = chartView) {
    if (!chartEl) return;
    if (!chart) { chart = echarts.init(chartEl, undefined, { renderer: 'canvas' }); }
    chart.clear();

    if (view === 'tenor') {
      chart.setOption({
        backgroundColor: 'transparent',
        legend: { show: false },
        tooltip: {
          trigger: 'item',
          formatter: (params: any) => {
            const row    = d.rows.find(r => r.date === params.seriesName);
            const isProj = row?.period_type === 'projected';
            const badge  = isProj
              ? `<span style="background:#fef3c7;color:#92400e;padding:1px 6px;border-radius:4px;font-size:10px">projection</span>`
              : `<span style="background:#dbeafe;color:#1e40af;padding:1px 6px;border-radius:4px;font-size:10px">réalisé</span>`;
            return `<div style="font-size:12px;line-height:1.8"><b>${params.seriesName}</b> ${badge}<br/>Tenor <b>${params.name}</b> : <b style="color:${params.color}">${params.value}%</b></div>`;
          },
        },
        grid: { left: 52, right: 24, top: 20, bottom: 36 },
        xAxis: { type: 'category', data: d.tenors, name: 'Tenor', nameLocation: 'middle', nameGap: 26,
                 axisLabel: { fontSize: 11 }, boundaryGap: false,
                 axisLine: { lineStyle: { color: '#e5e7eb' } }, splitLine: { show: false } },
        yAxis: { type: 'value', name: '%',
                 axisLabel: { formatter: (v: number) => v.toFixed(2) + '%', fontSize: 10 },
                 splitLine: { lineStyle: { color: '#f0f0f5' } } },
        series: buildTenorSeries(d),
      });
    } else {
      chart.setOption({
        backgroundColor: 'transparent',
        legend: { show: false },
        tooltip: {
          trigger: 'item',
          formatter: (params: any) => {
            const row    = d.rows.find(r => r.date === params.seriesName);
            const isProj = row?.period_type === 'projected';
            const badge  = isProj
              ? `<span style="background:#fef3c7;color:#92400e;padding:1px 6px;border-radius:4px;font-size:10px">projection</span>`
              : `<span style="background:#dbeafe;color:#1e40af;padding:1px 6px;border-radius:4px;font-size:10px">réalisé</span>`;
            const maturity = fmtTs(params.value[0]);
            const tenor    = params.data?.tenor ?? '';
            return `<div style="font-size:12px;line-height:1.9">Obs&nbsp;: <b>${params.seriesName}</b> ${badge}<br/>Maturité&nbsp;: <b>${maturity}</b> (${tenor})<br/>Taux&nbsp;: <b style="color:${params.color}">${params.value[1]}%</b></div>`;
          },
        },
        grid: { left: 56, right: 20, top: 16, bottom: 44 },
        xAxis: { type: 'time',
                 axisLabel: { fontSize: 10, formatter: (val: number) => fmtTs(val), rotate: 30 },
                 splitLine: { show: false }, axisLine: { lineStyle: { color: '#e5e7eb' } } },
        yAxis: { type: 'value', name: '%',
                 axisLabel: { formatter: (v: number) => v.toFixed(2) + '%', fontSize: 10 },
                 splitLine: { lineStyle: { color: '#f0f0f5' } } },
        series: buildFanSeries(d),
      });
    }
    chart.resize();
  }

  // ── Upload ────────────────────────────────────────────────────────────────────

  function openUpload() {
    uName = ''; uDesc = ''; uCurrency = 'EUR';
    uInterp = 'linear'; uStatus = 'draft';
    uRisks = new Set(); uFile = null;
    uploadError = null;
    showUpload = true;
  }

  function toggleUploadRisk(key: string) {
    const next = new Set(uRisks);
    if (next.has(key)) next.delete(key); else next.add(key);
    uRisks = next;
  }

  async function submitUpload() {
    if (!uName.trim()) { uploadError = 'Le nom est requis'; return; }
    if (!uFile)        { uploadError = 'Sélectionner un fichier'; return; }

    const form = new FormData();
    form.append('file',          uFile);
    form.append('name',          uName.trim());
    form.append('description',   uDesc);
    form.append('currency',      uCurrency);
    form.append('interp_method', uInterp);
    form.append('status',        uStatus);
    uRisks.forEach(k => form.append('risk_key', k));

    uploading   = true;
    uploadError = null;
    try {
      const created = await rateMatrices.create(form);
      matrices = [
        {
          id:           created.id,
          name:         created.name,
          description:  created.description,
          currency:     created.currency,
          status:       created.status,
          interp_method: created.interp_method,
          tenors:       created.tenors,
          row_count:    created.rows.length,
          date_from:    created.rows[0]?.date,
          date_to:      created.rows[created.rows.length - 1]?.date,
          created_at:   created.created_at,
          risks:        created.risks,
        },
        ...matrices,
      ];
      showUpload = false;
    } catch (e: any) {
      uploadError = e.message;
    } finally {
      uploading = false;
    }
  }

  // ── Suppression ───────────────────────────────────────────────────────────────

  async function deleteMatrix(id: string, name: string) {
    if (!confirm(`Supprimer la matrice "${name}" ?`)) return;
    try {
      await rateMatrices.delete(id);
      matrices = matrices.filter(m => m.id !== id);
      if (selectedId === id) { selectedId = null; detail = null; }
    } catch (e: any) {
      error = e.message;
    }
  }

  // ── Édition ───────────────────────────────────────────────────────────────────

  function openEdit() {
    if (!detail) return;
    editRisks  = new Set(detail.risks);
    editStatus = detail.status;
    editInterp = detail.interp_method;
    showEdit   = true;
  }

  function toggleEditRisk(key: string) {
    const next = new Set(editRisks);
    if (next.has(key)) next.delete(key); else next.add(key);
    editRisks = next;
  }

  async function saveEdit() {
    if (!detail) return;
    saving = true;
    try {
      const updated = await rateMatrices.update(detail.id, {
        status:       editStatus,
        interp_method: editInterp,
        risks:        [...editRisks],
      });
      detail = updated;
      // Mettre à jour la liste
      matrices = matrices.map(m =>
        m.id === updated.id
          ? { ...m, status: updated.status, interp_method: updated.interp_method, risks: updated.risks }
          : m
      );
      showEdit = false;
    } catch (e: any) {
      error = e.message;
    } finally {
      saving = false;
    }
  }

  // ── Helpers d'affichage ───────────────────────────────────────────────────────

  function fmtDate(d: string | undefined): string {
    return d ?? '—';
  }

  function statusClass(s: string): string {
    return `badge badge-${s}`;
  }
</script>

<div class="tab-content">
  <!-- ── En-tête ─────────────────────────────────────────────────────────────── -->
  <div class="tab-header">
    <h2>Matrices de taux</h2>
    <button class="btn-primary" onclick={openUpload}>
      <Plus size={15} /> Nouvelle matrice
    </button>
  </div>

  {#if error}
    <div class="alert-error">{error}</div>
  {/if}

  <!-- ── Liste ──────────────────────────────────────────────────────────────── -->
  {#if loading}
    <p class="loading">Chargement…</p>
  {:else if matrices.length === 0}
    <div class="empty-state">
      <p>Aucune matrice de taux</p>
      <p>Importez un fichier .xlsx ou .ods pour commencer</p>
    </div>
  {:else}
    <div class="card" style="overflow:hidden">
      <table class="data-table">
        <thead>
          <tr>
            <th>Nom</th>
            <th>Statut</th>
            <th>Devise</th>
            <th>Risques</th>
            <th>Période</th>
            <th>Lignes</th>
            <th>Tenors</th>
            <th>Interpolation</th>
            <th></th>
          </tr>
        </thead>
        <tbody>
          {#each matrices as m (m.id)}
            <tr
              class:row-selected={selectedId === m.id}
              onclick={() => selectMatrix(m.id)}
              style="cursor:pointer"
            >
              <td class="name-cell">
                <span class="matrix-name">{m.name}</span>
                {#if m.description}
                  <span class="matrix-desc">{m.description}</span>
                {/if}
              </td>
              <td><span class={statusClass(m.status)}>{m.status}</span></td>
              <td><span class="currency">{m.currency ?? '—'}</span></td>
              <td>
                <div class="risk-badges">
                  {#each m.risks as rk}
                    <span class="risk-badge" style="background:{riskColor(rk)}22;color:{riskColor(rk)};border:1px solid {riskColor(rk)}44">
                      {riskLabel(rk)}
                    </span>
                  {/each}
                  {#if m.risks.length === 0}
                    <span style="color:#9ca3af;font-size:11px">—</span>
                  {/if}
                </div>
              </td>
              <td class="dates-cell">
                <span>{fmtDate(m.date_from)}</span>
                {#if m.date_to && m.date_to !== m.date_from}
                  <span class="date-arrow">→</span>
                  <span>{fmtDate(m.date_to)}</span>
                {/if}
              </td>
              <td style="text-align:right">{m.row_count}</td>
              <td class="tenors-cell">{m.tenors.join(', ')}</td>
              <td><code class="interp-tag">{m.interp_method}</code></td>
              <td>
                <div class="row-actions" onclick={(e) => e.stopPropagation()}>
                  <button class="btn-sm" onclick={() => selectMatrix(m.id)} title="Détail">
                    <Eye size={13} />
                  </button>
                  <button class="btn-sm btn-danger" onclick={() => deleteMatrix(m.id, m.name)} title="Supprimer">
                    <Trash2 size={13} />
                  </button>
                </div>
              </td>
            </tr>
          {/each}
        </tbody>
      </table>
    </div>
  {/if}

  <!-- ── Panneau de détail ──────────────────────────────────────────────────── -->
  {#if selectedId}
    <div class="detail-panel card" style="margin-top:24px">
      {#if detailLoading}
        <div style="padding:24px"><p class="loading">Chargement du détail…</p></div>
      {:else if detail}
        <!-- En-tête détail -->
        <div class="detail-header">
          <div>
            <h3>{detail.name}</h3>
            {#if detail.description}
              <p class="detail-desc">{detail.description}</p>
            {/if}
            <div class="detail-meta">
              <span class={statusClass(detail.status)}>{detail.status}</span>
              <span class="meta-pill">{detail.currency ?? '—'}</span>
              <code class="interp-tag">{detail.interp_method}</code>
              {#each detail.risks as rk}
                <span class="risk-badge" style="background:{riskColor(rk)}22;color:{riskColor(rk)};border:1px solid {riskColor(rk)}44">
                  {riskLabel(rk)}
                </span>
              {/each}
              {#if detail.risks.length > 1}
                <span class="warn-multi-risk">⚠ Risques indissociables</span>
              {/if}
            </div>
          </div>
          <div style="display:flex;gap:8px">
            <button class="btn-sm" onclick={openEdit}>
              <Pencil size={12} /> Modifier
            </button>
            <button class="btn-sm" onclick={() => { selectedId = null; detail = null; }}>
              <X size={12} /> Fermer
            </button>
          </div>
        </div>

        <!-- Graphique -->
        <div class="chart-wrapper">
          <div class="chart-toolbar">
            <div class="chart-legend-hint">
              <span class="hint-solid">━</span> réalisé &nbsp;
              <span class="hint-dash">╌╌</span> projection &nbsp;
              <span style="color:#9ca3af">· bleu = plus ancienne &nbsp; orange = plus récente</span>
            </div>
            <div class="chart-view-toggle">
              <button
                class="toggle-btn"
                class:toggle-btn--active={chartView === 'tenor'}
                onclick={() => { chartView = 'tenor'; }}
                title="Vue par tenor : X = tenors, une courbe par mois"
              >Tenors</button>
              <button
                class="toggle-btn"
                class:toggle-btn--active={chartView === 'fan'}
                onclick={() => { chartView = 'fan'; }}
                title="Vue éventail : X = date calendaire, une courbe par mois d'observation"
              >Éventail</button>
            </div>
          </div>
          <div bind:this={chartEl} class="chart-canvas"></div>
        </div>

        <!-- Éditeur inline -->
        {#if showEdit}
          <div class="edit-section">
            <h4>Modifier la matrice</h4>
            <div class="edit-grid">
              <label>
                Statut
                <select bind:value={editStatus}>
                  <option value="draft">draft</option>
                  <option value="active">active</option>
                  <option value="archived">archived</option>
                </select>
              </label>
              <label>
                Interpolation
                <select bind:value={editInterp}>
                  <option value="linear">linear</option>
                  <option value="cubic">cubic</option>
                  <option value="flat_forward">flat_forward</option>
                </select>
              </label>
            </div>
            <div class="risks-section">
              <div class="risks-label">Types de risque</div>
              <div class="risks-grid">
                {#each allRisks as r}
                  <label class="risk-checkbox">
                    <input
                      type="checkbox"
                      checked={editRisks.has(r.key)}
                      onchange={() => toggleEditRisk(r.key)}
                    />
                    <span class="risk-badge" style="background:{riskColor(r.key)}22;color:{riskColor(r.key)};border:1px solid {riskColor(r.key)}44">
                      {r.label}
                    </span>
                  </label>
                {/each}
              </div>
              {#if editRisks.size > 1}
                <p class="warn-text">⚠ Plusieurs risques attribués → ils seront indissociables dans les analyses</p>
              {/if}
            </div>
            <div style="display:flex;gap:8px;margin-top:12px">
              <button class="btn-primary" onclick={saveEdit} disabled={saving}>
                {saving ? 'Enregistrement…' : 'Enregistrer'}
              </button>
              <button class="btn-sm" onclick={() => showEdit = false}>Annuler</button>
            </div>
          </div>
        {/if}

        <!-- Tableau des données brutes -->
        <div class="raw-data-section">
          <details>
            <summary class="raw-data-summary">
              Données brutes ({detail.rows.length} lignes · tenors : {detail.tenors.join(', ')})
            </summary>
            <div style="overflow-x:auto;margin-top:8px">
              <table class="data-table raw-table">
                <thead>
                  <tr>
                    <th>Date</th>
                    <th>Type</th>
                    {#each detail.tenors as t}<th>{t}</th>{/each}
                  </tr>
                </thead>
                <tbody>
                  {#each detail.rows as row}
                    <tr class:row-projected={row.period_type === 'projected'}>
                      <td>{row.date}</td>
                      <td>
                        <span class="period-badge period-badge--{row.period_type}">
                          {row.period_type}
                        </span>
                      </td>
                      {#each row.values as v}
                        <td style="text-align:right;font-family:monospace;font-size:12px">
                          {(v * 100).toFixed(4)}%
                        </td>
                      {/each}
                    </tr>
                  {/each}
                </tbody>
              </table>
            </div>
          </details>
        </div>
      {/if}
    </div>
  {/if}
</div>

<!-- ── Modale d'upload ──────────────────────────────────────────────────────── -->
{#if showUpload}
  <div class="overlay" onclick={() => (showUpload = false)}>
    <div class="modal" onclick={(e) => e.stopPropagation()}>
      <div class="modal-header">
        <h3>Importer une matrice de taux</h3>
        <button class="modal-close" onclick={() => (showUpload = false)}><X size={16}/></button>
      </div>

      {#if uploadError}
        <div class="alert-error">{uploadError}</div>
      {/if}

      <div class="form-grid">
        <label>
          Nom *
          <input type="text" bind:value={uName} placeholder="Ex. Base Rate EUR (ESTR)" />
        </label>
        <label>
          Description
          <input type="text" bind:value={uDesc} placeholder="Optionnel" />
        </label>
        <label>
          Devise
          <select bind:value={uCurrency}>
            <option>EUR</option>
            <option>USD</option>
            <option>GBP</option>
            <option>CHF</option>
            <option>JPY</option>
          </select>
        </label>
        <label>
          Interpolation
          <select bind:value={uInterp}>
            <option value="linear">linear — interpolation linéaire</option>
            <option value="cubic">cubic — spline cubique</option>
            <option value="flat_forward">flat_forward — taux forward constant</option>
          </select>
        </label>
        <label>
          Statut initial
          <select bind:value={uStatus}>
            <option value="draft">draft</option>
            <option value="active">active</option>
          </select>
        </label>
      </div>

      <!-- Sélection des risques -->
      <div class="risks-section">
        <div class="risks-label">Types de risque associés</div>
        <div class="risks-grid">
          {#each allRisks as r}
            <label class="risk-checkbox">
              <input
                type="checkbox"
                checked={uRisks.has(r.key)}
                onchange={() => toggleUploadRisk(r.key)}
              />
              <span class="risk-badge" style="background:{riskColor(r.key)}22;color:{riskColor(r.key)};border:1px solid {riskColor(r.key)}44">
                {r.label}
              </span>
            </label>
          {/each}
        </div>
        {#if uRisks.size > 1}
          <p class="warn-text">⚠ Plusieurs risques sélectionnés → ils seront indissociables dans les analyses</p>
        {/if}
      </div>

      <!-- Sélection du fichier -->
      <div class="file-zone" onclick={() => uFileInput.click()}>
        {#if uFile}
          <div class="file-selected">
            <span>📄 {uFile.name}</span>
            <span style="color:#6b7280;font-size:12px">({(uFile.size / 1024).toFixed(1)} Ko)</span>
          </div>
        {:else}
          <div>
            <Upload size={24} style="margin:0 auto 8px;display:block;color:#9ca3af" />
            <p style="font-weight:600;color:#374151">Cliquer pour sélectionner un fichier</p>
            <p style="color:#9ca3af;font-size:12px">Formats acceptés : .xlsx, .xlsm, .ods</p>
          </div>
        {/if}
        <input
          bind:this={uFileInput}
          type="file"
          accept=".xlsx,.xlsm,.ods"
          style="display:none"
          onchange={(e) => { uFile = (e.target as HTMLInputElement).files?.[0] ?? null; }}
        />
      </div>

      <div class="modal-footer">
        <button class="btn-primary" onclick={submitUpload} disabled={uploading || !uFile || !uName.trim()}>
          {uploading ? 'Import en cours…' : 'Importer'}
        </button>
        <button class="btn-sm" onclick={() => (showUpload = false)}>Annuler</button>
      </div>
    </div>
  </div>
{/if}

<style>
  /* ── Tableau ── */
  .data-table {
    width: 100%;
    border-collapse: collapse;
    font-size: 13px;
  }
  .data-table th {
    background: #f8f8fc;
    color: #6b7280;
    font-size: 11.5px;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: .04em;
    padding: 10px 14px;
    text-align: left;
    border-bottom: 1px solid #f0f0f5;
    white-space: nowrap;
  }
  .data-table td {
    padding: 10px 14px;
    border-bottom: 1px solid #f7f7fb;
    vertical-align: middle;
  }
  .data-table tbody tr:hover { background: #fafafe; }
  .row-selected { background: #eef2ff !important; }
  .row-projected { background: #fffbeb; }

  .name-cell { display: flex; flex-direction: column; gap: 2px; }
  .matrix-name { font-weight: 600; color: #1a1a2e; }
  .matrix-desc { font-size: 11.5px; color: #9ca3af; }
  .currency { font-weight: 600; font-size: 12px; }
  .dates-cell { font-size: 12px; color: #374151; white-space: nowrap; }
  .date-arrow { color: #9ca3af; margin: 0 4px; }
  .tenors-cell { font-size: 11.5px; color: #6b7280; max-width: 180px; }
  .interp-tag { background: #f1f0fe; color: #5b21b6; padding: 2px 7px; border-radius: 5px; font-size: 11.5px; font-family: 'JetBrains Mono', monospace; }

  .risk-badges { display: flex; flex-wrap: wrap; gap: 3px; }
  .risk-badge  { padding: 2px 7px; border-radius: 20px; font-size: 11px; font-weight: 600; white-space: nowrap; }

  .row-actions { display: flex; gap: 4px; }

  /* ── Panneau détail ── */
  .detail-panel { padding: 0; overflow: hidden; }

  .detail-header {
    display: flex;
    justify-content: space-between;
    align-items: flex-start;
    padding: 20px 24px;
    border-bottom: 1px solid #f0f0f5;
  }
  .detail-header h3 { font-size: 16px; font-weight: 700; margin-bottom: 4px; }
  .detail-desc  { font-size: 13px; color: #6b7280; margin-bottom: 8px; }
  .detail-meta  { display: flex; align-items: center; flex-wrap: wrap; gap: 6px; }
  .meta-pill    { background: #f3f4f6; color: #374151; padding: 2px 8px; border-radius: 20px; font-size: 11.5px; font-weight: 600; }
  .warn-multi-risk { color: #b45309; background: #fef3c7; padding: 2px 8px; border-radius: 20px; font-size: 11.5px; font-weight: 600; }

  .chart-wrapper { padding: 12px 24px 20px; }
  .chart-canvas  { width: 100%; height: 360px; }
  .chart-toolbar {
    display: flex;
    align-items: center;
    justify-content: space-between;
    margin-bottom: 8px;
    gap: 12px;
  }
  .chart-legend-hint {
    font-size: 11px;
    color: #6b7280;
    display: flex;
    align-items: center;
    gap: 4px;
  }
  .chart-view-toggle {
    display: flex;
    background: #f1f1f5;
    border-radius: 7px;
    padding: 3px;
    gap: 2px;
    flex-shrink: 0;
  }
  .toggle-btn {
    border: none;
    background: transparent;
    border-radius: 5px;
    padding: 4px 12px;
    font-size: 12px;
    font-weight: 500;
    color: #6b7280;
    cursor: pointer;
    transition: background 120ms, color 120ms;
  }
  .toggle-btn:hover { color: #374151; }
  .toggle-btn--active { background: #fff; color: #4f46e5; box-shadow: 0 1px 3px rgba(0,0,0,.08); font-weight: 600; }
  .hint-solid { color: #3b82f6; font-weight: 700; letter-spacing: -1px; }
  .hint-dash  { color: #f97316; font-weight: 700; letter-spacing: 1px; }

  /* ── Section édition ── */
  .edit-section {
    padding: 16px 24px;
    background: #f8f8fc;
    border-top: 1px solid #ededf5;
    border-bottom: 1px solid #ededf5;
  }
  .edit-section h4 { font-size: 13.5px; font-weight: 700; margin-bottom: 12px; }
  .edit-grid { display: grid; grid-template-columns: 1fr 1fr; gap: 12px; margin-bottom: 12px; }

  /* ── Section risques ── */
  .risks-section { margin-top: 12px; }
  .risks-label { font-size: 12.5px; font-weight: 500; color: #374151; margin-bottom: 8px; }
  .risks-grid   { display: flex; flex-wrap: wrap; gap: 6px; }
  .risk-checkbox { display: flex; align-items: center; gap: 5px; cursor: pointer; flex-direction: row; }
  .risk-checkbox input[type=checkbox] { width: auto; margin: 0; }
  .warn-text { font-size: 12px; color: #b45309; margin-top: 6px; }

  /* ── Données brutes ── */
  .raw-data-section { padding: 12px 24px 20px; }
  .raw-data-summary { font-size: 12.5px; color: #6b7280; cursor: pointer; list-style: none; font-weight: 500; }
  .raw-data-summary::-webkit-details-marker { display: none; }
  .raw-data-summary::before { content: '▶ '; font-size: 10px; }
  details[open] .raw-data-summary::before { content: '▼ '; }
  .raw-table { min-width: 600px; }
  .raw-table td, .raw-table th { padding: 6px 10px; }

  .period-badge { padding: 1px 6px; border-radius: 4px; font-size: 10.5px; font-weight: 600; }
  .period-badge--observed     { background: #dbeafe; color: #1e40af; }
  .period-badge--contrafactual { background: #ede9fe; color: #5b21b6; }
  .period-badge--projected     { background: #fef3c7; color: #92400e; }

  /* ── Modale ── */
  .overlay {
    position: fixed; inset: 0;
    background: rgba(0,0,0,.35);
    display: flex; align-items: center; justify-content: center;
    z-index: 100;
  }
  .modal {
    background: #fff;
    border-radius: 14px;
    padding: 24px;
    width: 680px;
    max-width: 95vw;
    max-height: 90vh;
    overflow-y: auto;
    box-shadow: 0 20px 60px rgba(0,0,0,.2);
  }
  .modal-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 20px;
  }
  .modal-header h3 { font-size: 16px; font-weight: 700; }
  .modal-close { background: none; border: none; cursor: pointer; color: #9ca3af; padding: 4px; border-radius: 6px; display:flex; }
  .modal-close:hover { background: #f3f4f6; color: #374151; }
  .modal-footer { display: flex; gap: 8px; margin-top: 20px; padding-top: 16px; border-top: 1px solid #f0f0f5; }

  .form-grid { display: grid; grid-template-columns: 1fr 1fr; gap: 12px; margin-bottom: 16px; }
  .form-grid label:first-child { grid-column: 1 / -1; }

  .file-zone {
    border: 2px dashed #e5e7eb;
    border-radius: 10px;
    padding: 24px;
    text-align: center;
    cursor: pointer;
    margin: 16px 0;
    transition: border-color 150ms, background 150ms;
  }
  .file-zone:hover { border-color: #6366f1; background: #f5f3ff; }
  .file-selected { display: flex; align-items: center; justify-content: center; gap: 8px; font-size: 13px; }
</style>
