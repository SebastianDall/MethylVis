use std::collections::HashMap;

use epimetheus_methylome::Motif;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::methylation::MotifSignature;

#[derive(Clone, Debug, Hash, PartialEq, Eq, Deserialize, Serialize, TS)]
#[ts(export, export_to = "../../../contam-map-frontend/src/bindings/")]
pub struct ContigId(pub String);
#[derive(Clone)]
pub struct Contig {
    pub contig_id: ContigId,
    pub motifs: HashMap<Motif, MotifSignature>,
    pub mean_coverage: f64,
    // length could be nice
}

impl Contig {
    pub fn derive_mean_coverage(&self) -> f64 {
        let total_cov = self.motifs.values().map(|m| m.mean_coverage).sum::<f64>();
        total_cov / self.motifs.values().len() as f64
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, TS, Default, PartialEq, Eq, Copy)]
#[ts(export, export_to = "../../../contam-map-frontend/src/bindings/")]
pub enum Assignment {
    #[default]
    None,
    Clean,
    Contamination,
    Ambiguous,
}

#[derive(Serialize, Deserialize, Debug, Clone, TS, PartialEq, Eq)]
#[ts(export, export_to = "../../../contam-map-frontend/src/bindings/")]
pub struct ContigAssignment {
    pub contig_id: ContigId,
    pub assignment: Assignment,
}

impl ContigAssignment {
    pub fn new(contig_id: ContigId, assignment: Assignment) -> Self {
        Self {
            contig_id,
            assignment,
        }
    }
}
