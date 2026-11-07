use exfat_slim::asynchronous::{
    BlockDevice,
    error::ExFatError,
    file::{File, OpenOptions},
    file_system::FileSystem,
};

pub const SD_BLOCK_SIZE: usize = 512;
pub const CACHE_DEPTH: usize = 8;

pub type Fs<D> = FileSystem<D, SD_BLOCK_SIZE, CACHE_DEPTH>;

pub enum FsError {
    NotFound,
    Io,
}

pub async fn open_read<D: BlockDevice<SD_BLOCK_SIZE>>(
    fs: &mut Fs<D>,
    path: &str,
) -> Result<File, FsError> {
    fs.open(path, OpenOptions::new().read(true))
        .await
        .map_err(exfat_err_to_fs_err)
}

pub async fn read_at<D: BlockDevice<SD_BLOCK_SIZE>>(
    fs: &mut Fs<D>,
    file: &mut File,
    offset: u64,
    buf: &mut [u8],
) -> Result<(), FsError> {
    file.seek(fs, offset).await.map_err(exfat_err_to_fs_err)?;
    let n = file.read(fs, buf).await.map_err(exfat_err_to_fs_err)?;
    // TODO(errors): split Truncated (EOF mid-fill / SeekOutOfRange) out of Io
    match n {
        Some(n) if n == buf.len() => Ok(()),
        _ => Err(FsError::Io),
    }
}

fn exfat_err_to_fs_err<E>(e: ExFatError<E>) -> FsError {
    match e {
        ExFatError::FileNotFound | ExFatError::DirectoryNotFound => FsError::NotFound,
        _ => FsError::Io,
    }
}
