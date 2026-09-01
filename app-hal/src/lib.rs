pub trait AudioFileReader {
    fn open(&mut self, path: &str) -> Result<(), FileError>;
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, FileError>;
    fn seek(&mut self, pos: u64) -> Result<(), FileError>;
    fn file_len(&self) -> Result<u64, FileError>;
}

pub trait AudioOutput {
    fn write_samples(&mut self, samples: &[i16]) -> Result<(), OutputError>;
    fn sample_rate(&self) -> u32;
    fn channels(&self) -> u16;
    fn flush(&mut self) -> Result<(), OutputError>;
}

#[derive(Debug)]
pub enum FileError {
    Io,
    NotFound,
    SeekError,
}

#[derive(Debug)]
pub enum OutputError {
    Overrun,
    Underflow,
}

// #[cfg(feature = "std")]
// pub mod mock;
