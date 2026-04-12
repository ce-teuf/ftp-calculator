<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { EditorState } from '@codemirror/state';
  import { EditorView, keymap, lineNumbers, highlightActiveLine } from '@codemirror/view';
  import { defaultKeymap, history, historyKeymap } from '@codemirror/commands';
  import { python } from '@codemirror/lang-python';
  import { oneDark } from '@codemirror/theme-one-dark';
  import { syntaxHighlighting, defaultHighlightStyle } from '@codemirror/language';

  import { rateSeries, curves, type SeriesInfo, type RateSeriesPoint } from '../api/client';
  import { usePyodide, type PyodideStatus, type PyodideResult } from '../usePyodide';

  // ── Constants ──────────────────────────────────────────────────────────────
  const FTP_TENORS = ['1M','3M','6M','1Y','2Y','3Y','5Y','7Y','10Y','15Y','20Y','30Y'];

  const STARTER_TEMPLATE = `import numpy as np
import pandas as pd
from scipy.interpolate import interp1d

# "data" est injecté automatiquement depuis la base de données
# data["ESTR"] → liste de dicts {date, tenor, rate}

df = pd.DataFrame(data["ESTR"])
df["rate"] = df["rate"].astype(float)

# Dernière date disponible
last = df["date"].max()
spot = df[df["date"] == last].copy()

# Mapper tenor → mois
tenor_months = {
    "1D": 0.033, "1W": 0.25, "1M": 1, "2M": 2, "3M": 3, "6M": 6,
    "9M": 9, "1Y": 12, "2Y": 24, "3Y": 36, "5Y": 60, "7Y": 84,
    "10Y": 120, "15Y": 180, "20Y": 240, "30Y": 360,
}
spot["months"] = spot["tenor"].map(tenor_months)
spot = spot.dropna(subset=["months"]).sort_values("months")

print(f"Dernière date: {last}")
print(f"Points disponibles: {len(spot)}")
print(spot[["tenor","months","rate"]].to_string(index=False))

# Interpolation sur les 12 tenors FTP standard
ftp_months = [1, 3, 6, 12, 24, 36, 60, 84, 120, 180, 240, 360]
f = interp1d(spot["months"], spot["rate"], kind="linear", fill_value="extrapolate")

result = [round(float(f(m)), 6) for m in ftp_months]
print(f"\\nCourbe interpolée: {[f'{r*100:.3f}%' for r in result]}")
`;

  // ── Pyodide ────────────────────────────────────────────────────────────────
  const py = usePyodide();
  let pyStatus = $state<PyodideStatus>('loading');
  let pyStatusText = $state('');
  let unsubStatus: (() => void) | null = null;

  // ── Series selection ───────────────────────────────────────────────────────
  let availableSeries = $state<SeriesInfo[]>([]);
  let selectedSeries  = $state<string[]>(['ESTR']);
  let dateFrom        = $state('2024-01-01');
  let dateTo          = $state(new Date().toISOString().slice(0, 10));
  let tenorFilter     = $state('');

  // ── Data state ─────────────────────────────────────────────────────────────
  let loadedData   = $state<Record<string, RateSeriesPoint[]>>({});
  let totalRows    = $state(0);
  let dataLoading  = $state(false);
  let dataError    = $state('');

  // ── Run state ──────────────────────────────────────────────────────────────
  let running    = $state(false);
  let runResult  = $state<PyodideResult | null>(null);
  let stdout     = $state('');

  // ── Save state ─────────────────────────────────────────────────────────────
  let saveName      = $state('Interpolated ESTR curve');
  let saveComponent = $state('base_rate');
  let saveCurrency  = $state('EUR');
  let saveDate      = $state(new Date().toISOString().slice(0, 10));
  let saving        = $state(false);
  let saveMsg       = $state('');

  // ── CodeMirror ─────────────────────────────────────────────────────────────
  let editorEl = $state<HTMLDivElement | null>(null);
  let editorView: EditorView | null = null;

  onMount(async () => {
    // Subscribe to Pyodide status
    unsubStatus = py.onStatus((s, text) => {
      pyStatus = s;
      if (text) pyStatusText = text;
    });

    // Load available series
    try {
      const r = await rateSeries.names();
      availableSeries = r.series;
    } catch {}

    // Init CodeMirror
    if (editorEl) {
      const state = EditorState.create({
        doc: STARTER_TEMPLATE,
        extensions: [
          history(),
          lineNumbers(),
          highlightActiveLine(),
          keymap.of([...defaultKeymap, ...historyKeymap]),
          python(),
          oneDark,
          EditorView.theme({
            '&': { height: '100%', fontSize: '13px' },
            '.cm-scroller': { overflow: 'auto', fontFamily: "'JetBrains Mono', 'Fira Code', monospace" },
          }),
          EditorView.lineWrapping,
        ],
      });
      editorView = new EditorView({ state, parent: editorEl });
    }
  });

  onDestroy(() => {
    unsubStatus?.();
    editorView?.destroy();
  });

  // ── Actions ────────────────────────────────────────────────────────────────

  async function loadData() {
    if (selectedSeries.length === 0) { dataError = 'Select at least one series.'; return; }
    dataLoading = true; dataError = '';
    try {
      const r = await rateSeries.query({
        series: selectedSeries,
        from: dateFrom || undefined,
        to: dateTo || undefined,
        tenor: tenorFilter || undefined,
      });
      loadedData = r.data;
      totalRows = r.total_rows;
    } catch (e: any) {
      dataError = e.message;
    } finally { dataLoading = false; }
  }

  async function runScript() {
    if (!editorView) return;
    const code = editorView.state.doc.toString();
    if (Object.keys(loadedData).length === 0) {
      stdout = '⚠ Load data before executing the script.'; return;
    }
    running = true; stdout = ''; runResult = null;
    try {
      const r = await py.run(code, { data: loadedData });
      runResult = r;
      stdout = r.stdout;
    } catch (e: any) {
      stdout = 'Erreur worker: ' + e.message;
    } finally { running = false; }
  }

  async function saveCurve() {
    if (!runResult?.result) return;
    saving = true; saveMsg = '';
    const tenors = runResult.resultTenors ?? FTP_TENORS;
    try {
      await curves.create({
        name: saveName,
        component: saveComponent,
        currency: saveCurrency,
        valid_from: saveDate || undefined,
        tenors_json: JSON.stringify(tenors),
        values_json: JSON.stringify(runResult.result),
        notes: 'Générée via Python Lab',
      });
      saveMsg = '✓ Curve saved in the library.';
    } catch (e: any) {
      saveMsg = '✗ ' + e.message;
    } finally { saving = false; }
  }

  function toggleSeries(name: string) {
    if (selectedSeries.includes(name)) {
      selectedSeries = selectedSeries.filter(s => s !== name);
    } else {
      selectedSeries = [...selectedSeries, name];
    }
  }

  function pct(v: number) {
    return (v * 100).toFixed(3) + '%';
  }

  // Derived: unique tenors available in loaded data
  let distinctTenors = $derived(
    [...new Set(
      Object.values(loadedData).flatMap(pts => pts.map(p => p.tenor ?? ''))
    )].filter(Boolean).sort()
  );

  // Derived: compute date range for preview chart (last 90 days, one tenor)
  let previewTenor = $derived(distinctTenors[0] ?? '');
