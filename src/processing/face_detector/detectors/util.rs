use std::path::Path;
use std::io::{Read, IoSliceMut};

pub enum DetectorHardware {
    Cpu,
    GpuCuda,
    // GpuROCm // soon, nvidia is big boomer proprietary cuda shit so we need to ship
}

pub enum DetectorDimensionality {
    Napjak2D,
    // rushia, suisei, shion, gura, ina, matsuri, kanata - Napjak (납작) means flat in korean
    Illusion2HalfD,
    // 2.5D
    BoingBoing3D, // 3D
}

pub trait DetectorTrait {
    // la la la filler lala
}

// struct to hold model data
pub struct ModelHolder {
    data: Vec<u8>
}

impl ModelHolder {
    pub fn new(path: &str) -> Self {
        let mut data = Vec::new();
        let data_arr = include_bytes!(path);
        for byte in data_arr {
            data.push(*byte);
        }
        ModelHolder {
            data
        }
    }
}

impl Read for ModelHolder {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        unimplemented!()
    }

    fn read_to_end(&mut self, buf: &mut Vec<u8>) -> Result<usize> {
        for byte in self.data {
            buf.push(byte);
        }
        Ok(buf.len())
    }
}