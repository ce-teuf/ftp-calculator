<script lang="ts">
  import { curves, runoff, executions,
           type RateCurve, type RunoffModel, type Execution } from '../api/client.ts';

  let curvesData   = $state<RateCurve[]>([]);
  let runoffData   = $state<RunoffModel[]>([]);
  let execData     = $state<Execution[]>([]);
  let loading      = $state(true);
  let activeFilter = $state<'all' | 'pending' | 'approved'>('all');

  async function load() {
    loading = true;
    try {
      [curvesData, runoffData, execData] = await Promise.all([
        curves.list(), runoff.list(), executions.list(),
      ]);
    } catch { /* ignore */ }
    finally { loading = false; }
  }

  async function approveCurve(id: string) {
    await curves.update(id, { status: 'approved' }); await load();
  }
  async function archiveCurve(id: string) {
    await curves.update(id, { status: 'archived' }); await load();
  }

  const pendingCurves  = $derived(curvesData.filter(c => c.status === 'draft'));
  const approvedCurves = $derived(curvesData.filter(c => c.status === 'approved'));
  const archivedCurves = $derived(curvesData.filter(c => c.status === 'archived'));
  const pendingRunoff  = $derived(runoffData.filter(r => r.status === 'draft'));
  const totalPending   = $derived(pendingCurves.length + pendingRunoff.length);

  const auditRows = $derived(
    execData.slice(0, 50).map(e => ({
      when:   e.created_at?.slice(0,19).replace('T',' ') ?? '—',
      action: 'Calcul FTP',
      method: e.method,
      ref:    e.id.slice(0, 8),
      status: e.status,
      label:  e.label ?? '',
    }))
  );

  const displayedCurves = $derived(
    activeFilter === 'pending'  ? pendingCurves  :
    activeFilter === 'approved' ? approvedCurves :
    curvesData
  );

  $effect(() => { load(); });
</script>

