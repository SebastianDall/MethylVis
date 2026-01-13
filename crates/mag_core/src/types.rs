use epimetheus_methylome::Motif;

pub struct Bin {
    pub id: String,
    pub contigs: Vec<Contig>,
}

pub struct Contig {
    pub id: String,
    pub methylation: Vec<MethylationData>,
    pub is_contamination: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub struct MethylationData {
    motif: Motif,
    methylation_value: u8,
    mean_read_cov: f64,
    n_motif_obs: u64,
    motif_occurences_total: u64,
}

#[derive(Clone, Debug, PartialEq)]
pub struct MethylationDataRow {
    contig: String,
    motif: String,
    mod_type: String,
    mod_position: u8,
    methylation_value: f64,
    mean_read_cov: f64,
    n_motif_obs: u32,
    motif_occurences_total: u32,
}
