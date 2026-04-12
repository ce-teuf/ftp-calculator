<script lang="ts">
  import { datasets as api, type FsDataset, type Contract } from '../api/client';

  let fsList    = $state<FsDataset[]>([]);
  let selected  = $state<FsDataset | null>(null);
  let contracts = $state<Contract[]>([]);
  let loading   = $state(false);
  let loadingC  = $state(false);
  let opMsg     = $state<{ ok: boolean; text: string } | null>(null);
  let acting    = $state(false);

  async function refresh() {
    loading = true; opMsg = null;
    try {
      const r = await api.available();
      fsList = r.datasets;
      // refresh selected too
      if (selected) {
        const updated = fsList.find(d => d.folder === selected!.folder);
        if (updated) {
          selected = updated;
          if (updated.loaded_in_db) await loadContracts(updated.meta.id);
          else contracts = [];
        }
      }
    } catch (e: any) {
      opMsg = { ok: false, text: e.message };
    } finally { loading = false; }
  }

  async function select(ds: FsDataset) {
    selected = ds;
    contracts = [];
    opMsg = null;
    if (ds.loaded_in_db) await loadContracts(ds.meta.id);
  }

  async function loadContracts(id: string) {
    loadingC = true;
    try { contracts = await api.contracts(id); }
    catch {}
    finally { loadingC = false; }
  }

  async function charger() {
    if (!selected || acting) return;
    acting = true; opMsg = null;
    try {
      const r = await api.loadFs(selected.folder);
      opMsg = { ok: true, text: `Chargé : ${r.loaded.contracts} contrats, ${r.loaded.curves} courbes, ${r.loaded.runoff_models} modèles NMD.` };
      await refresh();
    } catch (e: any) {
      opMsg = { ok: false, text: e.message };
    } finally { acting = false; }
  }

  async function retirer() {
    if (!selected || acting) return;
    if (!confirm(`Retirer "${selected.meta.name}" de la base ?`)) return;
    acting = true; opMsg = null;
    try {
      const res = await api.delete(selected.meta.id);
      if (!res.ok && res.status !== 404) throw new Error(`HTTP ${res.status}`);
      opMsg = { ok: true, text: 'Dataset retiré de la base.' };
      contracts = [];
      await refresh();
    } catch (e: any) {
      opMsg = { ok: false, text: e.message };
    } finally { acting = false; }
  }

  async function exportZip() {
    if (!selected || !selected.loaded_in_db || acting) return;
    acting = true; opMsg = null;
    try {
      const res = await api.exportZip(selected.meta.id);
      if (!res.ok) throw new Error(`HTTP ${res.status}`);
      const blob = await res.blob();
      const url = URL.createObjectURL(blob);
      const a = document.createElement('a');
      a.href = url;
      a.download = `dataset_${selected.meta.name.replace(/[^a-zA-Z0-9-]/g, '_')}.zip`;
      a.click();
      URL.revokeObjectURL(url);
      opMsg = { ok: true, text: 'Export ZIP téléchargé.' };
    } catch (e: any) {
      opMsg = { ok: false, text: e.message };
    } finally { acting = false; }
  }

  function fmtNum(n?: number) {
    return n != null ? n.toLocaleString('fr-FR') : '—';
  }

  $effect(() => { refresh(); });
</script>