<div class="governance-layout">

  <!-- ── En-tête ── -->
  <div class="gov-header">
    <div>
      <h2>Gouvernance & Approbation ALCO</h2>
      <p class="subtitle">Validation des courbes, modèles de runoff, piste d'audit</p>
    </div>
    {#if totalPending > 0}
      <div class="pending-badge">{totalPending} élément{totalPending > 1 ? 's' : ''} en attente</div>
    {/if}
  </div>

  <!-- ── KPIs rapides ── -->
  <div class="kpi-row">
    <div class="kpi" class:kpi-warn={pendingCurves.length > 0}>
      <div class="kpi-label">Courbes en attente</div>
      <div class="kpi-value">{pendingCurves.length}</div>
    </div>
    <div class="kpi">
      <div class="kpi-label">Courbes approuvées</div>
      <div class="kpi-value kpi-ok">{approvedCurves.length}</div>
    </div>
    <div class="kpi">
      <div class="kpi-label">Courbes archivées</div>
      <div class="kpi-value">{archivedCurves.length}</div>
    </div>
    <div class="kpi" class:kpi-warn={pendingRunoff.length > 0}>
      <div class="kpi-label">Modèles runoff en attente</div>
      <div class="kpi-value">{pendingRunoff.length}</div>
    </div>
    <div class="kpi">
      <div class="kpi-label">Exécutions (total)</div>
      <div class="kpi-value">{execData.length}</div>
    </div>
  </div>

  <!-- ── Courbes de taux ── -->
  <div class="card">
    <div class="card-header">
      <h3>Courbes de taux</h3>
      <div class="filter-tabs">
        {#each (['all', 'pending', 'approved'] as const) as f}
          <button class="filter-btn" class:active={activeFilter === f}
                  onclick={() => activeFilter = f}>
            {f === 'all' ? 'Toutes' : f === 'pending' ? 'En attente' : 'Approuvées'}
          </button>
        {/each}
      </div>
    </div>

    {#if loading}
      <div class="loading">Chargement…</div>
    {:else if displayedCurves.length === 0}
      <div class="empty">Aucune courbe dans cette catégorie.</div>
    {:else}
      <table>
        <thead>
          <tr>
            <th>Nom</th>
            <th>Composante</th>
            <th>Devise</th>
            <th>Source</th>
            <th>Statut</th>
            <th>Actions</th>
          </tr>
        </thead>
        <tbody>
          {#each displayedCurves as c}
            <tr>
              <td class="name">{c.name}</td>
              <td><span class="pill pill-{c.component}">{c.component}</span></td>
              <td>{c.currency}</td>
              <td>{c.source ?? '—'}</td>
              <td>
                <span class="status-badge" class:draft={c.status==='draft'}
                      class:approved={c.status==='approved'} class:archived={c.status==='archived'}>
                  {c.status}
                </span>
              </td>
              <td class="actions">
                {#if c.status === 'draft'}
                  <button class="btn-approve" onclick={() => approveCurve(c.id)}>✓ Approuver</button>
                {/if}
                {#if c.status === 'approved'}
                  <button class="btn-archive" onclick={() => archiveCurve(c.id)}>Archiver</button>
                {/if}
              </td>
            </tr>
          {/each}
        </tbody>
      </table>
    {/if}
  </div>

  <!-- ── Modèles runoff en attente ── -->
  {#if pendingRunoff.length > 0}
    <div class="card">
      <h3>Modèles de runoff en attente ({pendingRunoff.length})</h3>
      <table>
        <thead>
          <tr><th>Nom</th><th>Produit</th><th>Méthode</th><th>Catégorie</th><th>Créé le</th></tr>
        </thead>
        <tbody>
          {#each pendingRunoff as r}
            <tr>
              <td class="name">{r.name}</td>
              <td>{r.product_type}</td>
              <td>{r.method}</td>
              <td>{r.category ?? '—'}</td>
              <td>{r.created_at?.slice(0,10) ?? '—'}</td>
            </tr>
          {/each}
        </tbody>
      </table>
    </div>
  {/if}

  <!-- ── Piste d'audit ── -->
  <div class="card">
    <h3>Piste d'audit — dernières exécutions</h3>
    {#if auditRows.length === 0}
      <div class="empty">Aucune exécution enregistrée.</div>
    {:else}
      <table>
        <thead>
          <tr><th>Quand</th><th>Action</th><th>Méthode</th><th>Libellé</th><th>Référence</th><th>Statut</th></tr>
        </thead>
        <tbody>
          {#each auditRows as row}
            <tr>
              <td class="mono">{row.when}</td>
              <td>{row.action}</td>
              <td>{row.method.toUpperCase()}</td>
              <td>{row.label}</td>
              <td class="mono">{row.ref}</td>
              <td>
                <span class="status-badge" class:approved={row.status==='completed'}
                      class:draft={row.status==='error'}>
                  {row.status}
                </span>
              </td>
            </tr>
          {/each}
        </tbody>
      </table>
    {/if}
  </div>

</div>

<style>
.governance-layout { display: flex; flex-direction: column; gap: 1.25rem; }
.gov-header { display: flex; justify-content: space-between; align-items: flex-start; }
.gov-header h2 { margin: 0 0 .2rem; font-size: 1.1rem; }
.subtitle { margin: 0; font-size: .82rem; color: #666; }
.pending-badge { background: #fbbc04; color: #5f4200; padding: .3rem .9rem;
  border-radius: 20px; font-size: .82rem; font-weight: 700; }

.kpi-row { display: grid; grid-template-columns: repeat(5, 1fr); gap: .75rem; }
.kpi { background: #fff; border-radius: 10px; padding: .75rem; text-align: center;
  box-shadow: 0 1px 4px rgba(0,0,0,.06); }
.kpi-label { font-size: .75rem; color: #666; margin-bottom: .3rem; }
.kpi-value { font-size: 1.5rem; font-weight: 700; color: #333; }
.kpi-ok { color: #34a853; }
.kpi-warn .kpi-value { color: #ea8600; }

.card { background: #fff; border-radius: 12px; padding: 1.25rem; box-shadow: 0 1px 4px rgba(0,0,0,.08); }
.card h3 { margin: 0 0 .75rem; font-size: .95rem; font-weight: 600; }
.card-header { display: flex; justify-content: space-between; align-items: center; margin-bottom: .75rem; }
.card-header h3 { margin: 0; }
.filter-tabs { display: flex; gap: .3rem; }
.filter-btn { border: 1px solid #ddd; background: #fff; border-radius: 20px;
  padding: .25rem .75rem; font-size: .8rem; cursor: pointer; }
.filter-btn.active { background: #1a73e8; color: #fff; border-color: #1a73e8; }

table { width: 100%; border-collapse: collapse; font-size: .83rem; }
th, td { border-bottom: 1px solid #f0f0f0; padding: .4rem .7rem; }
th { background: #f8f9fa; font-weight: 600; color: #444; }
.name { font-weight: 500; }
.mono { font-family: monospace; font-size: .8rem; color: #555; }
.actions { white-space: nowrap; }
.loading { color: #888; padding: 1rem; text-align: center; }
.empty { color: #999; padding: 1rem; text-align: center; font-size: .85rem; }

.pill { border-radius: 20px; padding: .2rem .6rem; font-size: .75rem; font-weight: 500; }
.pill-base_rate, .pill-ois { background: #e3f2fd; color: #0d47a1; }
.pill-credit_spread { background: #fce4ec; color: #880e4f; }
.pill-tlp { background: #e8f5e9; color: #1b5e20; }
.pill-clp, .pill-capital_charge { background: #fff3e0; color: #e65100; }

.status-badge { border-radius: 20px; padding: .15rem .6rem; font-size: .75rem; font-weight: 500; }
.status-badge.draft { background: #fff3e0; color: #e65100; }
.status-badge.approved { background: #e8f5e9; color: #1b5e20; }
.status-badge.archived { background: #f5f5f5; color: #757575; }

.btn-approve { background: #e8f5e9; color: #1b5e20; border: 1px solid #a5d6a7;
  border-radius: 6px; padding: .2rem .6rem; font-size: .78rem; cursor: pointer; }
.btn-approve:hover { background: #c8e6c9; }
.btn-archive { background: #f5f5f5; color: #616161; border: 1px solid #e0e0e0;
  border-radius: 6px; padding: .2rem .6rem; font-size: .78rem; cursor: pointer; }
.btn-archive:hover { background: #eeeeee; }
</style>
