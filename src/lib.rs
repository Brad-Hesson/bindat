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
enum ReadError {
    #[error("IO Error: {0}")]
    IoError(#[from] io::Error),
    #[error("UTF8 Error: {0}")]
    Utf8Error(#[from] Utf8Error),
    #[error("Json Parse Error: {0}")]
    JsonError(#[from] serde_json::Error),
}

impl BinDat {
    fn new() -> Self {
        Self {
            datasets: Default::default(),
            metadata: serde_json::Value::Null,
        }
    }
    fn from_reader(mut reader: impl BufRead) -> Result<Self, ReadError> {
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

    fn to_writer(&self, mut writer: impl Write) -> io::Result<()> {
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
