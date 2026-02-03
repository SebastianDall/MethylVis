use std::{
    collections::{BTreeMap, HashMap, HashSet},
    fs::File,
    io::Write,
    path::{Path, PathBuf},
};

use epimetheus_methylome::Motif;
use toml;

use mag_core::{
    bin::{Bin, BinId, BinMetadataRecord},
    contig::{Contig, ContigAssignment, ContigId},
    io::reader::{checkm2::CheckM2Reader, contig_bin::ContigBinReader, methylation::MethReader},
    methylation::MotifSignature,
};

use crate::{
    errors::ApiError,
    models::{ContigMetadata, HeatmapData, MetadataUpdate, MethDataFilters, ProjectDetails},
};

pub struct Project {
    pub id: String,
    pub outdir: PathBuf,
    pub contig_metadata_path: PathBuf,
    pub motifs: HashSet<Motif>,
    pub bins: BTreeMap<BinId, Bin>,
    pub contig_methylation: HashMap<ContigId, Contig>,
}

impl Project {
    pub fn new(project_data: ProjectDetails) -> Result<Self, ApiError> {
        println!("{:#?}", project_data);
        let contig_bin = ContigBinReader::new(&project_data.contig_bin_path)?
            .read_all()
            .map_err(|e| {
                tracing::error!("Error reading contig_bin file: {}", e.to_string());
                e
            })?;

        let quality = if let Some(ref p) = project_data.bin_quality_path {
            let bin_qualities = CheckM2Reader::new(&p)?.read_all().map_err(|e| {
                tracing::error!("Error reading quality_file file: {}", e.to_string());
                e
            })?;
            bin_qualities
        } else {
            Vec::new()
        };

        let bins = Bin::from_records(contig_bin, quality);

        if bins.len() == 0 {
            tracing::error!("No bins were collected from provided files");
            return Err(ApiError::Io(
                "No bins were collected from provided files".to_string(),
            ));
        }

        let (contig_methylation, motifs) =
            Self::load_methylation(&project_data.methylation_data_path).map_err(|e| {
                tracing::error!("Error reading methylation file: {}", e.to_string());
                e
            })?;

        let mut metadata_path = project_data.output_path.clone();
        metadata_path.push("contig_metadata.tsv");

        // Create project toml
        let toml = toml::to_string(&project_data.clone())
            .map_err(|e| ApiError::ProjectFileCreation(e.to_string()))?;
        let mut toml_path = project_data.output_path.clone();
        toml_path.push("project.toml");

        std::fs::create_dir_all(&project_data.output_path)?;

        let mut file =
            File::create(toml_path).map_err(|e| ApiError::ProjectFileCreation(e.to_string()))?;
        let _ =
            write!(file, "{}", toml).map_err(|e| ApiError::ProjectFileCreation(e.to_string()))?;

        let _ = File::create(metadata_path.clone())
            .map_err(|e| ApiError::ProjectFileCreation(e.to_string()))?;

        let project = Self {
            id: project_data.project_id,
            outdir: project_data.output_path,
            contig_metadata_path: metadata_path,
            motifs,
            bins,
            contig_methylation,
        };

        project.save_metadata()?;

        Ok(project)
    }

    fn load_methylation(
        path: &Path,
    ) -> Result<(HashMap<ContigId, Contig>, HashSet<Motif>), ApiError> {
        let mut meth_rdr = MethReader::new(path)?;
        let mut motif_set = HashSet::new();

        let mut contig_meth: HashMap<ContigId, Contig> = HashMap::new();
        for rec in meth_rdr.records() {
            let res = rec?;

            let contig_id = ContigId(res.contig.clone());
            let motif_rec = MotifSignature::try_from(res)?;
            motif_set.insert(motif_rec.motif.clone());

            contig_meth
                .entry(contig_id.clone())
                .and_modify(|con| {
                    con.motifs
                        .insert(motif_rec.motif.clone(), motif_rec.clone());
                })
                .or_insert_with(|| Contig {
                    contig_id,
                    motifs: HashMap::from([(motif_rec.motif.clone(), motif_rec)]),
                    mean_coverage: 0.0,
                });
        }

        for contig in contig_meth.values_mut() {
            contig.mean_coverage = contig.derive_mean_coverage();
        }

        Ok((contig_meth, motif_set))
    }

