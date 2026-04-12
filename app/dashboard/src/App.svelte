<script lang="ts">
  import CurvesTab       from './lib/components/CurvesTab.svelte';
  import StacksTab       from './lib/components/StacksTab.svelte';
  import CubesTab        from './lib/components/CubesTab.svelte';
  import PortfolioV3Tab  from './lib/components/PortfolioV3Tab.svelte';
  import LinkersTab      from './lib/components/LinkersTab.svelte';
  import StudiesTab      from './lib/components/StudiesTab.svelte';
  import ExecutionsV3Tab  from './lib/components/ExecutionsV3Tab.svelte';
  import DashboardV3Tab   from './lib/components/DashboardV3Tab.svelte';
  import RateSeriesTab   from './lib/components/RateSeriesTab.svelte';

  import {
    LayoutDashboard, TrendingUp, Layers, Link, BookOpen,
    Play, Database, Briefcase, Box,
  } from '@lucide/svelte';

  type Tab =
    | 'dashboard'
    | 'curves'
    | 'stacks'
    | 'cubes'
    | 'portfolios'
    | 'linkers'
    | 'studies'
    | 'executions'
    | 'rate-series';

  let activeTab = $state<Tab>('curves');
  let openGroup = $state<string | null>(null);

  type NavLeaf  = { key: Tab; label: string; icon: any };
  type NavGroup = { group: string; label: string; icon: any; children: NavLeaf[] };
  type NavItem  = NavLeaf | NavGroup;

  const NAV: NavItem[] = [
    { key: 'dashboard',   label: 'Dashboard',      icon: LayoutDashboard },
    { key: 'rate-series', label: 'Rate Series',     icon: Database        },
    { key: 'curves',      label: 'Curves',         icon: TrendingUp      },
    { key: 'stacks',      label: 'Stacks',          icon: Layers          },
    { key: 'cubes',       label: 'Cubes',           icon: Box             },
    { key: 'portfolios',  label: 'Portfolios',      icon: Briefcase       },
    { key: 'linkers',     label: 'Linkers',         icon: Link            },
    { key: 'studies',     label: 'Studies',         icon: BookOpen        },
    { key: 'executions',  label: 'Executions',      icon: Play            },
  ];

  function isGroup(item: NavItem): item is NavGroup {
    return 'group' in item;
  }

  function navigate(tab: Tab) {
    activeTab = tab;
    // Auto-open parent group
    for (const item of NAV) {
      if (isGroup(item) && item.children.some(c => c.key === tab)) {
        openGroup = item.group;
        break;
      }
    }
  }

  function toggleGroup(groupKey: string) {
    openGroup = openGroup === groupKey ? null : groupKey;
  }

  // Derived: is any child of a group active?
  function groupHasActive(item: NavGroup): boolean {
    return item.children.some(c => c.key === activeTab);
  }
</script>

