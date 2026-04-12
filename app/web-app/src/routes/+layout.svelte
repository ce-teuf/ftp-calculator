<script lang="ts">
  import { page } from '$app/stores';
  import {
    LayoutDashboard, TrendingUp, Layers, Link, BookOpen,
    Play, Database, Box, Briefcase, ChevronDown, Activity, FlaskConical,
  } from '@lucide/svelte';

  // ── Nav structure ─────────────────────────────────────────────────────────────

  type NavItem  = { kind: 'link';  label: string; icon: any; href: string };
  type NavGroup = { kind: 'group'; label: string; icon: any; key: string;
                    children: { label: string; href: string }[] };
  type NavEntry = NavItem | NavGroup;

  const NAV: NavEntry[] = [
    {
      kind: 'group',
      key: 'rates',
      label: 'Taux',
      icon: TrendingUp,
      children: [
        { label: 'Matrices de taux', href: '/rate-matrices' },
        { label: 'Hypercubes',       href: '/hypercubes' },
      ],
    },
    { kind: 'link', label: 'Portfolios',   icon: Briefcase,     href: '/portfolios'   },
    { kind: 'link', label: 'Study Units',  icon: FlaskConical,  href: '/study-units'  },
    { kind: 'link', label: 'Studies',      icon: BookOpen,      href: '/studies'      },
    { kind: 'link', label: 'Exécutions',   icon: Play,          href: '/executions'   },
    { kind: 'link', label: 'Dashboard',    icon: LayoutDashboard, href: '/dashboard'  },
  ];

  // ── Group open state ──────────────────────────────────────────────────────────

  function groupIsActive(entry: NavGroup): boolean {
    return entry.children.some(c => $page.url.pathname.startsWith(c.href));
  }

  // Each group starts open if one of its children is the current route.
  let open = $state<Record<string, boolean>>(
    Object.fromEntries(
      NAV.filter((e): e is NavGroup => e.kind === 'group')
         .map(g => [g.key, g.children.some(c => $page.url.pathname.startsWith(c.href))])
    )
  );

  function toggle(key: string) {
    open[key] = !open[key];
  }

  function isActive(href: string): boolean {
    return $page.url.pathname.startsWith(href);
  }
</script>

