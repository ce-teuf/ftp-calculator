<script lang="ts">
  import { portfolios, executions, type Portfolio, type PortfolioPosition, type Execution } from '../api/client.ts';
  import { downloadTemplate, exportPortfolioExcel } from '../export/excel.ts';

  let list = $state<Portfolio[]>([]);
  let selected = $state<Portfolio | null>(null);
  let positions = $state<PortfolioPosition[]>([]);
  let loading = $state(false);
  let error = $state('');
  let showNewPortfolio = $state(false);
  let csvText = $state('');
  let csvError = $state('');

  let newPortfolio = $state({ name: '', description: '', as_of_date: new Date().toISOString().slice(0,10) });

  async function loadList() {
    loading = true; error = '';
    try { list = await portfolios.list(); }
    catch(e: any) { error = e.message; }
    finally { loading = false; }
  }

  async function selectPortfolio(p: Portfolio) {
    selected = p;
    positions = await portfolios.positions(p.id);
    loadLastExec(p.id);
  }

  async function createPortfolio() {
    try {
      await portfolios.create(newPortfolio);
      showNewPortfolio = false;
      newPortfolio = { name:'', description:'', as_of_date: new Date().toISOString().slice(0,10) };
      await loadList();
    } catch(e: any) { error = e.message; }
  }

  async function deletePortfolio(id: string) {
    if (!confirm('Supprimer ce portefeuille et toutes ses positions ?')) return;
    await portfolios.delete(id);
    if (selected?.id === id) { selected = null; positions = []; }
    await loadList();
  }

  async function deletePosition(posId: string) {
    await portfolios.deletePosition(posId);
    if (selected) positions = await portfolios.positions(selected.id);
  }

  async function importCsv() {
    csvError = '';
    if (!selected) return;
    const lines = csvText.trim().split('\n');
    const header = lines[0].split(',').map(h => h.trim().toLowerCase());
    const rows = lines.slice(1).map(line => {
      const vals = line.split(',');
      const obj: Record<string,string> = {};
      header.forEach((h,i) => { obj[h] = vals[i]?.trim() ?? ''; });
      return obj;
    });
    const parsed = rows.map(r => ({
      position_ref: r.position_ref || undefined,
      product_type: r.product_type || 'LOAN',
      branch: r.branch || undefined,
      seller: r.seller || undefined,
      currency: r.currency || 'EUR',
      outstanding: parseFloat(r.outstanding) || 0,
      origination_date: r.origination_date || undefined,
      maturity_date: r.maturity_date || undefined,
      client_rate: r.client_rate ? parseFloat(r.client_rate) : undefined,
      risk_weight: r.risk_weight ? parseFloat(r.risk_weight) : 1.0,
      profiles_json: r.profiles_json || undefined,
      rates_json: r.rates_json || undefined,
    }));
    try {
      const res = await portfolios.bulkImport(selected.id, parsed);
      csvText = '';
      positions = await portfolios.positions(selected.id);
      await loadList();
      alert(`${res.imported} positions importées.`);
    } catch(e: any) { csvError = e.message; }
  }

  let capitalChargePct = $state(8.0);
  let elPct = $state(0.5);
  let lastPortfolioExec = $state<Execution | null>(null);

  async function loadLastExec(portfolioId: string) {
    try {
      const all = await executions.list();
      lastPortfolioExec = all.find(e => e.portfolio_id === portfolioId && e.status === 'completed') ?? null;
    } catch { lastPortfolioExec = null; }
  }

  function getPositionFtpRate(posIdx: number): number | null {
    if (!lastPortfolioExec?.result_json) return null;
    try {
      const res = JSON.parse(lastPortfolioExec.result_json);
      return res.ftp_rate?.[posIdx]?.[0] ?? null;
    } catch { return null; }
  }

  function raroc(pos: PortfolioPosition, posIdx: number): string {
    const ftpRate = getPositionFtpRate(posIdx);
    if (ftpRate == null || pos.client_rate == null) return '—';
    const nim = pos.client_rate - ftpRate;
    const el  = elPct / 100;
    const cap = pos.outstanding * (pos.risk_weight ?? 1) * (capitalChargePct / 100);
    if (cap === 0) return '—';
    const rarocVal = (nim - el) * pos.outstanding / cap;
    return (rarocVal * 100).toFixed(1) + '%';
  }

  $effect(() => { loadList(); });
</script>

