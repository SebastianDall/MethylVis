use std::{
    collections::{BTreeMap, HashMap},
    str::FromStr,
};

use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::{
    contig::{Assignment, ContigAssignment, ContigId},
    io::reader::{checkm2::Checkm2Record, contig_bin::ContigBinRecord},
};

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Deserialize, Serialize, TS)]
#[ts(export, export_to = "../../../contam-map-frontend/src/bindings/")]
pub struct BinId(pub String);

#[derive(Serialize, Deserialize, Debug, Clone, TS)]
#[ts(export, export_to = "../../../contam-map-frontend/src/bindings/")]
pub struct Bin {
    pub id: BinId,
    pub contig_metadata: Vec<ContigAssignment>,
    pub completeness: Option<f64>,
    pub contamination: Option<f64>,
    pub quality: Option<BinQuality>,
}

impl Bin {
    pub fn from_records(
        contig_bin_records: Vec<ContigBinRecord>,
        checkm2_records: Vec<Checkm2Record>,
    ) -> BTreeMap<BinId, Bin> {
        let quality_map = checkm2_records
            .iter()
            .map(|r| (r.bin.clone(), r))
            .collect::<HashMap<String, &Checkm2Record>>();

        let mut bins: BTreeMap<BinId, Bin> = BTreeMap::new();
        for cb_rec in contig_bin_records {
            let binid = BinId(cb_rec.bin.clone());
            let contig_id = ContigId(cb_rec.contig.clone());
            bins.entry(binid.clone())
                .and_modify(|b| {
                    b.contig_metadata.push(ContigAssignment::new(
                        contig_id.clone(),
                        crate::contig::Assignment::None,
                    ))
                })
                .or_insert(Bin {
                    id: binid,
                    contig_metadata: vec![ContigAssignment::new(
                        contig_id,
                        crate::contig::Assignment::None,
                    )],
                    completeness: None,
                    contamination: None,
                    quality: None,
                });
        }

        bins.into_iter()
            .map(|(bin_id, bin)| {
                let quality = quality_map.get(&bin_id.0);

                let updated_bin = Bin {
                    completeness: quality.map(|q| q.completeness),
                    contamination: quality.map(|q| q.contamination),
                    quality: quality
                        .map(|q| BinQuality::from_values(q.completeness, q.contamination)),
                    ..bin
                };

                (bin_id, updated_bin)
            })
            .collect()
    }

    pub fn to_metadata_records(&self) -> Vec<BinMetadataRecord> {
        self.contig_metadata
            .iter()
            .map(|c| BinMetadataRecord {
                id: self.id.clone(),
                contig_id: c.contig_id.clone(),
                assignment: c.assignment.clone(),
                completeness: self.completeness,
                contamination: self.contamination,
                quality: self.quality.clone(),
            })
            .collect()
    }
}

#[derive(Serialize, Deserialize)]
pub struct BinMetadataRecord {
    pub id: BinId,
    pub contig_id: ContigId,
    pub assignment: Assignment,
    pub completeness: Option<f64>,
    pub contamination: Option<f64>,
    pub quality: Option<BinQuality>,
}

impl BinMetadataRecord {}

#[derive(Serialize, Deserialize, Debug, Clone, TS)]
#[ts(export, export_to = "../../../contam-map-frontend/src/bindings/")]
pub enum BinQuality {
    HQ,
    MQ,
    LQ,
}

impl BinQuality {
    pub fn from_values(completenss: f64, contamination: f64) -> Self {
        match (completenss, contamination) {
            (comp, cont) if comp > 90.0 && cont < 5.0 => BinQuality::HQ,
            (comp, cont) if comp > 50.0 && cont <= 10.0 => BinQuality::MQ,
            _ => BinQuality::LQ,
        }
    }
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

impl FromStr for BinQuality {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "HQ" => Ok(BinQuality::HQ),
            "MQ" => Ok(BinQuality::MQ),
            "LQ" => Ok(BinQuality::LQ),
            _ => Err(format!("Could not convert '{}' to BinQuality.", s)),
        }
    }
}