<div class="app">
  <nav class="sidebar">
    <div class="brand">
      <span class="brand-icon">⚡</span>
      <span class="brand-name">FTP Simulator</span>
    </div>

    <div class="nav-body">
      {#each NAV as entry}
        {#if entry.kind === 'link'}
          <a
            class="nav-item"
            class:nav-item--active={isActive(entry.href)}
            href={entry.href}
          >
            <span class="nav-icon"><entry.icon size={16} /></span>
            <span class="nav-label">{entry.label}</span>
          </a>

        {:else}
          <!-- Group header -->
          <button
            class="nav-group-header"
            class:nav-group-header--active={groupIsActive(entry)}
            onclick={() => toggle(entry.key)}
          >
            <span class="nav-icon"><entry.icon size={16} /></span>
            <span class="nav-label">{entry.label}</span>
            <span class="chevron" class:chevron--open={open[entry.key]}>
              <ChevronDown size={13} />
            </span>
          </button>

          <!-- Children -->
          {#if open[entry.key]}
            <div class="nav-children">
              {#each entry.children as child}
                <a
                  class="nav-child"
                  class:nav-child--active={isActive(child.href)}
                  href={child.href}
                >
                  {child.label}
                </a>
              {/each}
            </div>
          {/if}
        {/if}
      {/each}
    </div>

    <div class="sidebar-footer">
      <span class="version">v1.0.0</span>
    </div>
  </nav>

  <main class="content">
    <slot />
  </main>
</div>

<style>
  :global(*, *::before, *::after) { box-sizing: border-box; margin: 0; padding: 0; }
  :global(body) {
    font-family: 'Inter', system-ui, -apple-system, sans-serif;
    background: #f4f5f9;
    color: #1a1a2e;
    font-size: 14px;
    line-height: 1.5;
  }

  .app { display: flex; min-height: 100vh; }
  .content { flex: 1; min-height: 100vh; background: #f4f5f9; overflow-y: auto; }

  /* ── Sidebar ── */
  .sidebar {
    width: 220px;
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

  /* ── Top-level links ── */
  .nav-item {
    display: flex;
    align-items: center;
    gap: 10px;
    width: 100%;
    padding: 8px 10px;
    border-radius: 8px;
    background: transparent;
    color: #8888aa;
    font-size: 13.5px;
    font-weight: 500;
    text-decoration: none;
    transition: background 120ms, color 120ms;
    margin-bottom: 2px;
  }
  .nav-item:hover { background: rgba(255,255,255,.06); color: #cccce8; }
  .nav-item--active { background: rgba(99,102,241,.18); color: #a5b4fc; }
  .nav-item--active:hover { background: rgba(99,102,241,.22); }

  /* ── Group header ── */
  .nav-group-header {
    display: flex;
    align-items: center;
    gap: 10px;
    width: 100%;
    padding: 8px 10px;
    border: none;
    border-radius: 8px;
    background: transparent;
    color: #8888aa;
    font-size: 13.5px;
    font-weight: 500;
    cursor: pointer;
    transition: background 120ms, color 120ms;
    margin-bottom: 2px;
    text-align: left;
  }
  .nav-group-header:hover { background: rgba(255,255,255,.06); color: #cccce8; }
  .nav-group-header--active { color: #c4b5fd; }

  .chevron {
    margin-left: auto;
    display: flex;
    align-items: center;
    opacity: .5;
    transition: transform 180ms ease;
  }
  .chevron--open { transform: rotate(180deg); opacity: .8; }

  /* ── Children ── */
  .nav-children {
    margin-bottom: 4px;
    padding-left: 8px;
  }

  .nav-child {
    display: flex;
    align-items: center;
    padding: 6px 10px 6px 28px;
    border-radius: 6px;
    color: #6666aa;
    font-size: 13px;
    font-weight: 500;
    text-decoration: none;
    transition: background 120ms, color 120ms;
    margin-bottom: 1px;
    position: relative;
  }
  .nav-child::before {
    content: '';
    position: absolute;
    left: 12px;
    top: 50%;
    transform: translateY(-50%);
    width: 4px;
    height: 4px;
    border-radius: 50%;
    background: currentColor;
    opacity: .4;
  }
  .nav-child:hover { background: rgba(255,255,255,.05); color: #aaaacc; }
  .nav-child--active { color: #a5b4fc; background: rgba(99,102,241,.12); }
  .nav-child--active::before { opacity: 1; background: #a5b4fc; }

  /* ── Icon shared ── */
  .nav-icon { display: flex; align-items: center; flex-shrink: 0; opacity: .8; }
  .nav-item--active .nav-icon,
  .nav-group-header--active .nav-icon { opacity: 1; }
  .nav-label { flex: 1; }

  /* ── Footer ── */
  .sidebar-footer {
    padding: 12px 18px;
    border-top: 1px solid rgba(255,255,255,.06);
  }
  .version { font-size: 11px; color: #444466; }

  /* ── Global design tokens ── */
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
  :global(.card) {
    background: #fff;
    border-radius: 12px;
    box-shadow: 0 1px 3px rgba(0,0,0,.06), 0 1px 2px rgba(0,0,0,.04);
  }
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
  :global(.btn-primary:hover) { background: #4f46e5; }
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
  :global(.btn-sm:hover) { background: #e4e4ee; }
  :global(.btn-sm.btn-danger) { background: #fef2f2; color: #b91c1c; }
  :global(.btn-sm.btn-danger:hover) { background: #fee2e2; }
  :global(.btn-sm.btn-success) { background: #f0fdf4; color: #15803d; }
  :global(.btn-sm.btn-success:hover) { background: #dcfce7; }
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
  :global(.tag) {
    display: inline-block;
    background: #ede9fe;
    color: #5b21b6;
    padding: 2px 8px;
    border-radius: 6px;
    font-size: 11.5px;
    font-weight: 600;
  }
  :global(.alert-error) {
    background: #fee2e2;
    color: #991b1b;
    padding: 10px 14px;
    border-radius: 8px;
    margin-bottom: 14px;
    font-size: 13px;
    border-left: 3px solid #ef4444;
  }
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