<div class="tab-layout">
  <!-- ── Liste des portefeuilles ── -->
  <aside class="sidebar">
    <div class="sidebar-header">
      <h3>Portefeuilles</h3>
      <button class="btn-icon" onclick={() => showNewPortfolio = !showNewPortfolio} title="Nouveau">+</button>
    </div>

    {#if showNewPortfolio}
      <div class="new-form">
        <input type="text" bind:value={newPortfolio.name} placeholder="Nom du portefeuille" />
        <input type="text" bind:value={newPortfolio.description} placeholder="Description (optionnel)" />
        <input type="date" bind:value={newPortfolio.as_of_date} />
        <div class="btn-row">
          <button class="btn-primary" onclick={createPortfolio}>Créer</button>
          <button class="btn-ghost" onclick={() => showNewPortfolio = false}>Annuler</button>
        </div>
      </div>
    {/if}

    {#if error}
      <div class="alert-error">{error}</div>
    {/if}

    <div class="portfolio-list">
      {#each list as p}
        <div
          class="portfolio-item"
          class:active={selected?.id === p.id}
          onclick={() => selectPortfolio(p)}
          role="button"
          tabindex="0"
          onkeydown={e => e.key === 'Enter' && selectPortfolio(p)}
        >
          <div class="pi-name">{p.name}</div>
          <div class="pi-meta">{p.as_of_date} · {p.status}</div>
          <button class="btn-del" onclick={e => { e.stopPropagation(); deletePortfolio(p.id); }} title="Supprimer">✕</button>
        </div>
      {/each}
      {#if !loading && list.length === 0}
        <div class="empty">Aucun portefeuille</div>
      {/if}
    </div>
  </aside>

  <!-- ── Panneau positions ── -->
  <main class="main-panel">
    {#if selected}
      <div class="panel-header">
        <div>
          <h2>{selected.name}</h2>
          <span class="meta">{positions.length} position(s) · {selected.as_of_date}</span>
        </div>
        <div class="panel-actions">
          <button class="btn-sm" onclick={downloadTemplate}>⬇ Modèle Excel</button>
          <button class="btn-sm" onclick={() => exportPortfolioExcel(positions, selected!.name)}>⬇ Export Excel</button>
        </div>
      </div>

      <!-- RAROC params -->
      {#if lastPortfolioExec}
        <div class="raroc-bar">
          <span>Paramètres RAROC :</span>
          <label>Capital <input type="number" bind:value={capitalChargePct} step="0.5" min="0" max="100" />%</label>
          <label>EL <input type="number" bind:value={elPct} step="0.1" min="0" max="10" />%</label>
          <span class="hint">Basé sur : {lastPortfolioExec.method.toUpperCase()} — {lastPortfolioExec.created_at?.slice(0,10)}</span>
        </div>
      {/if}

      <!-- Table positions -->
      <div class="table-wrap">
        <table>
          <thead>
            <tr>
              <th>Réf.</th>
              <th>Produit</th>
              <th>Branche</th>
              <th>Vendeur</th>
              <th>Encours</th>
              <th>Taux client</th>
              <th>Taux FTP</th>
              <th>NIM</th>
              <th>RAROC</th>
              <th></th>
            </tr>
          </thead>
          <tbody>
            {#each positions as pos, i}
              {@const ftpRate = getPositionFtpRate(i)}
              {@const nim = ftpRate != null && pos.client_rate != null ? pos.client_rate - ftpRate : null}
              <tr>
                <td>{pos.position_ref ?? pos.id.slice(0,8)}</td>
                <td>{pos.product_type}</td>
                <td>{pos.branch ?? '—'}</td>
                <td>{pos.seller ?? '—'}</td>
                <td class="num">{pos.outstanding.toLocaleString('fr-FR', {maximumFractionDigits:0})} €</td>
                <td class="num">{pos.client_rate != null ? (pos.client_rate*100).toFixed(2)+'%' : '—'}</td>
                <td class="num">{ftpRate != null ? (ftpRate*100).toFixed(3)+'%' : '—'}</td>
                <td class="num" class:nim-pos={nim != null && nim > 0} class:nim-neg={nim != null && nim < 0}>
                  {nim != null ? (nim*100).toFixed(3)+'%' : '—'}
                </td>
                <td class="num raroc-cell">{raroc(pos, i)}</td>
                <td><button class="btn-del" onclick={() => deletePosition(pos.id)}>✕</button></td>
              </tr>
            {/each}
          </tbody>
        </table>
      </div>

      <!-- Import CSV -->
      <div class="card">
        <h3>Import CSV</h3>
        <p class="hint">Colonnes : position_ref, product_type, branch, seller, currency, outstanding, origination_date, maturity_date, client_rate, risk_weight, profiles_json, rates_json</p>
        {#if csvError}
          <div class="alert-error">{csvError}</div>
        {/if}
        <textarea rows="5" bind:value={csvText} placeholder="Collez votre CSV ici…" spellcheck="false"></textarea>
        <button class="btn-primary" onclick={importCsv} disabled={!csvText.trim()}>Importer</button>
      </div>
    {:else}
      <div class="empty-state">
        <div class="empty-icon">🗂</div>
        <p>Sélectionnez un portefeuille</p>
      </div>
    {/if}
  </main>
</div>

<style>
.tab-layout { display: flex; gap: 1rem; height: 100%; }
.sidebar { width: 240px; flex-shrink: 0; display: flex; flex-direction: column; gap: .5rem; }
.sidebar-header { display: flex; align-items: center; justify-content: space-between; }
.sidebar-header h3 { margin: 0; font-size: .95rem; }
.btn-icon { width: 28px; height: 28px; border-radius: 50%; border: 1px solid #1a73e8;
  background: #fff; color: #1a73e8; font-size: 1.2rem; cursor: pointer; line-height: 1; }
.btn-icon:hover { background: #e8f0fe; }
.new-form { background: #fff; border: 1px solid #e0e0e0; border-radius: 8px; padding: .75rem;
  display: flex; flex-direction: column; gap: .5rem; }
.new-form input { border: 1px solid #ddd; border-radius: 6px; padding: .35rem .6rem; font-size: .85rem; }
.btn-row { display: flex; gap: .5rem; }
.alert-error { background: #fce8e6; color: #c5221f; padding: .5rem .8rem; border-radius: 6px; font-size: .82rem; }
.portfolio-list { flex: 1; overflow-y: auto; display: flex; flex-direction: column; gap: .4rem; }
.portfolio-item { background: #fff; border: 1px solid #e0e0e0; border-radius: 8px; padding: .6rem .8rem;
  cursor: pointer; position: relative; }
.portfolio-item:hover { border-color: #1a73e8; }
.portfolio-item.active { border-color: #1a73e8; background: #e8f0fe; }
.pi-name { font-weight: 600; font-size: .88rem; }
.pi-meta { font-size: .75rem; color: #777; }
.empty { color: #999; font-size: .85rem; text-align: center; margin-top: 1rem; }

.main-panel { flex: 1; overflow-y: auto; display: flex; flex-direction: column; gap: 1rem; }
.panel-header { display: flex; justify-content: space-between; align-items: flex-start; }
.panel-header h2 { margin: 0; font-size: 1.1rem; }
.meta { font-size: .8rem; color: #666; }
.panel-actions { display: flex; gap: .5rem; }
.btn-primary { background: #1a73e8; color: #fff; border: none; border-radius: 8px;
  padding: .4rem 1rem; font-size: .85rem; cursor: pointer; }
.btn-primary:hover { background: #1557b0; }
.btn-primary:disabled { background: #b0c4de; cursor: not-allowed; }
.btn-ghost { background: #fff; border: 1px solid #ddd; border-radius: 8px;
  padding: .4rem .9rem; font-size: .85rem; cursor: pointer; }
.btn-sm { background: #fff; border: 1px solid #1a73e8; color: #1a73e8;
  border-radius: 6px; padding: .3rem .8rem; font-size: .82rem; cursor: pointer; }
.btn-sm:hover { background: #e8f0fe; }
.btn-del { background: none; border: none; color: #ea4335; cursor: pointer; padding: .1rem .3rem;
  font-size: .9rem; }
.btn-del:hover { background: #fce8e6; border-radius: 4px; }
.raroc-bar { display: flex; align-items: center; gap: 1rem; background: #fff;
  border-radius: 8px; padding: .6rem 1rem; font-size: .85rem; flex-wrap: wrap; }
.raroc-bar label { display: flex; align-items: center; gap: .4rem; }
.raroc-bar input { width: 60px; border: 1px solid #ddd; border-radius: 4px; padding: .2rem .4rem;
  font-size: .85rem; text-align: right; }
.hint { font-size: .78rem; color: #888; }
.table-wrap { overflow-x: auto; background: #fff; border-radius: 12px;
  box-shadow: 0 1px 4px rgba(0,0,0,.08); }
table { width: 100%; border-collapse: collapse; font-size: .83rem; }
th, td { border-bottom: 1px solid #f0f0f0; padding: .45rem .7rem; }
th { background: #f8f9fa; font-weight: 600; color: #444; position: sticky; top: 0; }
.num { text-align: right; font-variant-numeric: tabular-nums; }
.nim-pos { color: #34a853; font-weight: 600; }
.nim-neg { color: #ea4335; font-weight: 600; }
.raroc-cell { font-weight: 700; }
.card { background: #fff; border-radius: 12px; padding: 1.25rem; box-shadow: 0 1px 4px rgba(0,0,0,.08); }
.card h3 { margin: 0 0 .5rem; font-size: .95rem; }
.card textarea { width: 100%; border: 1px solid #ddd; border-radius: 6px; padding: .5rem;
  font-size: .82rem; font-family: monospace; resize: vertical; margin: .5rem 0; box-sizing: border-box; }
.empty-state { display: flex; flex-direction: column; align-items: center; justify-content: center;
  flex: 1; color: #999; gap: .5rem; }
.empty-icon { font-size: 3rem; }
</style>
