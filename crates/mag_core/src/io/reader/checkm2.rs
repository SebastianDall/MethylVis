use std::{
    fs::File,
    io::{BufRead, BufReader},
    path::Path,
};

use csv::ReaderBuilder;
use serde::Deserialize;

use crate::errors::DataError;

#[derive(Debug, Deserialize)]
pub struct Checkm2Record {
    #[serde(rename = "Name")]
    pub bin: String,

    #[serde(rename = "Completeness")]
    pub completeness: f64,

    #[serde(rename = "Contamination")]
    pub contamination: f64,

    #[serde(rename = "GC_Content")]
    pub gc_content: f64,

    #[serde(rename = "Genome_Size")]
    pub genome_size: f64,
}

pub struct CheckM2Reader<R>
where
    R: std::io::Read,
{
    rdr: csv::Reader<R>,
}

impl<R> CheckM2Reader<R>
where
    R: BufRead,
{
    pub fn from_reader(reader: R) -> Self {
        let rdr = ReaderBuilder::new()
            .delimiter(b'\t')
            .has_headers(true)
            .from_reader(reader);

        Self { rdr }
    }

    pub fn records(&mut self) -> impl Iterator<Item = Result<Checkm2Record, DataError>> + '_ {
        self.rdr
            .deserialize::<Checkm2Record>()
            .map(|r| r.map_err(DataError::from))
    }

    pub fn read_all(&mut self) -> Result<Vec<Checkm2Record>, DataError> {
        self.records().collect()
    }
}

impl CheckM2Reader<BufReader<File>> {
    pub fn new(path: &Path) -> Result<Self, DataError> {
        let file = File::open(path)?;
        let buf = BufReader::new(file);

        let reader = Self::from_reader(buf);

        // if let Some(headers) = reader.rdr.headers().ok() {
        //     eprintln!("Headers found: {:?}", headers);
        //     for (i, h) in headers.iter().enumerate() {
        //         eprintln!("    [{}]: '{}'", i, h);
        //     }
        // }
        Ok(reader)
    }
}
