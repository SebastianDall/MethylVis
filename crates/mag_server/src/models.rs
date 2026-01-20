use mag_core::manager::DataManager;
use serde::{Deserialize, Serialize};
use std::io::Write;
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
                if payload.output_path.exists() {
                    return Err(ApiError::ProjectExists(format!(
                        "Project already exists at: {:?}",
                        payload.output_path
                    )));
                }

                std::fs::create_dir_all(&payload.output_path)?;
                let new_project = Project::new(
                    payload.project_id.clone(),
                    payload.output_path.clone(),
                    payload.methylation_data_path.clone(),
                    payload.contig_bin_path.clone(),
                )?;

                // save project
                let toml = toml::to_string(&payload)
                    .map_err(|e| ApiError::ProjectTomlError(e.to_string()))?;
                let mut path = payload.output_path.clone();
                path.push("project.toml");
                let mut file =
                    File::create(path).map_err(|e| ApiError::ProjectTomlError(e.to_string()))?;
                let _ = write!(file, "{}", toml)
                    .map_err(|e| ApiError::ProjectTomlError(e.to_string()))?;

                entry.insert(new_project);
                Ok(())
            }
        }
    }

    pub fn load_project(&mut self, payload: CreateProjectPayload) -> Result<(), ApiError> {
        use std::collections::hash_map::Entry;

        match self.projects.entry(payload.project_id.clone()) {
            Entry::Occupied(_) => Err(ApiError::ProjectExists(payload.project_id)),
            Entry::Vacant(entry) => {
                let new_project = Project::new(
                    payload.project_id.clone(),
                    payload.output_path.clone(),
                    payload.methylation_data_path.clone(),
                    payload.contig_bin_path.clone(),
                )?;
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

pub struct Project {
    id: String,
    output: PathBuf,
    pub data: DataManager,
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

        Ok(Self { id, output, data })
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
