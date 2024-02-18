use bevy::asset::io::{AssetReader, AssetReaderError, PathStream, Reader};
use bevy::{
    asset::io::{AssetSource, VecReader},
    log::*,
    tasks::futures_lite::AsyncRead,
    utils::BoxedFuture,
};
use std::path::Path;

/// Remote assets reader
pub struct WebAssetReader {
    reader: Box<dyn AssetReader>,
    secure: bool,
}

impl WebAssetReader {
    pub fn new(secure: bool) -> Self {
        WebAssetReader {
            reader: AssetSource::get_default_reader("assets".to_string())(),
            secure,
        }
    }
    async fn download_remote<'a>(
        &'a self,
        url: &Path,
    ) -> Result<Box<dyn AsyncRead + Send + Sync + Unpin + 'a>, AssetReaderError> {
        // A simple GET request is used, but you could set custom headers, auth and so on.
        let Some(url) = url.to_str() else {
            return Err(AssetReaderError::NotFound(url.to_path_buf()));
        };
        let prefix = if self.secure { "https://" } else { "htpp://" };
        let url = format!("{prefix}{url}");
        info!("{url}");
        let request = ehttp::Request::get(url);

        let body = match ehttp::fetch_async(request).await {
            Ok(response) => {
                // Since this is an example, only check for 200 status, but in a real world use
                // it would be wise to check for others 2xx or 3xx status.
                if response.status != 200 {
                    return Err(AssetReaderError::HttpError(response.status));
                }

                response.bytes
            }
            Err(error) => {
                warn!("Failed to read remote asset: {error}");
                return Err(AssetReaderError::HttpError(500));
            }
        };

        let reader: Box<Reader> = Box::new(VecReader::new(body));
        Ok(reader)
    }
}

impl AssetReader for WebAssetReader {
    fn read<'a>(
        &'a self,
        path: &'a Path,
    ) -> BoxedFuture<'a, Result<Box<Reader<'a>>, AssetReaderError>> {
        Box::pin(self.download_remote(path))
    }

    fn read_meta<'a>(
        &'a self,
        path: &'a Path,
    ) -> BoxedFuture<'a, Result<Box<Reader<'a>>, AssetReaderError>> {
        self.reader.read_meta(path)
    }

    fn is_directory<'a>(
        &'a self,
        _path: &'a Path,
    ) -> BoxedFuture<'a, Result<bool, AssetReaderError>> {
        Box::pin(async { Ok(false) })
    }

    fn read_directory<'a>(
        &'a self,
        path: &'a Path,
    ) -> BoxedFuture<'a, Result<Box<PathStream>, AssetReaderError>> {
        self.reader.read_directory(path)
    }
}
