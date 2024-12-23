use anyhow::Result;
use std::path::Path;
use tempfile::tempfile;
use tokio::fs::{create_dir_all, File};
use tokio::io::{copy, AsyncSeekExt, AsyncWriteExt};

pub struct Tempfile(pub File);

impl Tempfile {
    pub async fn create() -> Result<Self> {
        let tmp = tempfile()?;
        Ok(Self(tmp.into()))
    }

    pub async fn write_chunk(&mut self, chunk: &[u8]) -> Result<()> {
        AsyncWriteExt::write_all(&mut self.0, chunk).await?;
        Ok(())
    }

    pub async fn finalize(&mut self) -> Result<()> {
        self.0.rewind().await?;
        Ok(())
    }

    /// Writes a temporary file to a path
    pub async fn persist(&mut self, path: impl AsRef<Path>) -> Result<File> {
        let path = path.as_ref();
        if let Some(dir) = path.parent() {
            create_dir_all(dir).await?;
        }
        self.0.rewind().await?;
        let mut dest = File::create(path).await?;
        copy(&mut self.0, &mut dest).await?;
        dest.rewind().await?;
        Ok(dest)
    }
}
