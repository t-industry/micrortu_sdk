use crate::{QualityDescriptor, M_DP_NA_1, M_SP_NA_1};

impl M_DP_NA_1 {
    #[must_use]
    pub fn get_qds(&self) -> &dyn QualityDescriptor {
        &self.value
    }
    #[must_use]
    pub fn get_qds_mut(&mut self) -> &mut dyn QualityDescriptor {
        &mut self.value
    }
}

impl M_SP_NA_1 {
    #[must_use]
    pub fn get_qds(&self) -> &dyn QualityDescriptor {
        &self.value
    }
    #[must_use]
    pub fn get_qds_mut(&mut self) -> &mut dyn QualityDescriptor {
        &mut self.value
    }
}
