use std::{
    io::{self, BufRead, ErrorKind, Write},
    str::Utf8Error,
};

use byteorder::{BigEndian, ReadBytesExt};
pub use serde_json;
use thiserror::Error;

#[derive(Debug)]
pub struct BinDat {
    pub datasets: Vec<Vec<f64>>,
    pub metadata: serde_json::Value,
}

impl Default for BinDat {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Error)]
pub enum ReadError {
    #[error("IO Error: {0}")]
    IoError(#[from] io::Error),
    #[error("UTF8 Error: {0}")]
    Utf8Error(#[from] Utf8Error),
    #[error("Json Parse Error: {0}")]
    JsonError(#[from] serde_json::Error),
}

impl BinDat {
    pub fn new() -> Self {
        Self {
            datasets: Default::default(),
            metadata: serde_json::Value::Null,
        }
    }
    pub fn from_reader(mut reader: impl BufRead) -> Result<Self, ReadError> {
        let mut json_bytes = Vec::new();
        let len = reader.read_until(b'\0', &mut json_bytes)?;
        let json_str = std::str::from_utf8(&json_bytes[..len - 1])?;
        let metadata = serde_json::from_str(json_str)?;

        let mut datasets = Vec::new();
        loop {
            let rows = match reader.read_u64::<BigEndian>() {
                Ok(rows) => rows,
                Err(e) if e.kind() == ErrorKind::UnexpectedEof => break,
                Err(e) => return Err(e.into()),
            };
            let mut vec = vec![0f64; rows as usize];
            reader.read_exact(bytemuck::cast_slice_mut(vec.as_mut_slice()))?;
            datasets.push(vec);
        }

        Ok(Self { datasets, metadata })
    }

    pub fn to_writer(&self, mut writer: impl Write) -> io::Result<()> {
        serde_json::to_writer_pretty(&mut writer, &self.metadata)?;
        writer.write(b"\n\0")?;

        for dataset in &self.datasets {
            let rows = dataset.len() as u64;
            writer.write(&rows.to_be_bytes())?;
            writer.write(bytemuck::cast_slice(dataset.as_slice()))?;
        }
        writer.flush()?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::io::BufWriter;

    use super::*;
    use serde::{Deserialize, Serialize};
    use serde_json::json;

    #[derive(Debug, Serialize, Deserialize)]
    struct Test {
        peak_peak: f64,
        rate: f64,
    }

    #[test]
    fn test_write() -> Result<(), Box<dyn std::error::Error>> {
        let file = BufWriter::new(
            std::fs::OpenOptions::new()
                .create(true)
                .truncate(true)
                .write(true)
                .open("test.dat")?,
        );
        let mut dat = BinDat::new();
        let params = Test {
            peak_peak: 0.,
            rate: 0.,
        };
        dat.metadata = json!({"id": "0032", "test_parameters": params});
        dat.datasets.push(vec![1., 2., 3., 4.]);
        dat.datasets.push(vec![1.; 10000000]);
        dat.to_writer(file)?;
        Ok(())
    }
}
