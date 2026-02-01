use epimetheus_methylome::Motif;

use crate::{errors::DataError, io::reader::methylation::MethylationRecord};

#[derive(Debug, Clone)]
pub struct MotifSignature {
    pub motif: Motif,
    pub methylation_value: f64,
    pub n_motif_obs: u32,
    pub mean_coverage: f64,
}
impl TryFrom<MethylationRecord> for MotifSignature {
    type Error = DataError;

    fn try_from(value: MethylationRecord) -> Result<Self, Self::Error> {
        let motif = Motif::new(&value.motif, &value.mod_type, value.mod_position).map_err(|e| {
            let error_msg = format!(
                "Wrong motif mod: {}_{}_{}. Error: {}",
                value.motif,
                value.mod_type,
                value.mod_position,
                e.to_string()
            );
            DataError::DataAssertion(error_msg)
        })?;

        Ok(Self {
            motif,
            methylation_value: value.methylation_value,
            n_motif_obs: value.n_motif_obs,
            mean_coverage: value.mean_read_cov,
        })
    }
}
