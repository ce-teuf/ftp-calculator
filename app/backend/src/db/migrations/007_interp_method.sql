ALTER TABLE curve_stack_components
  ADD COLUMN IF NOT EXISTS interp_method TEXT NOT NULL DEFAULT 'linear';
-- values: 'linear' | 'cubic' | 'flat_forward'
