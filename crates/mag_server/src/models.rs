use crate::errors::ApiError;
use crate::project::Project;
use mag_core::{
    bin::{BinId, BinQuality},
    contig::{Assignment, ContigAssignment},
};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, path::PathBuf};
use ts_rs::TS;

pub struct AppState {
    pub projects: HashMap<String, Project>,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            projects: HashMap::new(),
        }
    }

    pub fn add_project(&mut self, payload: ProjectDetails) -> Result<(), ApiError> {
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
        tracing::info!("Loading project: {}", new_project.id.clone());

        match self.projects.entry(new_project.id.clone()) {
            Entry::Occupied(_) => Err(ApiError::ProjectExists(new_project.id.clone())),
            Entry::Vacant(entry) => {
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
}

#[derive(Serialize, Deserialize, Debug, TS, Clone)]
#[ts(export, export_to = "../../../contam-map-frontend/src/bindings/")]
pub struct ProjectDetails {
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
pub enum ContigSelection {
    Bin(String),
    Contigs(Vec<String>),
}

#[derive(Deserialize, Debug, TS)]
#[ts(export, export_to = "../../../contam-map-frontend/src/bindings/")]
pub struct MethDataFilters {
    pub selection: ContigSelection,
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
    pub metadata: Option<HashMap<String, ContigMetadata>>,
}

#[derive(Serialize, Deserialize, Debug, TS, Clone)]
#[ts(export, export_to = "../../../contam-map-frontend/src/bindings/")]
pub struct ContigMetadata {
    pub contig_id: String,
    pub assignment: Assignment,
    pub mean_coverage: f64,
    pub note: Option<String>,
}

#[derive(Deserialize)]
pub struct BinQueryParams {
    #[serde(default)]
    pub quality_filter: Vec<BinQuality>,
}

#[derive(Serialize, Deserialize, Debug, TS, Clone)]
#[ts(export, export_to = "../../../contam-map-frontend/src/bindings/")]
pub struct MetadataUpdate {
    pub bin: BinId,
    pub contigs: Vec<ContigAssignment>,
}
