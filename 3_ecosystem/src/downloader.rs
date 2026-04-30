use thiserror::Error;

// ---------------------------------------------------------------------------
// Errors
// ---------------------------------------------------------------------------

#[derive(Debug, Error)]
pub enum DownloadError {
    #[error("HTTP request failed for '{url}': {source}")]
    RequestFailed {
        url: String,
        #[source]
        source: reqwest::Error,
    },

    #[error("server returned non-200 status {status} for '{url}'")]
    NonSuccessStatus { url: String, status: u16 },

    #[error("failed to read response body for '{url}': {source}")]
    BodyRead {
        url: String,
        #[source]
        source: reqwest::Error,
    },
}

// ---------------------------------------------------------------------------
// 6.1 download
// ---------------------------------------------------------------------------

/// Download the resource at `url` and return its raw bytes.
///
/// Returns `DownloadError::NonSuccessStatus` for any HTTP status other than 2xx.
///
/// Requirements: 1.1
pub async fn download(client: &reqwest::Client, url: &str) -> Result<Vec<u8>, DownloadError> {
    let response = client
        .get(url)
        .send()
        .await
        .map_err(|e| DownloadError::RequestFailed {
            url: url.to_owned(),
            source: e,
        })?;

    let status = response.status();
    if !status.is_success() {
        return Err(DownloadError::NonSuccessStatus {
            url: url.to_owned(),
            status: status.as_u16(),
        });
    }

    let bytes = response
        .bytes()
        .await
        .map_err(|e| DownloadError::BodyRead {
            url: url.to_owned(),
            source: e,
        })?;

    Ok(bytes.to_vec())
}
