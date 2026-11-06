use exfat_slim::asynchronous::{
    BlockDevice,
    error::ExFatError,
    file::{File, OpenOptions},
    file_system::FileSystem,
};

pub enum FsError {
    NotFound,
    Io,
}

pub async fn open_read<D: BlockDevice<512>, const N: usize>(
    fs: &mut FileSystem<D, 512, N>,
    path: &str,
) -> Result<File, FsError> {
    fs.open(path, OpenOptions::new().read(true))
        .await
        .map_err(exfat_err_to_fs_err)
}

pub async fn read_at<D: BlockDevice<512>, const N: usize>(
    fs: &mut FileSystem<D, 512, N>,
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
