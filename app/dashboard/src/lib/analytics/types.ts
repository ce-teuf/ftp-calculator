export interface NimResult {
  total_nim: number;
  asset_nim: number;
  liability_nim: number;
  treasury_contribution: number;
}

export interface RarocResult {
  line_id: string;
  capital_charge: number;
  expected_loss: number;
  risk_weight: number;
  raroc: number;
  hurdle: number;
  status: 'green' | 'orange' | 'red';
}

export interface ScenarioResult {
  scenario_name: string;
  nim_impact: number;
  ftp_rate_change: number;
  treasury_pnl_change: number;
}