pub struct RateCurve {
    pub component: String,
    pub tenors: Vec<String>,
    pub values: Vec<f64>,
}

impl RateCurve {
    pub fn new(component: String, tenors: Vec<String>, values: Vec<f64>) -> Self {
        Self {
            component,
            tenors,
            values,
        }
    }

    pub fn interpolate(&self, tenor: &str) -> Option<f64> {
        let idx = self.tenors.iter().position(|t| t == tenor)?;
        self.values.get(idx).copied()
    }
}

pub struct CurveBundle {
    pub curves: Vec<RateCurve>,
}

impl CurveBundle {
    pub fn new() -> Self {
        Self { curves: Vec::new() }
    }

    pub fn add_curve(&mut self, curve: RateCurve) {
        self.curves.push(curve);
    }

    pub fn get_rate(&self, component: &str, tenor: &str) -> Option<f64> {
        self.curves
            .iter()
            .find(|c| c.component == component)
            .and_then(|c| c.interpolate(tenor))
    }
}

impl Default for CurveBundle {
    fn default() -> Self {
        Self::new()
    }
}
