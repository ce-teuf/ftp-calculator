import * as XLSX from 'xlsx';

export function exportToExcel(data: any, filename: string) {
  const ws = XLSX.utils.json_to_sheet(data);
  const wb = XLSX.utils.book_new();
  XLSX.utils.book_append_sheet(wb, ws, 'Data');
  XLSX.writeFile(wb, `${filename}.xlsx`);
}

export function exportPortfolioToExcel(portfolio: any[], curves: any[]) {
  const wb = XLSX.utils.book_new();
  
  const wsPortfolio = XLSX.utils.json_to_sheet(portfolio);
  XLSX.utils.book_append_sheet(wb, wsPortfolio, 'Portfolio');
  
  const wsCurves = XLSX.utils.json_to_sheet(curves);
  XLSX.utils.book_append_sheet(wb, wsCurves, 'Curves');
  
  XLSX.writeFile(wb, 'ftp-export.xlsx');
}

export function exportCsv(data: any[], filename: string) {
  const csv = XLSX.utils.sheet_to_csv(XLSX.utils.json_to_sheet(data));
  const blob = new Blob([csv], { type: 'text/csv' });
  const url = URL.createObjectURL(blob);
  const a = document.createElement('a');
  a.href = url;
  a.download = `${filename}.csv`;
  a.click();
  URL.revokeObjectURL(url);
}