<div class="tab-content">
  <div class="tab-header">
    <div>
      <h2>Datasets Explorer</h2>
      <p class="sub">Datasets disponibles sur le serveur. Chargez-les en base pour les utiliser dans les calculs.</p>
    </div>
    <button class="btn-ghost" onclick={refresh} disabled={loading}>
      {loading ? 'Actualisation…' : 'Actualiser'}
    </button>
  </div>

  <div class="layout">
    <!-- ── Sidebar ── -->
    <aside class="sidebar">
      <div class="sidebar-hdr">Disponibles ({fsList.length})</div>

      {#if loading && fsList.length === 0}
        <div class="empty-side">Chargement…</div>
      {:else if fsList.length === 0}
        <div class="empty-side">Aucun dataset trouvé.<br>Générez-en via le Creator.</div>
      {:else}
        {#each fsList as ds}
          {@const isSelected = selected?.folder === ds.folder}
          <div
            class="ds-item"
            class:active={isSelected}
            onclick={() => select(ds)}
            role="button"
            tabindex="0"
            onkeydown={(e) => e.key === 'Enter' && select(ds)}
          >
            <div class="ds-row-top">
              <span class="ds-name">{ds.meta.name}</span>
              {#if ds.loaded_in_db}
                <span class="badge badge-db">En DB</span>
              {:else}
                <span class="badge badge-fs">Disponible</span>
              {/if}
            </div>
            {#if ds.meta.description}
              <div class="ds-desc">{ds.meta.description}</div>
            {/if}
            <div class="ds-counts">
              {#if ds.meta.contract_count != null}
                <span>{ds.meta.contract_count} contrats</span>
              {/if}
              {#if ds.meta.as_of_date}
                <span>{ds.meta.as_of_date}</span>
              {/if}
            </div>
          </div>
        {/each}
      {/if}
    </aside>

    <!-- ── Detail panel ── -->
    <section class="detail">
      {#if !selected}
        <div class="empty-state"><p>Sélectionnez un dataset</p></div>
      {:else}
        <!-- Header card -->
        <div class="card detail-hdr">
          <div class="detail-info">
            <h3>{selected.meta.name}</h3>
            {#if selected.meta.description}<p class="desc">{selected.meta.description}</p>{/if}
            <div class="folder-path">📁 {selected.folder}</div>
          </div>
          <div class="action-btns">
            {#if selected.loaded_in_db}
              <button class="btn-outline" onclick={exportZip} disabled={acting}>Export ZIP</button>
              <button class="btn-danger" onclick={retirer} disabled={acting}>
                {acting ? '…' : 'Retirer de la DB'}
              </button>
            {:else}
              <button class="btn-primary" onclick={charger} disabled={acting}>
                {acting ? 'Chargement…' : 'Charger en DB'}
              </button>
            {/if}
          </div>
        </div>

        {#if opMsg}
          <div class="alert" class:alert-ok={opMsg.ok} class:alert-err={!opMsg.ok}>{opMsg.text}</div>
        {/if}

        <!-- KPI strip -->
        <div class="kpi-row">
          <div class="kpi">
            <div class="kv">{selected.meta.contract_count ?? '?'}</div>
            <div class="kl">Contrats</div>
          </div>
          <div class="kpi">
            <div class="kv">{(selected.meta as any).rate_curves_count ?? '?'}</div>
            <div class="kl">Courbes de taux</div>
          </div>
          <div class="kpi">
            <div class="kv">{(selected.meta as any).entities?.sellers ?? (selected.meta as any).entities?.branches != null ? Object.values((selected.meta as any).entities ?? {}).reduce((a: number, b: any) => a + (b as number), 0) : '?'}</div>
            <div class="kl">Entités org.</div>
          </div>
          <div class="kpi">
            <div class="kv">{(selected.meta as any).rate_series_count ?? '?'}</div>
            <div class="kl">Séries historiques</div>
          </div>
          <div class="kpi">
            <div class="kv kv-status" class:loaded={selected.loaded_in_db}>
              {selected.loaded_in_db ? 'En DB' : 'Non chargé'}
            </div>
            <div class="kl">Statut</div>
          </div>
          <div class="kpi">
            <div class="kv">{selected.meta.as_of_date ?? '—'}</div>
            <div class="kl">Date de référence</div>
          </div>
        </div>

        <!-- Contracts table (only when loaded) -->
        {#if selected.loaded_in_db}
          <div class="card">
            <div class="card-hdr">
              <span>Contrats en base ({contracts.length})</span>
              {#if loadingC}<span class="loading-inline">Chargement…</span>{/if}
            </div>
            {#if contracts.length === 0 && !loadingC}
              <div class="empty-state"><p>Aucun contrat lié à ce dataset.</p></div>
            {:else}
              <div class="tbl-wrap">
                <table>
                  <thead>
                    <tr>
                      <th>ID Contrat</th><th>Type</th><th>Côté</th><th>Branche</th>
                      <th>Devise</th><th>Notionnel</th><th>Taux</th><th>Ténor</th><th>Maturité</th>
                    </tr>
                  </thead>
                  <tbody>
                    {#each contracts as c}
                      <tr>
                        <td class="mono">{c.contract_id}</td>
                        <td><span class="tag">{c.contract_type}</span></td>
                        <td>
                          <span class="badge" class:badge-asset={c.side === 'ASSET'} class:badge-liab={c.side !== 'ASSET'}>
                            {c.side}
                          </span>
                        </td>
                        <td>{c.branch_code ?? '—'}</td>
                        <td>{c.currency}</td>
                        <td class="num">{fmtNum(c.notional)}</td>
                        <td class="num">{c.interest_rate != null ? (c.interest_rate * 100).toFixed(2) + '%' : '—'}</td>
                        <td class="num">{c.tenor_months != null ? c.tenor_months + ' m' : '—'}</td>
                        <td>{c.maturity_date ?? '—'}</td>
                      </tr>
                    {/each}
                  </tbody>
                </table>
              </div>
            {/if}
          </div>
        {:else}
          <div class="card hint-card">
            <p>Ce dataset n'est pas encore chargé en base de données.</p>
            <p>Cliquez sur <strong>Charger en DB</strong> pour importer les contrats, courbes de taux et modèles NMD.</p>
          </div>
        {/if}
      {/if}
    </section>
  </div>
</div>

<style>
.sub { margin: .1rem 0 0; font-size: .83rem; color: #888; }

.layout { display: flex; gap: 1.25rem; align-items: flex-start; }

/* ── Sidebar ── */
.sidebar {
  width: 280px; flex-shrink: 0;
  background: white; border-radius: 10px;
  box-shadow: 0 1px 4px rgba(0,0,0,.1); overflow: hidden;
}
.sidebar-hdr {
  padding: .6rem 1rem; background: #f8f9fa;
  border-bottom: 1px solid #eee; font-size: .8rem; font-weight: 600; color: #666;
  text-transform: uppercase; letter-spacing: .04em;
}
.ds-item {
  padding: .7rem 1rem; border-bottom: 1px solid #f2f2f2;
  cursor: pointer; transition: background .1s; border-left: 3px solid transparent;
}
.ds-item:hover { background: #f5f7ff; }
.ds-item.active { background: #eef0ff; border-left-color: #6366f1; }
.ds-row-top { display: flex; justify-content: space-between; align-items: center; gap: .5rem; }
.ds-name { font-size: .86rem; font-weight: 600; flex: 1; }
.ds-desc { font-size: .74rem; color: #777; margin-top: .15rem; line-height: 1.35; }
.ds-counts { display: flex; gap: .6rem; font-size: .72rem; color: #aaa; margin-top: .2rem; }
.empty-side { padding: 1.5rem 1rem; text-align: center; color: #bbb; font-size: .82rem; line-height: 1.6; }

/* ── Badges ── */
.badge {
  display: inline-block; padding: 1px 7px; border-radius: 99px;
  font-size: .71rem; font-weight: 600; white-space: nowrap;
}
.badge-db { background: #dcfce7; color: #166534; }
.badge-fs { background: #f1f5f9; color: #64748b; }
.badge-asset { background: #ede9fe; color: #5b21b6; }
.badge-liab { background: #fef3c7; color: #92400e; }

/* ── Detail ── */
.detail { flex: 1; display: flex; flex-direction: column; gap: 1rem; min-width: 0; }

.card { background: white; border-radius: 10px; box-shadow: 0 1px 4px rgba(0,0,0,.09); overflow: hidden; }
.detail-hdr { display: flex; justify-content: space-between; align-items: flex-start; padding: 1rem 1.25rem; gap: 1rem; }
.detail-info h3 { margin: 0; font-size: 1.05rem; }
.desc { margin: .2rem 0 .3rem; font-size: .82rem; color: #777; }
.folder-path { font-size: .74rem; color: #aaa; font-family: monospace; margin-top: .2rem; }

.action-btns { display: flex; gap: .5rem; flex-shrink: 0; }

/* ── Alerts ── */
.alert {
  padding: .65rem 1rem; border-radius: 8px; font-size: .84rem;
}
.alert-ok { background: #dcfce7; color: #166534; }
.alert-err { background: #fee2e2; color: #991b1b; }

/* ── KPIs ── */
.kpi-row { display: flex; gap: 1rem; }
.kpi {
  flex: 1; background: white; border-radius: 10px;
  padding: .85rem 1rem; text-align: center;
  box-shadow: 0 1px 4px rgba(0,0,0,.08);
}
.kv { font-size: 1.25rem; font-weight: 700; color: #6366f1; }
.kv-small { font-size: .78rem; font-weight: 600; line-height: 1.4; }
.kv-status { color: #94a3b8; }
.kv-status.loaded { color: #16a34a; }
.kl { font-size: .73rem; color: #999; margin-top: .1rem; }

/* ── Card header ── */
.card-hdr {
  display: flex; gap: 1rem; align-items: center;
  padding: .6rem 1rem; border-bottom: 1px solid #eee;
  font-size: .84rem; font-weight: 600;
}
.loading-inline { font-size: .79rem; color: #aaa; font-weight: 400; }

/* ── Hint card ── */
.hint-card { padding: 1.5rem; color: #666; font-size: .88rem; line-height: 1.6; }
.hint-card p { margin: 0 0 .5rem; }
.hint-card p:last-child { margin: 0; }

/* ── Table ── */
.tbl-wrap { overflow-x: auto; }
table { width: 100%; border-collapse: collapse; font-size: .81rem; }
thead { background: #f8f9fa; }
th {
  padding: .42rem .75rem; text-align: left;
  font-size: .74rem; text-transform: uppercase; letter-spacing: .04em;
  color: #888; font-weight: 600; border-bottom: 1px solid #eee; white-space: nowrap;
}
td { padding: .38rem .75rem; border-bottom: 1px solid #f4f4f4; }
tr:last-child td { border-bottom: none; }
tr:hover td { background: #fafbff; }
.mono { font-family: monospace; font-size: .77rem; }
.num { text-align: right; font-variant-numeric: tabular-nums; }
.tag {
  display: inline-block; padding: 1px 6px; border-radius: 4px;
  background: #f0f0f0; font-size: .74rem; font-weight: 500; color: #555;
}

/* ── Buttons ── */
.btn-primary {
  padding: .45rem .9rem; border-radius: 7px; border: none; cursor: pointer;
  background: #6366f1; color: white; font-weight: 600; font-size: .85rem;
  transition: background .12s;
}
.btn-primary:hover:not(:disabled) { background: #4f52d9; }
.btn-primary:disabled { opacity: .55; cursor: not-allowed; }

.btn-outline {
  padding: .42rem .85rem; border-radius: 7px; border: 1.5px solid #6366f1;
  background: transparent; color: #6366f1; font-weight: 600; font-size: .85rem;
  cursor: pointer; transition: background .12s;
}
.btn-outline:hover:not(:disabled) { background: #eef0ff; }
.btn-outline:disabled { opacity: .55; cursor: not-allowed; }

.btn-danger {
  padding: .42rem .85rem; border-radius: 7px; border: 1.5px solid #ef4444;
  background: transparent; color: #ef4444; font-weight: 600; font-size: .85rem;
  cursor: pointer; transition: background .12s;
}
.btn-danger:hover:not(:disabled) { background: #fee2e2; }
.btn-danger:disabled { opacity: .55; cursor: not-allowed; }

.btn-ghost {
  padding: .42rem .85rem; border-radius: 7px; border: 1px solid #ddd;
  background: white; color: #666; font-size: .85rem; cursor: pointer;
  transition: background .1s;
}
.btn-ghost:hover:not(:disabled) { background: #f5f5f5; }
.btn-ghost:disabled { opacity: .55; cursor: not-allowed; }

.empty-state { padding: 2.5rem; text-align: center; color: #bbb; font-size: .88rem; }
</style>
