export interface FtpMethod {
  name: string;
  key: string;
  description: string;
}

export const METHODS: FtpMethod[] = [
  { name: 'Weighted-Average (Stock)', key: 'stock', description: 'Méthode basée sur le stock moyen' },
  { name: 'Flux (Multi-Vintage)', key: 'flux', description: 'Méthode flux avec cohortes' },
  { name: 'Duration Method', key: 'duration', description: 'Méthode par duration' },
  { name: 'Pool Method', key: 'pool', description: 'Taux unique pool' },
  { name: 'Multiple Pool', key: 'multiple_pool', description: 'Pools par maturité' },
  { name: 'Refinancing / Forward Rate', key: 'refinancing', description: 'Taux forward' },
  { name: 'Floating-Rate', key: 'floating', description: 'Double profil taux + liquidité' },
  { name: 'Replicating Portfolio', key: 'replicating', description: 'Portefeuille répliquant' },
  { name: 'Behavioral Run-off', key: 'behavioral', description: 'Modèle comportemental' },
];