    pub fn load_from_path(path: PathBuf) -> Result<Self, ApiError> {
        let toml_str =
            std::fs::read_to_string(path).map_err(|e| ApiError::ProjectNotFound(e.to_string()))?;
        let project_details: ProjectDetails =
            toml::from_str(&toml_str).map_err(|e| ApiError::ProjectFileCreation(e.to_string()))?;

        let mut saved_path = project_details.output_path.clone();
        saved_path.push("contig_metadata.csv");

        let (contig_methylation, motifs) =
            Self::load_methylation(&project_details.methylation_data_path).map_err(|e| {
                ApiError::Io(format!(
                    "Error loading contig methylation data: {}",
                    e.to_string()
                ))
            })?;

        let mut metadata_path = project_details.output_path.clone();
        metadata_path.push("contig_metadata.tsv");
        let mut rdr = csv::ReaderBuilder::new()
            .has_headers(true)
            .delimiter(b'\t')
            .from_path(&metadata_path)
            .map_err(|e| {
                ApiError::Io(format!(
                    "Could not create metadata reader from path {}. Error: {}",
                    metadata_path.display(),
                    e.to_string()
                ))
            })?;

        let mut bins: BTreeMap<BinId, Bin> = BTreeMap::new();
        for rec in rdr.deserialize() {
            let row: BinMetadataRecord =
                rec.map_err(|e| ApiError::Io(format!("Could not load bins: {}", e.to_string())))?;

            bins.entry(row.id.clone())
                .and_modify(|b| {
                    b.contig_metadata.push(ContigAssignment {
                        contig_id: row.contig_id.clone(),
                        assignment: row.assignment.clone(),
                    })
                })
                .or_insert_with(|| Bin {
                    id: row.id,
                    contig_metadata: vec![ContigAssignment {
                        contig_id: row.contig_id,
                        assignment: row.assignment,
                    }],
                    completeness: row.completeness,
                    contamination: row.contamination,
                    quality: row.quality,
                });
        }

        let project = Self {
            id: project_details.project_id,
            outdir: project_details.output_path,
            contig_metadata_path: metadata_path,
            motifs,
            bins,
            contig_methylation,
        };

        Ok(project)
    }

    pub fn update_metadata(&mut self, metadata: MetadataUpdate) -> Result<(), ApiError> {
        match self.bins.get_mut(&metadata.bin) {
            Some(b) => {
                if !b
                    .contig_metadata
                    .iter()
                    .all(|c| metadata.contigs.contains(c))
                    && b.contig_metadata.len() != metadata.contigs.len()
                {
                    return Err(ApiError::MetadataUpdate(
                        "Mismatch between contigs received and in bin. Change bin name."
                            .to_string(),
                    ));
                }

                b.contig_metadata = metadata.contigs;
            }
            None => {
                let new_bin = Bin {
                    id: metadata.bin.clone(),
                    contig_metadata: metadata.contigs,
                    completeness: None,
                    contamination: None,
                    quality: None,
                };
                self.bins.insert(metadata.bin.clone(), new_bin);
            }
        }
        tracing::info!("Updated metadata");
        Ok(())
    }

    pub fn save_metadata(&self) -> Result<(), ApiError> {
        tracing::info!("Saving metadata");
        // Load currently saved
        let saved_path = self.contig_metadata_path.clone();

        let mut wtr = csv::WriterBuilder::new()
            .has_headers(true)
            .delimiter(b'\t')
            .from_path(&saved_path)
            .map_err(|e| ApiError::Io(e.to_string()))?;

        for (_bin_id, bin) in &self.bins {
            let records = bin.to_metadata_records();

            for record in records {
                wtr.serialize(record)
                    .map_err(|e| ApiError::Io(e.to_string()))?;
            }
        }

        tracing::info!("metadata saved to: {}", saved_path.display());
        Ok(())
    }

