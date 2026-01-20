use csv::Writer;
use mag_core::manager::DataManager;
use polars::prelude::*;
use serde::{Deserialize, Serialize};
use std::io::{BufReader, Write};
use std::{collections::HashMap, fs::File, path::PathBuf};
use toml;
use ts_rs::TS;

use crate::errors::ApiError;

pub struct AppState {
    pub projects: HashMap<String, Project>,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            projects: HashMap::new(),
        }
    }

    pub fn add_project(&mut self, payload: CreateProjectPayload) -> Result<(), ApiError> {
        use std::collections::hash_map::Entry;

        match self.projects.entry(payload.project_id.clone()) {
            Entry::Occupied(_) => Err(ApiError::ProjectExists(payload.project_id)),
            Entry::Vacant(entry) => {
                let new_project = Project::new(payload)?;

                entry.insert(new_project);
                Ok(())
            }
        }
    }

    pub fn load_project(&mut self, path: PathBuf) -> Result<(), ApiError> {
        use std::collections::hash_map::Entry;

        let new_project = Project::load_from_path(path)?;

        match self.projects.entry(new_project.id.clone()) {
            Entry::Occupied(_) => Err(ApiError::ProjectExists(new_project.id.clone())),
            Entry::Vacant(entry) => {
                tracing::info!("Loading project: {}", new_project.id.clone());
                entry.insert(new_project);
                Ok(())
            }
        }
    }

    pub fn get_mut_project(&mut self, project: &str) -> Result<&mut Project, ApiError> {
        match self.projects.get_mut(project) {
            Some(p) => Ok(p),
            None => Err(ApiError::ProjectNotFound(project.to_string())),
        }
    }

    pub fn get_project(&self, project: &str) -> Result<&Project, ApiError> {
        match self.projects.get(project) {
            Some(p) => Ok(p),
            None => Err(ApiError::ProjectNotFound(project.to_string())),
        }
    }
    pub fn get_all_project_ids(&self) -> Vec<String> {
        self.projects
            .keys()
            .into_iter()
            .map(|c| c.to_string())
            .collect::<Vec<String>>()
    }

    /// DATA
    ///

    pub fn get_heatmap_data(
        &self,
        project_id: &String,
        filters: MethDataFilters,
    ) -> Result<HeatmapData, ApiError> {
        let project = self.get_project(project_id)?;

        let meth_data = project.data.filter_methylation_data(
            &filters.contigs,
            filters.min_n_motif_obs,
            filters.min_coverage,
            filters.min_motif_variance,
        )?;

        // Step 1: Extract unique, sorted axes
        let contigs: Vec<String> = meth_data
            .column("contig")?
            .unique()?
            .sort(SortOptions::default())?
            .str()?
            .into_iter()
            .filter_map(|s| s.map(String::from))
            .collect();

        let motifs: Vec<String> = meth_data
            .column("motif_mod")?
            .unique()?
            .sort(SortOptions::default())?
            .str()?
            .into_iter()
            .filter_map(|s| s.map(String::from))
            .collect();

        // Step 2: Build index maps for O(1) lookup
        let contig_idx: HashMap<String, usize> = contigs
            .iter()
            .enumerate()
            .map(|(i, c)| (c.clone(), i))
            .collect();

        let motif_idx: HashMap<String, usize> = motifs
            .iter()
            .enumerate()
            .map(|(i, m)| (m.clone(), i))
            .collect();

        // Step 3: Initialize matrix with None (sparse data)
        let mut matrix: Vec<Vec<Option<f64>>> = vec![vec![None; motifs.len()]; contigs.len()];

        // Step 4: Populate matrix with observed values
        let contig_col = meth_data.column("contig")?.str()?;
        let motif_col = meth_data.column("motif_mod")?.str()?;
        let value_col = meth_data.column("methylation_value")?.f64()?;

        for i in 0..meth_data.height() {
            if let (Some(contig), Some(motif), Some(value)) =
                (contig_col.get(i), motif_col.get(i), value_col.get(i))
            {
                if let (Some(&row), Some(&col)) = (contig_idx.get(contig), motif_idx.get(motif)) {
                    matrix[row][col] = Some(value);
                }
            }
        }
        // Step 5: Build metadata for each contig
        let mut metadata = HashMap::new();

        for contig in &contigs {
            // Filter DataFrame for this specific contig
            let contig_data = meth_data
                .clone()
                .lazy()
                .filter(col("contig").eq(lit(contig.clone())))
                .collect()?;

            // Extract bin (take first value, should be same for all rows)
            let bin = contig_data
                .column("bin")?
                .str()?
                .get(0)
                .unwrap_or("unbinned")
                .to_string();

            // Calculate mean coverage for this contig
            let mean_coverage = contig_data
                .column("mean_read_cov")?
                .f64()?
                .mean()
                .unwrap_or(0.0);

            // Get total motif observations for this contig
            let n_motif_obs = contig_data.column("n_motif_obs")?.u32()?.sum().unwrap_or(0) as i32;

            let current_metadata = &project.contig_metadata;
            let assignment = if let Some(contig_metadata) = current_metadata.get(contig) {
                contig_metadata.assignment.clone()
            } else {
                ContigAssignment::None
            };
            let note = None;

            metadata.insert(
                contig.clone(),
                ContigMetadata {
                    contig_id: contig.clone(),
                    bin,
                    assignment,
                    mean_coverage,
                    n_motif_obs,
                    note,
                },
            );
        }

        Ok(HeatmapData {
            contigs,
            motifs,
            matrix,
            metadata,
        })
    }
}

