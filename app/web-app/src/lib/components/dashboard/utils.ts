import type { AssignmentResult } from '$lib/api/client';

// ── Palette ───────────────────────────────────────────────────────────────────
export const C_OBS    = '#6366f1';
export const C_PROJ   = '#f97316';
export const C_CONTRA = '#7c3aed';
export const C_FLUX   = '#16a34a';
export const C_AREA_PROJ   = 'rgba(254,243,199,0.35)';
export const C_AREA_CONTRA = 'rgba(237,233,254,0.35)';
export const SERIES_COLORS = ['#6366f1','#f97316','#16a34a','#0ea5e9','#ec4899','#eab308','#14b8a6'];

// ── KPI aggregate type ────────────────────────────────────────────────────────
export interface KpiData {
  obsTotalOut:  number;
  projTotalOut: number;
  obsAvgRate:   number;
  projAvgRate:  number;
  obsInt:       number;
  projInt:      number;
  wal:          number;
  method:       string;
}

// ── ECharts helpers ───────────────────────────────────────────────────────────
export function hexToRgb(hex: string): string {
  const r = parseInt(hex.slice(1,3),16);
  const g = parseInt(hex.slice(3,5),16);
  const b = parseInt(hex.slice(5,7),16);
  return `${r},${g},${b}`;
}

export function buildMarkArea(dates: string[], assignments: AssignmentResult[]) {
  const projStart   = dates.find(d => assignments.some(a => a.time_steps.find(t => t.date === d && t.period_type === 'projected')));
  const contraStart = dates.find(d => assignments.some(a => a.time_steps.find(t => t.date === d && t.period_type === 'contrafactual')));
  const areas: any[] = [];
  if (projStart)   areas.push([{ xAxis: projStart,   itemStyle: { color: C_AREA_PROJ   } }, { xAxis: dates[dates.length - 1] }]);
  if (contraStart) areas.push([{ xAxis: contraStart, itemStyle: { color: C_AREA_CONTRA } }, { xAxis: dates[dates.length - 1] }]);
  return { silent: true, data: areas };
}

export function buildMarkLine(boundary: string | null) {
  if (!boundary) return { data: [] };
  return {
    silent: true, symbol: 'none',
    data: [{ xAxis: boundary, lineStyle: { color: C_PROJ, type: 'dashed', width: 2 } }],
    label: { formatter: 'Projeté →', position: 'insideEndTop', color: C_PROJ, fontSize: 10 },
  };
}

// ── Formatters ────────────────────────────────────────────────────────────────
export function ptColor(pt: string): string {
  return pt === 'projected' ? C_PROJ : pt === 'contrafactual' ? C_CONTRA : C_OBS;
}
export function ptLabel(pt: string): string {
  return pt === 'projected' ? 'Proj.' : pt === 'contrafactual' ? 'Contra.' : 'Obs.';
}
export function fmtPct(v: number): string { return (v * 100).toFixed(4) + ' %'; }
export function fmtM(v: number):   string { return (v / 1e6).toLocaleString('fr-FR', { maximumFractionDigits: 2 }) + ' M€'; }
export function fmtAmt(v: number): string { return v.toLocaleString('fr-FR', { maximumFractionDigits: 0 }) + ' €'; }
