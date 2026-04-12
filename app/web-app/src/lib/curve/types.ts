export interface RateCurve {
  id: string;
  name: string;
  component: string;
  currency: string;
  version: number;
  status: 'draft' | 'approved' | 'archived';
  valid_from?: string;
  valid_to?: string;
  tenors: string[];
  values: number[];
  source?: string;
  notes?: string;
  created_at: string;
  created_by?: string;
}

export interface CurveComponent {
  base_rate: string;
  tlp: string;
  credit_spread: string;
  clp: string;
  oas: string;
  capital_charge: string;
  basis_risk: string;
  operational_risk: string;
}