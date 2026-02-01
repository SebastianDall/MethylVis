use std::{fs::File, io::{BufRead, BufReader}, path::Path};

use csv::{ReaderBuilder};
use serde::Deserialize;

use crate::{errors::DataError};


#[derive(Debug, Deserialize)]
pub struct MethylationRecord {
    pub contig: String,
    pub motif: String,
    pub mod_type: String,
    pub mod_position: u8,
    pub methylation_value: f64,
    pub mean_read_cov: f64,
    pub n_motif_obs: u32,
    pub motif_occurences_total: u32,
}



pub struct MethReader<R>
where
    R: std::io::Read,
{
    rdr: csv::Reader<R>
}

impl<R> MethReader<R>
where
    R: BufRead
{
    pub fn from_reader(reader: R) -> Self {
        let rdr = ReaderBuilder::new()
            .delimiter(b'\t')
            .has_headers(true)
            .from_reader(reader);

        Self { rdr }
    }

    pub fn records(&mut self) -> impl Iterator<Item = Result<MethylationRecord, DataError>> + '_ {
        self.rdr.deserialize::<MethylationRecord>().map(|r| r.map_err(DataError::from))
    }
    

}

impl MethReader<BufReader<File>> {
    pub fn new(path: &Path) -> Result<Self, DataError> {
        let file = File::open(path)?;
        let buf = BufReader::new(file);

        Ok(Self::from_reader(buf))

    }
}