    pub fn get_heatmap_data(&self, filters: MethDataFilters) -> Result<HeatmapData, ApiError> {
        let contigs_filter: Vec<&str> = match filters.selection {
            crate::models::ContigSelection::Bin(ref b) => self
                .bins
                .get(&BinId(b.to_string()))
                .ok_or_else(|| ApiError::Query(format!("Bin '{}' not found.", b)))?
                .contig_metadata
                .iter()
                .map(|c| c.contig_id.0.as_str())
                .collect(),
            crate::models::ContigSelection::Contigs(ref c) => {
                c.iter().map(|id| id.as_str()).collect()
            }
        };

        let contigs = self
            .contig_methylation
            .iter()
            .filter(|(id, _contig)| contigs_filter.contains(&id.0.as_str()))
            .map(|(_id, contig)| contig)
            .collect::<Vec<&Contig>>();

        // Iter through hashset will provide a random access each time. Therefore
        // we collect to vector first!
        let mut motif_vec: Vec<_> = self.motifs.iter().collect();

        let mut contig_meth_matrix = Vec::new();
        for contig in &contigs {
            let mut meth_values = Vec::new();
            for motif in &motif_vec {
                let motif_signature = contig.motifs.get(motif);
                let val = motif_signature.and_then(|m| {
                    if filters
                        .min_n_motif_obs
                        .map_or(false, |f| m.n_motif_obs < f as u32)
                        || filters.min_coverage.map_or(false, |f| m.mean_coverage < f)
                    {
                        None
                    } else {
                        Some(m.methylation_value)
                    }
                });
                meth_values.push(val);
            }
            contig_meth_matrix.push(meth_values);
        }

        if let Some(f) = filters.min_motif_variance {
            let mut retained_motif_idxs = Vec::new();
            for motif_idx in 0..motif_vec.len() {
                let values = contig_meth_matrix
                    .iter()
                    .filter_map(|row| row[motif_idx])
                    .collect::<Vec<f64>>();

                let n = values.len();
                let mean = values.iter().sum::<f64>() / n as f64;

                let sum_square = values.iter().map(|v| (v - mean).powf(2.0)).sum::<f64>();
                let var = sum_square / ((n - 1) as f64);

                if var < f {
                    continue;
                }

                retained_motif_idxs.push(motif_idx);
            }
            let filtered_motifs: Vec<_> = retained_motif_idxs
                .iter()
                .map(|&idx| motif_vec[idx])
                .collect();

            contig_meth_matrix = contig_meth_matrix
                .into_iter()
                .map(|row| retained_motif_idxs.iter().map(|&idx| row[idx]).collect())
                .collect();

            motif_vec = filtered_motifs;
        }

        let contig_ids = contigs.iter().map(|c| c.contig_id.0.clone()).collect();

        let metadata = match filters.selection {
            crate::models::ContigSelection::Bin(b) => {
                let bin = self.bins.get(&BinId(b)).unwrap();
                let contig_metadata: HashMap<String, ContigMetadata> = bin
                    .contig_metadata
                    .iter()
                    .map(|c| {
                        let cm = ContigMetadata {
                            contig_id: c.contig_id.0.clone(),
                            assignment: c.assignment,
                            mean_coverage: self
                                .contig_methylation
                                .get(&c.contig_id)
                                .map(|c| c.mean_coverage)
                                .unwrap_or(0.0),
                            note: Some(String::new()),
                        };
                        (c.contig_id.0.clone(), cm)
                    })
                    .collect();

                Some(contig_metadata)
            }
            crate::models::ContigSelection::Contigs(c) => {
                let contig_metadata = c
                    .into_iter()
                    .map(|c| {
                        let contig_id = ContigId(c.clone());
                        let cm = ContigMetadata {
                            contig_id: contig_id.0.clone(),
                            assignment: mag_core::contig::Assignment::None,
                            mean_coverage: self
                                .contig_methylation
                                .get(&contig_id)
                                .map(|c| c.mean_coverage)
                                .unwrap_or(1.0),
                            note: Some(String::new()),
                        };
                        (c, cm)
                    })
                    .collect::<HashMap<String, ContigMetadata>>();

                Some(contig_metadata)
            }
        };

        let hm = HeatmapData {
            contigs: contig_ids,
            motifs: motif_vec
                .iter()
                .map(|m| {
                    format!(
                        "{}_{}_{}",
                        m.sequence_to_string(),
                        m.mod_type.to_pileup_code(),
                        m.mod_position
                    )
                })
                .collect(),
            matrix: contig_meth_matrix,
            metadata,
        };

        Ok(hm)
    }
}
