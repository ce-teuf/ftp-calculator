<script lang="ts">
  const exportUrl = '/api/export';
</script>

<div class="datasets-layout">
  <div class="card">
    <h2>Datasets & Export</h2>
    <p class="subtitle">Import reference data or export a complete backup.</p>

    <div class="actions-grid">
      <div class="action-item">
        <div class="action-icon">⬇</div>
        <div class="action-content">
          <h3>Full JSON Export</h3>
          <p>Downloads a backup of all tables (curves, portfolios, positions, runoff, executions).</p>
          <a class="btn-primary" href={exportUrl} download>Download backup</a>
        </div>
      </div>

      <div class="action-item">
        <div class="action-icon">📈</div>
        <div class="action-content">
          <h3>Rate curves</h3>
          <p>Manage the curve library and build multi-component CoF curves.</p>
          <p class="hint">→ <strong>Curves</strong> tab</p>
        </div>
      </div>

      <div class="action-item">
        <div class="action-icon">🗂</div>
        <div class="action-content">
          <h3>Portfolios</h3>
          <p>Import positions via CSV or create portfolios manually.</p>
          <p class="hint">→ <strong>Portfolios</strong> tab</p>
        </div>
      </div>

      <div class="action-item">
        <div class="action-icon">🏦</div>
        <div class="action-content">
          <h3>NMD models</h3>
          <p>Calibrate and manage behavioral models for demand deposits.</p>
          <p class="hint">→ <strong>NMD</strong> tab</p>
        </div>
      </div>
    </div>
  </div>
</div>

<style>
.datasets-layout { display: flex; flex-direction: column; gap: 1.25rem; }
.card { background: #fff; border-radius: 12px; padding: 1.5rem; box-shadow: 0 1px 4px rgba(0,0,0,.08); }
.card h2 { margin: 0 0 .25rem; font-size: 1.1rem; font-weight: 700; }
.subtitle { margin: 0 0 1.5rem; font-size: .83rem; color: #666; }
.actions-grid { display: grid; grid-template-columns: 1fr 1fr; gap: 1rem; }
.action-item { display: flex; gap: 1rem; background: #f8f9fa; border-radius: 10px; padding: 1rem; }
.action-icon { font-size: 1.8rem; flex-shrink: 0; }
.action-content h3 { margin: 0 0 .4rem; font-size: .9rem; font-weight: 600; }
.action-content p { margin: 0 0 .6rem; font-size: .82rem; color: #555; }
.hint { color: #888 !important; font-size: .8rem !important; }
.btn-primary { display: inline-block; background: #1a73e8; color: #fff; border-radius: 7px;
  padding: .4rem .9rem; font-size: .83rem; text-decoration: none; }
.btn-primary:hover { background: #1557b0; }
</style>
