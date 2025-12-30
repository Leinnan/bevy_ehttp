use bevy_asset::io::{
    AssetReader, AssetReaderError, AssetSource, ErasedAssetReader, PathStream, Reader,
    ReaderRequiredFeatures, VecReader,
};
use bevy_log::*;
use bevy_tasks::ConditionalSendFuture;
use std::path::Path;

/// Remote assets reader
pub struct WebAssetReader<const SECURE: bool> {
    reader: Box<dyn ErasedAssetReader>,
}

impl<const SECURE: bool> Default for WebAssetReader<SECURE> {
    fn default() -> Self {
        WebAssetReader {
            reader: AssetSource::get_default_reader("assets".to_string())(),
        }
    }
}

impl<const SECURE: bool> WebAssetReader<SECURE> {
    async fn download_remote<'a>(
        &'a self,
        url: &Path,
    ) -> Result<Box<dyn Reader + 'a>, AssetReaderError> {
        // A simple GET request is used, but you could set custom headers, auth and so on.
        let Some(url) = url.to_str() else {
            return Err(AssetReaderError::NotFound(url.to_path_buf()));
        };
        let prefix = if SECURE { "https://" } else { "http://" };
        let url = format!("{prefix}{url}");
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

        let reader: Box<dyn Reader> = Box::new(VecReader::new(body));
        Ok(reader)
    }
}

impl<const SECURE: bool> AssetReader for WebAssetReader<SECURE> {
    fn read<'a>(
        &'a self,
        path: &'a Path,
        _required_features: ReaderRequiredFeatures,
    ) -> impl ConditionalSendFuture<Output = Result<Box<dyn Reader + 'a>, AssetReaderError>> {
        self.download_remote(path)
    }

    fn read_meta<'a>(
        &'a self,
        path: &'a Path,
    ) -> impl ConditionalSendFuture<Output = Result<Box<dyn Reader + 'a>, AssetReaderError>> {
        self.reader.read_meta(path)
    }

    async fn is_directory<'a>(&'a self, _path: &'a Path) -> Result<bool, AssetReaderError> {
        Ok(false)
    }

    async fn read_directory<'a>(
        &'a self,
        path: &'a Path,
    ) -> Result<Box<PathStream>, AssetReaderError> {
        Err(AssetReaderError::NotFound(path.to_path_buf()))
    }
}
