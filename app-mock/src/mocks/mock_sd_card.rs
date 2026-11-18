use std::{convert::Infallible, io::Read};

use aligned::{A4, Aligned};
use block_device_driver::BlockDevice;
use flate2::read::GzDecoder;

use app_core::file_manager::filesystem_io::SD_BLOCK_SIZE;

pub struct MockBlockDevice {
    bytes: Vec<u8>,
}

impl MockBlockDevice {
    pub fn new() -> Self {
        const IMG_GZ: &[u8] = include_bytes!("../../assets/sd.img.gz");
        let mut bytes = Vec::new();
        GzDecoder::new(IMG_GZ)
            .read_to_end(&mut bytes)
            .expect("failed to gunzip sd img");
        Self { bytes }
    }
}

impl BlockDevice<SD_BLOCK_SIZE> for MockBlockDevice {
    type Error = Infallible;
    type Align = A4;

    async fn read(
        &mut self,
        block_address: u32,
        data: &mut [Aligned<Self::Align, [u8; SD_BLOCK_SIZE]>],
    ) -> Result<(), Self::Error> {
        for (i, block) in data.iter_mut().enumerate() {
            let start = (block_address as usize + i) * SD_BLOCK_SIZE;
            let end = start + SD_BLOCK_SIZE;
            block.copy_from_slice(&self.bytes[start..end]);
        }
        Ok(())
    }

    async fn write(
        &mut self,
        block_address: u32,
        data: &[Aligned<Self::Align, [u8; SD_BLOCK_SIZE]>],
    ) -> Result<(), Self::Error> {
        for (i, block) in data.iter().enumerate() {
            let start = (block_address as usize + i) * SD_BLOCK_SIZE;
            let end = start + SD_BLOCK_SIZE;
            self.bytes[start..end].copy_from_slice(block.as_slice());
        }
        Ok(())
    }

    async fn size(&mut self) -> Result<u64, Self::Error> {
        Ok(self.bytes.len() as u64)
    }
}
