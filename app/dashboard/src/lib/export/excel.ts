/**
 * Excel export using SheetJS (xlsx).
 *
 * Generates a multi-tab workbook from an execution result:
 *  - Portefeuille   : input positions
 *  - FTP Rates      : ftp_rate matrix
 *  - FTP Interest   : ftp_int matrix
 *  - Stock Amort    : stock_amort matrix
 *  - Market Rates   : market_rate matrix
 *  - NIM            : NIM per position (client_rate - ftp_rate)
 *  - KPIs           : summary
 */
import * as XLSX from 'xlsx';
import type { ComputeResponse } from '../api/client.ts';

export interface ExportPosition {
  position_ref?: string;
  product_type: string;
  branch?: string;
  seller?: string;
  currency: string;
  outstanding: number;
  client_rate?: number;
}

export function exportExecutionExcel(
  result: ComputeResponse,
  positions: ExportPosition[],
  tenors: string[],
  filename = 'ftp-export.xlsx',
) {
  const wb = XLSX.utils.book_new();

  // Helper: matrix sheet
  const matSheet = (mat: number[][] | undefined, label: string) => {
    if (!mat) return;
    const header = ['Position', ...tenors];
    const rows = mat.map((row, i) => [
      positions[i]?.position_ref ?? `Pos ${i+1}`,
      ...row.map(v => parseFloat((v * 100).toFixed(6))),
    ]);
    const ws = XLSX.utils.aoa_to_sheet([header, ...rows]);
    // Style header row (bold)
    XLSX.utils.book_append_sheet(wb, ws, label);
  };

  // Portefeuille
  const ptfHeader = ['Réf','Type','Branche','Vendeur','Devise','Encours','Taux client'];
  const ptfRows = positions.map(p => [
    p.position_ref ?? '',
    p.product_type,
    p.branch ?? '',
    p.seller ?? '',
    p.currency,
    p.outstanding,
    p.client_rate != null ? parseFloat((p.client_rate * 100).toFixed(4)) : '',
  ]);
  XLSX.utils.book_append_sheet(wb, XLSX.utils.aoa_to_sheet([ptfHeader, ...ptfRows]), 'Portefeuille');

  // FTP matrices
  matSheet(result.ftp_rate,    'FTP Rates (%)');
  matSheet(result.ftp_int,     'FTP Interest');
  matSheet(result.stock_amort, 'Stock Amort');
  matSheet(result.market_rate, 'Market Rates (%)');

  // NIM per position
  if (result.ftp_rate) {
    const nimHeader = ['Réf','Type','Encours','Taux client','Taux FTP t=0','NIM bps'];
    const nimRows = result.ftp_rate.map((row, i) => {
      const cr = positions[i]?.client_rate ?? null;
      const ftp = row[0] ?? null;
      const nim = cr != null && ftp != null ? (cr - ftp) * 10000 : '';
      return [
        positions[i]?.position_ref ?? `Pos ${i+1}`,
        positions[i]?.product_type ?? '',
        positions[i]?.outstanding ?? '',
        cr != null ? parseFloat((cr*100).toFixed(4)) : '',
        ftp != null ? parseFloat((ftp*100).toFixed(4)) : '',
        nim !== '' ? parseFloat((nim as number).toFixed(1)) : '',
      ];
    });
    XLSX.utils.book_append_sheet(wb, XLSX.utils.aoa_to_sheet([nimHeader, ...nimRows]), 'NIM');
  }

  // KPIs
  const kpiSheet = XLSX.utils.aoa_to_sheet([
    ['KPI', 'Valeur'],
    ['Méthode', result.method],
    ['Encours total', result.total_outstanding],
    ['Taux FTP pondéré (%)', parseFloat((result.weighted_ftp_rate * 100).toFixed(4))],
    ['Int. FTP mensuel', result.total_ftp_int_monthly],
    ['Durée calcul (ms)', result.duration_ms],
    ['Exécution ID', result.execution_id],
  ]);
  XLSX.utils.book_append_sheet(wb, kpiSheet, 'KPIs');

  XLSX.writeFile(wb, filename);
}

/** Export raw portfolio positions to Excel */
export function exportPortfolioExcel(positions: ExportPosition[], filename = 'portefeuille.xlsx') {
  const header = ['Réf','Type','Branche','Vendeur','Devise','Encours','Taux client'];
  const rows = positions.map(p => [
    p.position_ref ?? '',
    p.product_type,
    p.branch ?? '',
    p.seller ?? '',
    p.currency,
    p.outstanding,
    p.client_rate != null ? parseFloat((p.client_rate*100).toFixed(4)) : '',
  ]);
  const wb = XLSX.utils.book_new();
  XLSX.utils.book_append_sheet(wb, XLSX.utils.aoa_to_sheet([header, ...rows]), 'Portefeuille');
  XLSX.writeFile(wb, filename);
}

/** Download a portfolio template Excel file */
export function downloadTemplate() {
  const header = ['position_ref','product_type','branch','seller','currency','outstanding','client_rate'];
  const example = [['LOAN-001','mortgage','Nord','Dupont','EUR',500000,0.0425]];
  const wb = XLSX.utils.book_new();
  XLSX.utils.book_append_sheet(wb, XLSX.utils.aoa_to_sheet([header, ...example]), 'Template');
  XLSX.writeFile(wb, 'ftp-template.xlsx');
}
