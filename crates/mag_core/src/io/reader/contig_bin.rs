use std::{fs::File, io::{BufRead, BufReader}, path::Path};

use csv::{ReaderBuilder};
use serde::Deserialize;

use crate::{errors::DataError};


#[derive(Debug, Deserialize)]
pub struct ContigBinRecord {
    pub contig: String,
    pub bin: String,
}



pub struct ContigBinReader<R>
where
    R: std::io::Read,
{
    rdr: csv::Reader<R>
}

impl<R> ContigBinReader<R>
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

    pub fn records(&mut self) -> impl Iterator<Item = Result<ContigBinRecord, DataError>> + '_ {
        self.rdr.deserialize::<ContigBinRecord>().map(|r| r.map_err(DataError::from))
    }
    
    pub fn read_all(&mut self) -> Result<Vec<ContigBinRecord>, DataError> {
        self.records().collect()
    }

}

impl ContigBinReader<BufReader<File>> {
    pub fn new(path: &Path) -> Result<Self, DataError> {
        let file = File::open(path)?;
        let buf = BufReader::new(file);

        Ok(Self::from_reader(buf))

    }
}

