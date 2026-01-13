use polars::prelude::*;
use std::path::Path;

use crate::{
    errors::DataError,
    parsers::{parse_contig_bin, parse_methylation_data},
};

pub struct DataManager {
    pub methylation_data: DataFrame,
    pub contig_bin: DataFrame,
}

impl DataManager {
    pub fn new(methylation_data: &Path, contig_bin_path: &Path) -> Result<Self, DataError> {
        let methylation_data = parse_methylation_data(methylation_data)?;
        let contig_bin = parse_contig_bin(contig_bin_path)?;

        // CREATE ASSERTIUON!
        let contigs = contig_bin
            .column("contig")?
            .as_materialized_series()
            .clone();
        let data_shape = methylation_data
            .clone()
            .lazy()
            .filter(col("contig").is_in(lit(contigs), true))
            .collect()?
            .shape();

        if data_shape.0 == 0 {
            return Err(DataError::DataAssertion(
                "There are no contig match between contig_bin and methylation_data".to_string(),
            ));
        }

        Ok(Self {
            methylation_data,
            contig_bin,
        })
    }

    // Operations

    pub fn get_contigs(&self) -> Result<Vec<String>, DataError> {
        let unique_series = self.methylation_data.column("contig")?.unique()?;

        let unique_vec = unique_series
            .str()?
            .into_iter()
            .filter_map(|v| v.map(String::from))
            .collect::<Vec<String>>();
        Ok(unique_vec)
    }

    pub fn get_bins(&self) -> Result<Vec<String>, DataError> {
        let unique_series = self.contig_bin.column("bin".into())?.unique()?;
        println!("{:#?}", unique_series);

        let unique_vec = unique_series
            .str()?
            .into_iter()
            .filter_map(|v| v.map(String::from))
            .collect::<Vec<String>>();
        Ok(unique_vec)
    }

    pub fn filter_bin(&self, bin_name: &str) -> Result<DataFrame, DataError> {
        let bin_contigs = self
            .contig_bin
            .clone()
            .lazy()
            .filter(col("bin").eq(lit(bin_name)));

        let filtered_meth = bin_contigs
            .join(
                self.methylation_data.clone().lazy(),
                [col("contig")],
                [col("contig")],
                JoinArgs::new(JoinType::Left),
            )
            .collect()?;

        Ok(filtered_meth)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn create_test_data() -> (TempDir, std::path::PathBuf, std::path::PathBuf) {
        let temp_dir = TempDir::new().unwrap();

        let meth_path = temp_dir.path().join("methylation.tsv");
        let meth_content = "contig\tposition\tmethylation\ncontig1\t100\t0.8\ncontig1\t200\t0.9\ncontig2\t150\t0.7\ncontig3\t300\t0.6";
        fs::write(&meth_path, meth_content).unwrap();

        let bin_path = temp_dir.path().join("contig_bin.tsv");
        let bin_content = "contig\tbin\ncontig1\tbin1\ncontig2\tbin1\ncontig3\tbin2";
        fs::write(&bin_path, bin_content).unwrap();

        (temp_dir, meth_path, bin_path)
    }

    #[test]
    fn test_new_data_manager() {
        let (_temp_dir, meth_path, bin_path) = create_test_data();

        let manager = DataManager::new(&meth_path, &bin_path);
        assert!(manager.is_ok());

        let manager = manager.unwrap();
        assert_eq!(manager.methylation_data.height(), 4);
        assert_eq!(manager.contig_bin.height(), 3);
    }

    #[test]
    fn test_get_contigs() {
        let (_temp_dir, meth_path, bin_path) = create_test_data();
        let manager = DataManager::new(&meth_path, &bin_path).unwrap();

        let contigs = manager.get_contigs().unwrap();
        assert_eq!(contigs.len(), 3);
        assert!(contigs.contains(&"contig1".to_string()));
        assert!(contigs.contains(&"contig2".to_string()));
        assert!(contigs.contains(&"contig3".to_string()));
    }

    #[test]
    fn test_get_bins() {
        let (_temp_dir, meth_path, bin_path) = create_test_data();
        let manager = DataManager::new(&meth_path, &bin_path).unwrap();

        let bins = manager.get_bins().unwrap();
        assert_eq!(bins.len(), 2);
        assert!(bins.contains(&"bin1".to_string()));
        assert!(bins.contains(&"bin2".to_string()));
    }

    #[test]
    fn test_filter_bin() {
        let (_temp_dir, meth_path, bin_path) = create_test_data();
        let manager = DataManager::new(&meth_path, &bin_path).unwrap();

        let filtered = manager.filter_bin("bin1").unwrap();
        // bin1 contains contig1 and contig2, which have 3 methylation records total
        assert_eq!(filtered.height(), 3);

        let filtered = manager.filter_bin("bin2").unwrap();
        // bin2 contains contig3, which has 1 methylation record
        assert_eq!(filtered.height(), 1);
    }

    #[test]
    fn test_filter_nonexistent_bin() {
        let (_temp_dir, meth_path, bin_path) = create_test_data();
        let manager = DataManager::new(&meth_path, &bin_path).unwrap();

        let filtered = manager.filter_bin("nonexistent").unwrap();
        assert_eq!(filtered.height(), 0);
    }
}
