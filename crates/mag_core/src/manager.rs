use polars::prelude::*;
use std::path::{Path, PathBuf};

use crate::{
    errors::DataError,
    parsers::{parse_bin_quality, parse_contig_bin, parse_methylation_data},
};

pub struct DataManager {
    pub methylation_data: DataFrame,
    pub contig_bin: DataFrame,
    pub bin_quality: Option<DataFrame>,
    pub contigs: Vec<String>,
}

impl DataManager {
    pub fn new(
        methylation_data: &Path,
        contig_bin_path: &Path,
        bin_quality_path: Option<PathBuf>,
    ) -> Result<Self, DataError> {
        let methylation_data = parse_methylation_data(methylation_data)?;
        let contig_bin = parse_contig_bin(contig_bin_path)?;

        // CREATE ASSERTIONS!
        let contigs = contig_bin
            .column("contig")?
            .as_materialized_series()
            .clone();
        let data_shape = methylation_data
            .clone()
            .lazy()
            .filter(col("contig").is_in(lit(contigs).implode(), true))
            .collect()?
            .shape();

        if data_shape.0 == 0 {
            return Err(DataError::DataAssertion(
                "There are no contig match between contig_bin and methylation_data".to_string(),
            ));
        }

        let methylation_data = methylation_data
            .clone()
            .lazy()
            .with_column(
                concat_str(
                    [
                        col("motif"),
                        lit("_"),
                        col("mod_type"),
                        lit("_"),
                        col("mod_position").cast(DataType::String),
                    ],
                    "",
                    true,
                )
                .alias("motif_mod"),
            )
            .collect()?;

        let unique_series = methylation_data.column("contig")?.unique()?;

        let contigs = unique_series
            .str()?
            .into_iter()
            .filter_map(|v| v.map(String::from))
            .collect::<Vec<String>>();

        let bin_quality = if let Some(p) = bin_quality_path {
            let parsed_qualities = parse_bin_quality(&p)?.lazy();
            let bin_qualities = parsed_qualities
                .with_column(
                    when(
                        col("completeness")
                            .gt(lit(90.0))
                            .and(col("contamination").lt(lit(5.0))),
                    )
                    .then(lit("HQ"))
                    .otherwise(
                        when(
                            col("completeness")
                                .gt_eq(lit(50.0))
                                .and(col("contamination").lt(lit(10.0))),
                        )
                        .then(lit("MQ"))
                        .otherwise(lit("LQ")),
                    )
                    .alias("quality"),
                )
                .collect()?;

            let bins_in_qualities = bin_qualities
                .column("bin")?
                .str()?
                .into_iter()
                .filter_map(|b| b.map(String::from))
                .collect::<Vec<String>>();

            let bins_in_contig_bin = contig_bin
                .column("bin")?
                .unique()?
                .str()?
                .into_iter()
                .filter_map(|b| b.map(String::from))
                .collect::<Vec<String>>();

            let bin_overlap = bins_in_contig_bin
                .iter()
                .filter(|b| bins_in_qualities.contains(b))
                .count();

            if bin_overlap == 0 {
                return Err(DataError::DataAssertion(
                    "Contig Bin and Bin qualities have 0 overlap".to_string(),
                ));
            }

            Some(bin_qualities)
        } else {
            None
        };

        Ok(Self {
            methylation_data,
            contig_bin,
            bin_quality,
            contigs,
        })
    }

    // Operations

    pub fn get_contigs(&self) -> Vec<String> {
        self.contigs.clone()
    }