pub struct Project {
    id: String,
    outdir: PathBuf,
    pub data: DataManager,
    pub contig_metadata: HashMap<String, ContigMetadata>,
    contig_metadata_path: PathBuf,
}

impl Project {
    fn from_payload(
        payload: CreateProjectPayload,
        contig_metadata: HashMap<String, ContigMetadata>,
        contig_metadata_path: PathBuf,
    ) -> Result<Self, ApiError> {
        tracing::info!("Creating DataManager");
        tracing::info!(
            "  methylation_data_path: {:?}",
            payload.methylation_data_path
        );
        tracing::info!("  contig_bin_path: {:?}", payload.contig_bin_path);
        tracing::info!("  methylation_data_path: {:?}", payload.bin_quality_path);

        let data = DataManager::new(
            &payload.methylation_data_path,
            &payload.contig_bin_path,
            payload.bin_quality_path,
        )?;
        tracing::info!("DataManager created:");
        tracing::info!(
            "  methylation_data rows: {}",
            data.methylation_data.height()
        );
        tracing::info!("  contig_bin rows: {}", data.contig_bin.height());
        tracing::info!(
            "  contig_bin columns: {:?}",
            data.contig_bin.get_column_names()
        );

        Ok(Self {
            id: payload.project_id,
            outdir: payload.output_path,
            data,
            contig_metadata,
            contig_metadata_path,
        })
    }
    pub fn new(payload: CreateProjectPayload) -> Result<Self, ApiError> {
        if payload.output_path.exists() {
            return Err(ApiError::ProjectExists(format!(
                "Project: {} already exists at: {:?}",
                payload.project_id, payload.output_path
            )));
        }

        // Create metadata path
        let mut metadata_path = payload.output_path.clone();
        metadata_path.push("contig_metadata.csv");
        // Create project toml
        let toml =
            toml::to_string(&payload).map_err(|e| ApiError::ProjectFileCreation(e.to_string()))?;
        let mut toml_path = payload.output_path.clone();
        toml_path.push("project.toml");

        let project = Self::from_payload(payload, HashMap::new(), metadata_path.clone())?;

        std::fs::create_dir_all(&project.outdir)?;

        let mut file =
            File::create(toml_path).map_err(|e| ApiError::ProjectFileCreation(e.to_string()))?;
        let _ =
            write!(file, "{}", toml).map_err(|e| ApiError::ProjectFileCreation(e.to_string()))?;

        let _ = File::create(metadata_path.clone())
            .map_err(|e| ApiError::ProjectFileCreation(e.to_string()))?;

        Ok(project)
    }