</script>

<div class="lab">
  <!-- ── Top bar ─────────────────────────────────────────────────────────── -->
  <div class="topbar">
    <div class="topbar-left">
      <span class="title">Python Lab</span>
      <span class="py-badge" class:ready={pyStatus === 'ready'} class:running={pyStatus === 'running'}>
        {#if pyStatus === 'ready'}
          Python ready
        {:else if pyStatus === 'running'}
          Running…
        {:else if pyStatus === 'loading'}
          {pyStatusText || 'Loading Pyodide…'}
        {:else}
          Error
        {/if}
      </span>
    </div>
    <div class="topbar-right">
      <button class="btn-run" onclick={runScript}
        disabled={running || pyStatus === 'loading' || pyStatus === 'error'}>
        {running ? '▶ Running…' : '▶ Execute'}
      </button>
    </div>
  </div>

  <!-- ── Three-panel layout ─────────────────────────────────────────────── -->
  <div class="panels">

    <!-- ── Panel 1: Data selector ──────────────────────────────────────── -->
    <aside class="panel panel-data">
      <div class="panel-hdr">Data</div>

      <div class="section-label">Series</div>
      <div class="series-chips">
        {#each availableSeries as s}
          <button
            class="chip"
            class:selected={selectedSeries.includes(s.name)}
            onclick={() => toggleSeries(s.name)}
          >{s.name}</button>
        {/each}
        {#if availableSeries.length === 0}
          <span class="muted">No series in database</span>
        {/if}
      </div>

      <div class="section-label">Period</div>
      <label class="field-sm">
        <span>From</span>
        <input type="date" bind:value={dateFrom} />
      </label>
      <label class="field-sm">
        <span>To</span>
        <input type="date" bind:value={dateTo} />
      </label>

      <div class="section-label">Tenor (optional)</div>
      <select class="sel-sm" bind:value={tenorFilter}>
        <option value="">All</option>
        {#each ['1D','1W','1M','2M','3M','6M','9M','1Y','2Y','3Y','5Y','7Y','10Y','15Y','20Y','30Y'] as t}
          <option value={t}>{t}</option>
        {/each}
      </select>

      {#if dataError}
        <div class="alert-err">{dataError}</div>
      {/if}

      <button class="btn-load" onclick={loadData} disabled={dataLoading}>
        {dataLoading ? 'Loading…' : 'Load data'}
      </button>

      {#if totalRows > 0}
        <div class="data-summary">
          <span class="data-ok">✓ {totalRows.toLocaleString()} observations loaded</span>
          {#each Object.entries(loadedData) as [name, pts]}
            <div class="data-series-row">
              <span class="series-name">{name}</span>
              <span class="series-count">{pts.length.toLocaleString('fr-FR')} pts</span>
            </div>
          {/each}
        </div>
      {/if}
    </aside>

    <!-- ── Panel 2: Code editor ─────────────────────────────────────────── -->
    <div class="panel panel-editor">
      <div class="panel-hdr">
        Script Python
        <span class="env-hint">vars: <code>data</code>, out: <code>result</code> (list[float])</span>
      </div>
      <div class="editor-wrap" bind:this={editorEl}></div>

      {#if stdout}
        <div class="stdout-box">
          <div class="stdout-hdr">stdout / stderr</div>
          <pre class="stdout">{stdout}</pre>
        </div>
      {/if}
    </div>

    <!-- ── Panel 3: Output ──────────────────────────────────────────────── -->
    <aside class="panel panel-output">
      <div class="panel-hdr">Result</div>

      {#if !runResult}
        <div class="empty-output">
          <p>Execute the script to see the result.</p>
          <p class="hint">The script must define <code>result</code> — a list of decimal rates at standard FTP tenors.</p>
        </div>
      {:else if !runResult.ok}
        <div class="run-error">
          <div class="err-title">Python error</div>
          <pre class="err-trace">{runResult.error}</pre>
        </div>
      {:else if !runResult.result}
        <div class="run-warn">⚠ The script didn't define <code>result</code>.</div>
      {:else}
        {@const tenors = runResult.resultTenors ?? FTP_TENORS}
        {@const values = runResult.result ?? []}
        {@const maxV = Math.max(...values, 0.001)}
        <!-- Result table -->
        <div class="result-table-wrap">
          <table class="result-table">
            <thead>
              <tr><th>Tenor</th><th>Rate</th><th>%</th></tr>
            </thead>
            <tbody>
              {#each tenors as t, i}
                {@const v = values[i] ?? null}
                <tr class:missing={v == null}>
                  <td class="mono">{t}</td>
                  <td class="num">{v != null ? v.toFixed(6) : '—'}</td>
                  <td class="num accent">{v != null ? pct(v) : '—'}</td>
                </tr>
              {/each}
            </tbody>
          </table>
        </div>

        <!-- Mini bar chart -->
        <div class="minichartlabel">Interpolated curve</div>
        <div class="minichart">
          {#each values as v, i}
            <div class="bar-col">
              <div class="bar" style="height:{Math.max(2, (v/maxV)*100)}%;background:#6366f1"></div>
              <div class="bar-lbl">{tenors[i] ?? ''}</div>
            </div>
          {/each}
        </div>

        <!-- Save to library -->
        <div class="save-section">
          <div class="section-label">Save to library</div>
          <input class="input-sm" type="text" bind:value={saveName} placeholder="Curve name" />
          <div class="save-row">
            <select class="sel-sm" bind:value={saveComponent}>
              <option value="base_rate">Base Rate</option>
              <option value="credit_spread">Credit Spread</option>
              <option value="tlp">TLP</option>
              <option value="clp">CLP</option>
              <option value="all_in_cof">All-In CoF</option>
            </select>
            <select class="sel-sm" bind:value={saveCurrency}>
              <option>EUR</option><option>USD</option><option>GBP</option>
            </select>
          </div>
          <input class="input-sm" type="date" bind:value={saveDate} />
          <button class="btn-save" onclick={saveCurve} disabled={saving}>
            {saving ? 'Saving…' : 'Save to library'}
          </button>
          {#if saveMsg}
            <div class="save-msg" class:ok={saveMsg.startsWith('✓')}>{saveMsg}</div>
          {/if}
        </div>
      {/if}
    </aside>
  </div>
</div>

<style>
.lab {
  display: flex; flex-direction: column; height: 100%; gap: 0;
  background: #0f0f1a; color: #e2e8f0; border-radius: 10px; overflow: hidden;
}

/* ── Top bar ── */
.topbar {
  display: flex; justify-content: space-between; align-items: center;
  padding: .6rem 1rem; background: #16162a; border-bottom: 1px solid #2a2a45;
  flex-shrink: 0;
}
.topbar-left { display: flex; align-items: center; gap: .75rem; }
.title { font-weight: 700; font-size: .95rem; color: #a5b4fc; }
.py-badge {
  display: inline-block; padding: 2px 10px; border-radius: 99px; font-size: .73rem;
  background: #2a2a45; color: #94a3b8; border: 1px solid #3a3a55;
}
.py-badge.ready { background: #14532d40; color: #4ade80; border-color: #16a34a60; }
.py-badge.running { background: #312e8140; color: #a5b4fc; border-color: #6366f160;
  animation: pulse 1.2s ease-in-out infinite; }
@keyframes pulse { 0%,100% { opacity: 1; } 50% { opacity: .6; } }

/* ── Panel layout ── */
.panels {
  display: flex; flex: 1; overflow: hidden; min-height: 0;
}

.panel {
  display: flex; flex-direction: column; overflow: hidden;
  border-right: 1px solid #2a2a45;
}
.panel:last-child { border-right: none; }

.panel-data   { width: 220px; flex-shrink: 0; background: #131325; overflow-y: auto; }
.panel-editor { flex: 1; background: #1a1a2e; }
.panel-output { width: 260px; flex-shrink: 0; background: #131325; overflow-y: auto; }

.panel-hdr {
  padding: .5rem .75rem; background: #1e1e38; border-bottom: 1px solid #2a2a45;
  font-size: .78rem; font-weight: 600; color: #a5b4fc; flex-shrink: 0;
  display: flex; align-items: center; gap: .5rem;
}
.env-hint { font-size: .7rem; color: #64748b; font-weight: 400; margin-left: auto; }

/* ── Data panel ── */
.section-label {
  padding: .5rem .75rem .2rem; font-size: .7rem; text-transform: uppercase;
  letter-spacing: .06em; color: #64748b; font-weight: 600;
}
.series-chips { display: flex; flex-wrap: wrap; gap: .3rem; padding: .35rem .6rem .6rem; }
.chip {
  padding: 2px 9px; border-radius: 99px; font-size: .74rem; font-weight: 600;
  border: 1px solid #3a3a55; background: #1e1e38; color: #94a3b8; cursor: pointer;
  transition: all .12s;
}
.chip.selected { background: #4f46e5; color: white; border-color: #6366f1; }
.chip:hover:not(.selected) { border-color: #6366f1; color: #a5b4fc; }

.field-sm {
  display: flex; flex-direction: column; gap: .2rem;
  padding: .2rem .6rem .45rem; font-size: .74rem; color: #94a3b8;
}
.field-sm input, .sel-sm {
  background: #0f0f1a; border: 1px solid #3a3a55; border-radius: 5px;
  color: #e2e8f0; padding: .28rem .5rem; font-size: .78rem; outline: none;
}
.field-sm input:focus, .sel-sm:focus { border-color: #6366f1; }
.sel-sm { margin: 0 .6rem .5rem; width: calc(100% - 1.2rem); }

.alert-err {
  margin: .25rem .6rem; background: #fee2e260; color: #fca5a5; border-radius: 5px;
  padding: .35rem .5rem; font-size: .74rem;
}
.muted { font-size: .74rem; color: #475569; padding: .2rem 0; }

.btn-load {
  margin: .4rem .6rem .5rem; padding: .38rem .7rem; border-radius: 6px;
  border: none; background: #4f46e5; color: white; font-size: .8rem;
  font-weight: 600; cursor: pointer; transition: background .12s; width: calc(100% - 1.2rem);
}
.btn-load:hover:not(:disabled) { background: #4338ca; }
.btn-load:disabled { opacity: .5; cursor: not-allowed; }

.data-summary { padding: .4rem .75rem .75rem; display: flex; flex-direction: column; gap: .2rem; }
.data-ok { font-size: .74rem; color: #4ade80; font-weight: 600; }
.data-series-row { display: flex; justify-content: space-between; font-size: .73rem; }
.series-name { color: #a5b4fc; }
.series-count { color: #64748b; }

/* ── Editor ── */
.editor-wrap {
  flex: 1; overflow: hidden; min-height: 0;
  /* CodeMirror fills this */
}
.editor-wrap :global(.cm-editor) { height: 100%; }

.stdout-box {
  border-top: 1px solid #2a2a45; background: #0a0a14; flex-shrink: 0;
  max-height: 180px; overflow: hidden; display: flex; flex-direction: column;
}
.stdout-hdr {
  padding: .3rem .75rem; font-size: .7rem; color: #475569; background: #0f0f1a;
  border-bottom: 1px solid #1a1a2e; flex-shrink: 0;
}
.stdout { margin: 0; padding: .5rem .75rem; font-size: .75rem; font-family: monospace;
  color: #94a3b8; overflow-y: auto; line-height: 1.5; white-space: pre-wrap; }

/* ── Run button ── */
.btn-run {
  padding: .38rem 1.1rem; border-radius: 6px; border: none;
  background: #059669; color: white; font-size: .85rem; font-weight: 700;
  cursor: pointer; transition: background .12s;
}
.btn-run:hover:not(:disabled) { background: #047857; }
.btn-run:disabled { opacity: .5; cursor: not-allowed; }

/* ── Output panel ── */
.empty-output { padding: 1.25rem .75rem; color: #475569; font-size: .82rem; line-height: 1.6; }
.empty-output code { color: #a5b4fc; }
.hint { color: #374151; font-size: .76rem; }

.run-error { padding: .75rem; }
.err-title { color: #f87171; font-size: .82rem; font-weight: 600; margin-bottom: .4rem; }
.err-trace { font-size: .73rem; color: #fca5a5; white-space: pre-wrap; word-break: break-all;
  background: #1a0a0a; border-radius: 5px; padding: .5rem; margin: 0; max-height: 200px; overflow-y: auto; }
.run-warn { padding: .75rem; color: #fbbf24; font-size: .82rem; }
.run-warn code { color: #a5b4fc; }

/* ── Result table ── */
.result-table-wrap { overflow-x: auto; }
.result-table { width: 100%; border-collapse: collapse; font-size: .78rem; }
.result-table th {
  padding: .3rem .5rem; text-align: left; color: #475569;
  font-size: .7rem; text-transform: uppercase; letter-spacing: .04em;
  border-bottom: 1px solid #2a2a45;
}
.result-table td { padding: .28rem .5rem; border-bottom: 1px solid #1e1e38; }
.result-table tr.missing td { opacity: .4; }
.mono { font-family: monospace; font-size: .75rem; }
.num { text-align: right; font-variant-numeric: tabular-nums; color: #94a3b8; }
.accent { color: #a5b4fc !important; font-weight: 600; }

/* ── Mini chart ── */
.minichartlabel { padding: .5rem .75rem .25rem; font-size: .7rem; color: #475569;
  text-transform: uppercase; letter-spacing: .05em; }
.minichart {
  display: flex; align-items: flex-end; gap: 2px; height: 60px;
  padding: 0 .75rem .5rem; border-bottom: 1px solid #2a2a45;
}
.bar-col { display: flex; flex-direction: column; align-items: center; flex: 1; height: 100%; }
.bar { width: 100%; border-radius: 2px 2px 0 0; min-height: 2px; transition: height .3s ease; }
.bar-lbl { font-size: .56rem; color: #475569; margin-top: 2px; white-space: nowrap;
  transform: rotate(-45deg); transform-origin: center; margin-top: 4px; }

/* ── Save section ── */
.save-section {
  padding: .75rem; border-top: 1px solid #2a2a45;
  display: flex; flex-direction: column; gap: .4rem;
}
.input-sm {
  background: #0f0f1a; border: 1px solid #3a3a55; border-radius: 5px;
  color: #e2e8f0; padding: .3rem .5rem; font-size: .78rem; outline: none; width: 100%;
  box-sizing: border-box;
}
.input-sm:focus { border-color: #6366f1; }
.save-row { display: flex; gap: .4rem; }
.save-row .sel-sm { margin: 0; width: auto; flex: 1; }
.btn-save {
  padding: .4rem .7rem; border-radius: 6px; border: none;
  background: #6366f1; color: white; font-size: .8rem; font-weight: 600;
  cursor: pointer; transition: background .12s; width: 100%;
}
.btn-save:hover:not(:disabled) { background: #4f52d9; }
.btn-save:disabled { opacity: .5; cursor: not-allowed; }
.save-msg { font-size: .78rem; padding: .3rem .4rem; border-radius: 4px; }
.save-msg.ok { color: #4ade80; }
.save-msg:not(.ok) { color: #f87171; }
</style>