    pub fn get_bins(&self, qualities: Option<Vec<String>>) -> Result<Vec<String>, DataError> {
        let mut bins_df = self.contig_bin.clone().lazy();

        if let Some(q) = qualities {
            let bin_qualities = self
                .bin_quality
                .clone()
                .ok_or(DataError::BinQuality("No bin qualities found".to_string()))?
                .lazy()
                .clone();

            let quality_bins = bin_qualities
                .filter(
                    col("quality").is_in(lit(Series::new("quality".into(), q)).implode(), false),
                )
                .collect()?
                .column("bin")?
                .as_materialized_series()
                .clone();

            bins_df = bins_df.filter(col("bin").is_in(lit(quality_bins).implode(), false))
        }

        let bins_df = bins_df.collect()?;
        println!("{:#?}", bins_df.head(Some(10)));
        let unique_series = bins_df.column("bin".into())?.unique()?;
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

    pub fn filter_methylation_data(
        &self,
        contigs: &Vec<String>,
        min_n_motif_obs: Option<i32>,
        min_coverage: Option<f64>,
        min_variance: Option<f64>,
    ) -> Result<DataFrame, DataError> {
        let mut meth_lazy = self.methylation_data.clone().lazy();
        let contigs = Series::new("contig".into(), contigs);

        meth_lazy = meth_lazy.filter(col("contig").is_in(lit(contigs.clone()).implode(), true));

        if let Some(min_obs) = min_n_motif_obs {
            meth_lazy = meth_lazy.filter(col("n_motif_obs").gt_eq(lit(min_obs)))
        }
        if let Some(min_cov) = min_coverage {
            meth_lazy = meth_lazy.filter(col("mean_read_cov").gt_eq(lit(min_cov)))
        }
        if let Some(min_var) = min_variance {
            let motif_var = meth_lazy
                .clone()
                .group_by([col("motif_mod")])
                .agg([col("methylation_value").var(1).alias("variance")])
                .collect()?;

            let high_var_motifs = motif_var
                .lazy()
                .filter(col("variance").gt_eq(lit(min_var)))
                .collect()?
                .column("motif_mod")?
                .unique()?
                .as_materialized_series()
                .clone();

            meth_lazy =
                meth_lazy.filter(col("motif_mod").is_in(lit(high_var_motifs).implode(), true))
        }

        let contig_bin = self
            .contig_bin
            .clone()
            .lazy()
            .filter(col("contig").is_in(lit(contigs).implode(), true))
            .collect()?
            .lazy();

        meth_lazy = meth_lazy
            .left_join(contig_bin, col("contig"), col("contig"))
            .with_column(col("bin").fill_null(lit("unbinned").alias("bin")))
            .sort(["bin"], Default::default());

        let meth = meth_lazy.collect()?;

        Ok(meth)
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

        let manager = DataManager::new(&meth_path, &bin_path, None);
        assert!(manager.is_ok());

        let manager = manager.unwrap();
        assert_eq!(manager.methylation_data.height(), 4);
        assert_eq!(manager.contig_bin.height(), 3);
    }

    #[test]
    fn test_get_contigs() {
        let (_temp_dir, meth_path, bin_path) = create_test_data();
        let manager = DataManager::new(&meth_path, &bin_path, None).unwrap();

        let contigs = manager.get_contigs();
        assert_eq!(contigs.len(), 3);
        assert!(contigs.contains(&"contig1".to_string()));
        assert!(contigs.contains(&"contig2".to_string()));
        assert!(contigs.contains(&"contig3".to_string()));
    }

    #[test]
    fn test_get_bins() {
        let (_temp_dir, meth_path, bin_path) = create_test_data();
        let manager = DataManager::new(&meth_path, &bin_path, None).unwrap();

        let bins = manager.get_bins(None).unwrap();
        assert_eq!(bins.len(), 2);
        assert!(bins.contains(&"bin1".to_string()));
        assert!(bins.contains(&"bin2".to_string()));
    }

    #[test]
    fn test_filter_bin() {
        let (_temp_dir, meth_path, bin_path) = create_test_data();
        let manager = DataManager::new(&meth_path, &bin_path, None).unwrap();

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
        let manager = DataManager::new(&meth_path, &bin_path, None).unwrap();

        let filtered = manager.filter_bin("nonexistent").unwrap();
        assert_eq!(filtered.height(), 0);
    }
}
