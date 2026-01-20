use std::path::Path;

use polars::prelude::*;

use crate::errors::DataError;

pub fn parse_methylation_data(path: &Path) -> Result<DataFrame, DataError> {
    let schema = Schema::from_iter(vec![
        Field::new("contig".into(), DataType::String),
        Field::new("motif".into(), DataType::String),
        Field::new("mod_type".into(), DataType::String),
        Field::new("mod_position".into(), DataType::UInt8),
        Field::new("methylation_value".into(), DataType::Float64),
        Field::new("mean_read_cov".into(), DataType::Float64),
        Field::new("n_motif_obs".into(), DataType::UInt32),
        Field::new("motif_occurences_total".into(), DataType::UInt32),
    ]);

    let parse_options = CsvParseOptions::default()
        .with_quote_char(None)
        .with_missing_is_null(false)
        .with_separator(b'\t');
    let reader = CsvReadOptions::default()
        .with_parse_options(parse_options)
        .with_raise_if_empty(true)
        .with_has_header(true)
        .with_schema(Some(schema.into()))
        .try_into_reader_with_file_path(Some(path.into()))?;

    let df = reader.finish()?;
    Ok(df)
}

pub fn parse_contig_bin(path: &Path) -> Result<DataFrame, DataError> {
    let schema = Schema::from_iter(vec![
        Field::new("contig".into(), DataType::String),
        Field::new("bin".into(), DataType::String),
    ]);

    let parse_options = CsvParseOptions::default()
        .with_quote_char(None)
        .with_separator(b'\t');
    let reader = CsvReadOptions::default()
        .with_parse_options(parse_options)
        .with_has_header(true)
        .with_schema(Some(schema.into()))
        .try_into_reader_with_file_path(Some(path.into()))?;

    let df = reader.finish()?;
    Ok(df)
}

pub fn parse_bin_quality(path: &Path) -> Result<DataFrame, DataError> {
    let schema = Schema::from_iter(vec![
        Field::new("bin".into(), DataType::String),
        Field::new("completeness".into(), DataType::Float64),
        Field::new("contamination".into(), DataType::Float64),
    ]);

    let parse_options = CsvParseOptions::default()
        .with_quote_char(None)
        .with_separator(b'\t');
    let reader = CsvReadOptions::default()
        .with_parse_options(parse_options)
        .with_has_header(true)
        .with_schema(Some(schema.into()))
        .try_into_reader_with_file_path(Some(path.into()))?;

    let df = reader.finish()?;
    Ok(df)
}

#[cfg(test)] // This attribute means the following code is only compiled when running tests
mod tests {
    use super::*; // Import everything from the parent module (lib.rs)
    use std::io::Write; // Needed for writing to files
    use tempfile::NamedTempFile; // A crate to create temporary files easily
    // Helper function to create a temporary file with content
    fn create_temp_file(content: &str) -> NamedTempFile {
        let mut file = NamedTempFile::new().expect("Failed to create temp file");
        file.write_all(content.as_bytes())
            .expect("Failed to write to temp file");
        file
    }
    #[test]
    fn test_parse_methylation_data_valid() {
        let content = "contig\tmotif\tmod_type\tmod_position\tmethylation_value\tmean_read_cov\tn_motif_obs\tmotif_occurences_total\n\
                       contig_2\tGATC\ta\t1\t0.83\t130.9\t760\t760\n\
                       contig_3\tGATC\tm\t3\t0.003\t299.7\t698\t708";
        let file = create_temp_file(content);
        let path = file.path();
        let df = parse_methylation_data(path).expect(
            "Failed to parse valid methylation
data",
        );
        // Assertions:
        assert_eq!(df.height(), 2); // Check number of rows
        assert_eq!(df.width(), 8); // Check number of columns
        // Check a specific value (e.g., methylation_value for the first row)
        let first_methylation_value: f64 = df
            .column("methylation_value")
            .expect("Column not found")
            .get(0)
            .expect("Row not found")
            .try_extract()
            .expect("Failed to extract f64");
        assert_eq!(first_methylation_value, 0.83);
        // You can add more specific checks for other columns and data types
    }
    // #[test]
    // fn test_parse_methylation_data_malformed() {
    //     // Missing a column in one row
    //     let content = "contig\tmotif\tmod_type\tmod_position\tmethylation_value\tmean_read_cov\tn_motif_obs\tmotif_occurences_total\ncontig_2\tGATC\ta\t1\t0.83\t130.9\t760\ncontig_3\tGATC\tm\t3\t0.003\t299.7\t698\t708";
    //     let file = create_temp_file(content);
    //     let path = file.path();
    //     // This should return an error because the row structure is bad
    //     let result = parse_methylation_data(path).unwrap();
    //     println!("{:#?}", result);

    //     // assert!(result.is_err());
    //     // You could check the error message specifically if you convert to a custom DataError
    //     // let err = result.unwrap_err();
    //     // assert!(format!("{:?}", err).contains("some expected error message part"));
    // }
    // Now, write similar tests for `parse_contig_bin`
    #[test]
    fn test_parse_contig_bin_valid() {
        let content = "contig_2\tbin_A\ncontig_3\tbin_B";
        let file = create_temp_file(content);
        let path = file.path();
        let df = parse_contig_bin(path).expect("Failed to parse valid contig bin data");
        assert_eq!(df.height(), 2);
        assert_eq!(df.width(), 2);
        let first_bin = df
            .column("bin")
            .expect("Column not found")
            .str()
            .expect("Could not parse to string")
            .iter()
            .nth(0)
            .expect("Value not found")
            .unwrap();
        assert_eq!(first_bin, "bin_A");
    }
    // Add a malformed test for parse_contig_bin too
}
