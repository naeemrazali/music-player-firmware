use exfat_slim::asynchronous::{
    BlockDevice, error::ExFatError, file::OpenOptions, file_system::FileSystem,
};

pub enum FsError {
    NotFound,
    Io,
}

pub async fn read_at<D: BlockDevice<512>, const N: usize>(
    fs: &mut FileSystem<D, 512, N>,
    path: &str,
    offset: u64,
    buf: &mut [u8],
) -> Result<(), FsError> {
    // open file
    //
    // seek to offset
    //
    // read file
    //
    let mut file = fs
        .open(path, OpenOptions::new().read(true))
        .await
        .map_err(map_err)?;

    file.seek(fs, offset).await.map_err(map_err)?;
    let n = file.read(fs, buf).await.map_err(map_err)?;
    match n {
        Some(n) if n == buf.len() => Ok(()),
        _ => Err(FsError::Io),
    }
}

fn map_err<E>(e: ExFatError<E>) -> FsError {
    match e {
        ExFatError::FileNotFound | ExFatError::DirectoryNotFound => FsError::NotFound,
        _ => FsError::Io,
    }
}