<div class="app">
  <!-- ─── Sidebar ─────────────────────────────────────── -->
  <nav class="sidebar">
    <div class="brand">
      <span class="brand-icon">⚡</span>
      <span class="brand-name">FTP Simulator</span>
    </div>

    <div class="nav-body">
      {#each NAV as item}
        {#if isGroup(item)}
          <!-- Expandable group -->
          <div class="nav-group">
            <button
              class="nav-item nav-parent"
              class:nav-parent--open={openGroup === item.group}
              class:nav-parent--active={groupHasActive(item)}
              onclick={() => toggleGroup(item.group)}
              aria-expanded={openGroup === item.group}
            >
              <span class="nav-icon"><item.icon size={16} /></span>
              <span class="nav-label">{item.label}</span>
              <span class="nav-chevron" class:rotated={openGroup === item.group}>
                <ChevronRight size={14} />
              </span>
            </button>

            {#if openGroup === item.group}
              <div class="nav-children">
                {#each item.children as child}
                  <button
                    class="nav-item nav-child"
                    class:nav-item--active={activeTab === child.key}
                    onclick={() => navigate(child.key)}
                  >
                    <span class="nav-icon child-icon"><child.icon size={14} /></span>
                    <span class="nav-label">{child.label}</span>
                  </button>
                {/each}
              </div>
            {/if}
          </div>
        {:else}
          <!-- Simple leaf -->
          <button
            class="nav-item"
            class:nav-item--active={activeTab === item.key}
            onclick={() => navigate(item.key)}
          >
            <span class="nav-icon"><item.icon size={16} /></span>
            <span class="nav-label">{item.label}</span>
          </button>
        {/if}
      {/each}
    </div>

    <div class="sidebar-footer">
      <span class="version">v1.0.0</span>
    </div>
  </nav>

  <!-- ─── Main content ────────────────────────────────── -->
  <main class="content">
    {#if activeTab === 'dashboard'}
      <DashboardV3Tab />
    {:else if activeTab === 'rate-series'}
      <RateSeriesTab />
    {:else if activeTab === 'curves'}
      <CurvesTab />
    {:else if activeTab === 'stacks'}
      <StacksTab />
    {:else if activeTab === 'cubes'}
      <CubesTab />
    {:else if activeTab === 'portfolios'}
      <PortfolioV3Tab />
    {:else if activeTab === 'linkers'}
      <LinkersTab />
    {:else if activeTab === 'studies'}
      <StudiesTab />
    {:else if activeTab === 'executions'}
      <ExecutionsV3Tab />
    {:else}
      <div class="tab-content">
        <div class="empty-state" style="margin-top:40px">
          <p>Module en construction…</p>
        </div>
      </div>
    {/if}
  </main>
</div>

<style>
  /* ── Reset & base ──────────────────────────────────────────────────────────── */
  :global(*, *::before, *::after) { box-sizing: border-box; margin: 0; padding: 0; }
  :global(body) {
    font-family: 'Inter', system-ui, -apple-system, sans-serif;
    background: #f4f5f9;
    color: #1a1a2e;
    font-size: 14px;
    line-height: 1.5;
  }

  /* ── Layout ────────────────────────────────────────────────────────────────── */
  .app    { display: flex; min-height: 100vh; }
  .content {
    flex: 1;
    min-height: 100vh;
    background: #f4f5f9;
    overflow-y: auto;
  }

  /* ── Sidebar ───────────────────────────────────────────────────────────────── */
  .sidebar {
    width: 228px;
    flex-shrink: 0;
    background: #16162a;
    display: flex;
    flex-direction: column;
    position: sticky;
    top: 0;
    height: 100vh;
    overflow: hidden;
  }

  .brand {
    display: flex;
    align-items: center;
    gap: 10px;
    padding: 20px 18px 16px;
    border-bottom: 1px solid rgba(255,255,255,.06);
  }
  .brand-icon { font-size: 20px; }
  .brand-name { font-size: 15px; font-weight: 700; color: #fff; letter-spacing: .01em; }

  .nav-body { flex: 1; overflow-y: auto; padding: 10px 8px; }

  /* ── Nav item (shared) ─────────────────────────────────────────────────────── */
  .nav-item {
    display: flex;
    align-items: center;
    gap: 10px;
    width: 100%;
    padding: 8px 10px;
    border: none;
    border-radius: 8px;
    background: transparent;
    color: #8888aa;
    text-align: left;
    cursor: pointer;
    font-size: 13.5px;
    font-weight: 500;
    transition: background 120ms, color 120ms;
    margin-bottom: 2px;
  }
  .nav-item:hover { background: rgba(255,255,255,.06); color: #cccce8; }
  .nav-item--active {
    background: rgba(99, 102, 241, .18);
    color: #a5b4fc;
  }
  .nav-item--active:hover { background: rgba(99, 102, 241, .22); color: #a5b4fc; }

  /* ── Nav icon ──────────────────────────────────────────────────────────────── */
  .nav-icon { display: flex; align-items: center; flex-shrink: 0; opacity: .8; }
  .nav-item--active .nav-icon { opacity: 1; }

  /* ── Nav label ─────────────────────────────────────────────────────────────── */
  .nav-label { flex: 1; }

  /* ── Group parent button ───────────────────────────────────────────────────── */
  .nav-parent { font-weight: 600; }
  .nav-parent--active { color: #cccce8; }

  .nav-chevron {
    display: flex;
    align-items: center;
    color: #555577;
    transition: transform 180ms ease;
  }
  .nav-chevron.rotated { transform: rotate(90deg); }

  /* ── Children ──────────────────────────────────────────────────────────────── */
  .nav-group { margin-bottom: 2px; }
  .nav-children {
    margin: 2px 0 4px 12px;
    padding-left: 10px;
    border-left: 1px solid rgba(255,255,255,.08);
  }
  .nav-child { font-size: 13px; font-weight: 400; padding: 6px 10px; }
  .child-icon { opacity: .65; }
  .nav-child.nav-item--active .child-icon { opacity: 1; }

  /* ── Sidebar footer ────────────────────────────────────────────────────────── */
  .sidebar-footer {
    padding: 12px 18px;
    border-top: 1px solid rgba(255,255,255,.06);
  }
  .version { font-size: 11px; color: #444466; }

  /* ═══════════════════════════════════════════════════════════════════════════
     GLOBAL DESIGN TOKENS — used by all tab components
  ═══════════════════════════════════════════════════════════════════════════ */

  /* Tab wrapper */
  :global(.tab-content) { padding: 28px 32px; }
  :global(.tab-header) {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 24px;
  }
  :global(.tab-header h2) {
    font-size: 20px;
    font-weight: 700;
    color: #1a1a2e;
    letter-spacing: -.01em;
  }

  /* Card */
  :global(.card) {
    background: #fff;
    border-radius: 12px;
    box-shadow: 0 1px 3px rgba(0,0,0,.06), 0 1px 2px rgba(0,0,0,.04);
  }

  /* Buttons */
  :global(.btn-primary) {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    background: #6366f1;
    color: #fff;
    border: none;
    border-radius: 8px;
    padding: 8px 16px;
    font-size: 13.5px;
    font-weight: 600;
    cursor: pointer;
    transition: background 120ms;
    white-space: nowrap;
    text-decoration: none;
  }
  :global(.btn-primary:hover)    { background: #4f46e5; }
  :global(.btn-primary:disabled) { background: #c7c7d4; cursor: not-allowed; }

  :global(.btn-sm) {
    display: inline-flex;
    align-items: center;
    gap: 4px;
    background: #f1f1f5;
    color: #444;
    border: none;
    border-radius: 6px;
    padding: 4px 10px;
    font-size: 12.5px;
    font-weight: 500;
    cursor: pointer;
    transition: background 120ms;
    white-space: nowrap;
  }
  :global(.btn-sm:hover)           { background: #e4e4ee; }
  :global(.btn-sm.btn-danger)      { background: #fef2f2; color: #b91c1c; }
  :global(.btn-sm.btn-danger:hover){ background: #fee2e2; }
  :global(.btn-sm.btn-success)     { background: #f0fdf4; color: #15803d; }
  :global(.btn-sm.btn-success:hover){ background: #dcfce7; }

  /* Badges */
  :global(.badge) {
    display: inline-block;
    padding: 2px 8px;
    border-radius: 20px;
    font-size: 11.5px;
    font-weight: 600;
    line-height: 1.6;
  }
  :global(.badge-draft)     { background: #fef3c7; color: #92400e; }
  :global(.badge-active)    { background: #d1fae5; color: #065f46; }
  :global(.badge-approved)  { background: #d1fae5; color: #065f46; }
  :global(.badge-archived)  { background: #f3f4f6; color: #6b7280; }
  :global(.badge-frozen)    { background: #e0e7ff; color: #3730a3; }
  :global(.badge-pending)   { background: #dbeafe; color: #1e40af; }
  :global(.badge-completed) { background: #d1fae5; color: #065f46; }
  :global(.badge-error)     { background: #fee2e2; color: #991b1b; }

  /* Tag (smaller label) */
  :global(.tag) {
    display: inline-block;
    background: #ede9fe;
    color: #5b21b6;
    padding: 2px 8px;
    border-radius: 6px;
    font-size: 11.5px;
    font-weight: 600;
  }

  /* Alerts */
  :global(.alert-error) {
    background: #fee2e2;
    color: #991b1b;
    padding: 10px 14px;
    border-radius: 8px;
    margin-bottom: 14px;
    font-size: 13px;
    border-left: 3px solid #ef4444;
  }

  /* States */
  :global(.loading) { color: #9ca3af; font-style: italic; font-size: 13px; }
  :global(.empty-state) {
    background: #fafafa;
    border: 2px dashed #e5e7eb;
    border-radius: 12px;
    padding: 48px 32px;
    text-align: center;
    color: #9ca3af;
    font-size: 13px;
  }
  :global(.empty-state p) { margin-bottom: 8px; }

  /* Forms */
  :global(label) {
    display: flex;
    flex-direction: column;
    gap: 4px;
    font-size: 12.5px;
    font-weight: 500;
    color: #374151;
  }
  :global(input), :global(select), :global(textarea) {
    border: 1px solid #e5e7eb;
    border-radius: 8px;
    padding: 7px 11px;
    font-size: 13.5px;
    width: 100%;
    background: #fff;
    color: #1a1a2e;
    transition: border-color 120ms, box-shadow 120ms;
  }
  :global(input:focus), :global(select:focus), :global(textarea:focus) {
    outline: none;
    border-color: #6366f1;
    box-shadow: 0 0 0 3px rgba(99,102,241,.12);
  }
</style>