    pub fn load_from_path(path: PathBuf) -> Result<Self, ApiError> {
        let toml_str =
            std::fs::read_to_string(path).map_err(|e| ApiError::ProjectNotFound(e.to_string()))?;
        let project: CreateProjectPayload =
            toml::from_str(&toml_str).map_err(|e| ApiError::ProjectFileCreation(e.to_string()))?;

        let mut saved_path = project.output_path.clone();
        saved_path.push("contig_metadata.csv");

        let buf = BufReader::new(File::open(saved_path.clone()).map_err(|e| {
            let error = format!("Could not load metadata file: {}", e.to_string());
            tracing::error!(error);
            ApiError::MetadataUpdate(error)
        })?);

        let mut saved_data = HashMap::new();
        let mut rdr = csv::Reader::from_reader(buf);
        for result in rdr.deserialize() {
            let record: ContigMetadata = result.map_err(|e| {
                let error = format!(
                    "Error reading record in {}: {}",
                    saved_path.display(),
                    e.to_string()
                );
                tracing::error!(error);
                ApiError::Io(error)
            })?;
            saved_data.insert(record.contig_id.clone(), record);
        }

        let loaded_project = Self::from_payload(project, saved_data, saved_path)?;

        Ok(loaded_project)
    }

    pub fn update_metadata(&mut self, metadata: Vec<ContigMetadata>) {
        for c in metadata {
            self.contig_metadata.insert(c.contig_id.clone(), c);
        }
        tracing::info!("Updated metadata");
    }

    pub fn save_metadata(&self) -> Result<(), ApiError> {
        // Load currently saved
        let saved_path = self.contig_metadata_path.clone();

        let mut wtr = Writer::from_path(saved_path.clone()).map_err(|e| {
            let error = format!(
                "Could not open path: {} due to: {}",
                saved_path.display(),
                e.to_string()
            );
            tracing::error!(error);
            ApiError::Io(error)
        })?;

        for (_id, metadata) in &self.contig_metadata {
            wtr.serialize(&metadata).map_err(|e| {
                let error = format!("Could not write record due to: {}", e.to_string());
                tracing::error!(error);
                ApiError::Io(error)
            })?;
        }

        tracing::info!("Updated contig metadata");
        Ok(())
    }
}

#[derive(Serialize, Deserialize, Debug, TS)]
#[ts(export, export_to = "../../../contam-map-frontend/src/bindings/")]
pub struct CreateProjectPayload {
    pub project_id: String,
    pub methylation_data_path: PathBuf,
    pub contig_bin_path: PathBuf,
    pub bin_quality_path: Option<PathBuf>,
    pub output_path: PathBuf,
}

#[derive(Deserialize, Debug, TS)]
#[ts(export, export_to = "../../../contam-map-frontend/src/bindings/")]
pub struct ProjectFilter {
    pub project_id: String,
}

#[derive(Deserialize, Debug, TS)]
#[ts(export, export_to = "../../../contam-map-frontend/src/bindings/")]
pub struct MethDataFilters {
    pub contigs: Vec<String>,
    pub min_n_motif_obs: Option<i32>,
    pub min_motif_variance: Option<f64>,
    pub min_coverage: Option<f64>,
}

#[derive(Serialize, Debug, TS)]
#[ts(export, export_to = "../../../contam-map-frontend/src/bindings/")]
pub struct HeatmapData {
    pub contigs: Vec<String>,
    pub motifs: Vec<String>,
    pub matrix: Vec<Vec<Option<f64>>>,
    pub metadata: HashMap<String, ContigMetadata>,
}

#[derive(Serialize, Deserialize, Debug, TS, Clone)]
#[ts(export, export_to = "../../../contam-map-frontend/src/bindings/")]
pub struct ContigMetadata {
    pub contig_id: String,
    pub bin: String,
    pub assignment: ContigAssignment,
    pub mean_coverage: f64,
    pub n_motif_obs: i32,
    pub note: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone, TS, Default)]
#[ts(export, export_to = "../../../contam-map-frontend/src/bindings/")]
pub enum ContigAssignment {
    #[default]
    None,
    Clean,
    Contamination,
    Ambiguous,
}

#[derive(Serialize, Deserialize, Debug, Clone, TS)]
#[ts(export, export_to = "../../../contam-map-frontend/src/bindings/")]
pub enum BinQuality {
    HQ,
    MQ,
    LQ,
}

impl ToString for BinQuality {
    fn to_string(&self) -> String {
        match self {
            BinQuality::HQ => "HQ".to_string(),
            BinQuality::MQ => "MQ".to_string(),
            BinQuality::LQ => "LQ".to_string(),
        }
    }
}

#[derive(Deserialize)]
pub struct BinQueryParams {
    #[serde(default)]
    pub quality_filter: Vec<BinQuality>,
}
