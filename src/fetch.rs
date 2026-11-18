//! Script fetcher: only `data:` URLs and `file:` URLs under a root directory.
//! Network access is not supported.

use std::path::{Path, PathBuf};

use blitz_vibey_script::{DefaultScriptFetcher, FetchError, ScriptFetcher};
use url::Url;

pub struct SandboxedFetcher {
    root: PathBuf,
}

impl SandboxedFetcher {
    /// `root` must exist; files outside it are refused.
    pub fn new(root: &Path) -> std::io::Result<Self> {
        Ok(Self { root: root.canonicalize()? })
    }
}

impl ScriptFetcher for SandboxedFetcher {
    fn fetch(&self, url: &Url) -> Result<String, FetchError> {
        match url.scheme() {
            "data" => DefaultScriptFetcher.fetch(url),
            "file" => {
                let path = url
                    .to_file_path()
                    .map_err(|_| FetchError::InvalidData(format!("invalid file URL: {url}")))?;
                let path = path.canonicalize().map_err(FetchError::Io)?;
                if !path.starts_with(&self.root) {
                    return Err(FetchError::InvalidData(format!(
                        "{} is outside the allowed directory {}",
                        path.display(),
                        self.root.display()
                    )));
                }
                std::fs::read_to_string(path).map_err(FetchError::Io)
            }
            scheme => Err(FetchError::UnsupportedScheme(scheme.to_string())),
        }
    }
}
