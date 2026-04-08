use crate::error::OnionError;
use futures_util::StreamExt;
use reqwest::Client;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tokio::fs;
use tokio::io::{AsyncReadExt, AsyncWriteExt, BufWriter};
use tokio::sync::mpsc;
use tokio::time::{sleep, Duration};

/// Number of parallel connections for chunk downloading
const MAX_CHUNKS: usize = 100;
/// Minimum chunk size (1 MB) - smaller files use single connection
const MIN_CHUNK_SIZE: u64 = 1024 * 1024;
#[derive(Debug, Clone)]
pub enum DownloadProgress {
    Started {
        filename: String,
        total_bytes: Option<u64>,
    },
    Progress {
        downloaded: u64,
        total: Option<u64>,
    },
    Completed {
        filepath: PathBuf,
        total_bytes: u64,
    },
    Failed {
        error: String,
    },
}

/// Extract a usable filename from a URL path segment.
pub fn extract_filename(url: &str) -> String {
    url.split('/')
        .next_back()
        .and_then(|s| {
            let s = s.split('?').next().unwrap_or(s);
            if s.is_empty() { None } else { Some(s.to_string()) }
        })
        .unwrap_or_else(|| "download".to_string())
}

/// Check if server supports range requests
async fn supports_range(client: &Client, url: &str) -> Option<u64> {
    let response = client.head(url).send().await.ok()?;
    if !response.status().is_success() {
        return None;
    }

    let accept_ranges = response
        .headers()
        .get("accept-ranges")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_lowercase().contains("bytes"))
        .unwrap_or(false);

    if !accept_ranges {
        return None;
    }

    response.content_length()
}

/// Download a single chunk and return the temp file path
async fn download_chunk(
    client: Client,
    url: String,
    start: u64,
    end: u64,
    chunk_idx: usize,
    temp_dir: &Path,
    paused: Arc<AtomicBool>,
) -> Result<PathBuf, OnionError> {
    let range_header = format!("bytes={}-{}", start, end);

    let response = client
        .get(&url)
        .header("Range", range_header)
        .send()
        .await
        .map_err(|e| OnionError::DownloadFailed(format!("Chunk {}: {}", chunk_idx, e)))?;

    if !response.status().is_success() && response.status().as_u16() != 206 {
        return Err(OnionError::DownloadFailed(format!(
            "Chunk {}: HTTP {}",
            chunk_idx,
            response.status()
        )));
    }

    let temp_path = temp_dir.join(format!("chunk_{}", chunk_idx));
    let mut temp_file = fs::File::create(&temp_path).await.map_err(|e| {
        OnionError::DownloadFailed(format!("Chunk {} temp file: {}", chunk_idx, e))
    })?;

    let mut stream = response.bytes_stream();
    let mut writer = BufWriter::new(&mut temp_file);

    while let Some(chunk) = stream.next().await {
        // Check if paused and wait
        while paused.load(Ordering::Relaxed) {
            sleep(Duration::from_millis(100)).await;
        }
        
        let chunk = chunk.map_err(|e| {
            OnionError::DownloadFailed(format!("Chunk {} stream error: {}", chunk_idx, e))
        })?;
        writer
            .write_all(&chunk)
            .await
            .map_err(|e| OnionError::DownloadFailed(format!("Chunk {} write: {}", chunk_idx, e)))?;
    }

    writer
        .flush()
        .await
        .map_err(|e| OnionError::DownloadFailed(format!("Chunk {} flush: {}", chunk_idx, e)))?;

    Ok(temp_path)
}

/// Download file using parallel chunks for maximum speed
async fn download_parallel(
    client: &Client,
    url: &str,
    output_dir: &Path,
    total_size: u64,
    progress_tx: mpsc::UnboundedSender<DownloadProgress>,
    paused: Arc<AtomicBool>,
) -> Result<(), OnionError> {
    let filename = extract_filename(url);
    let filepath = output_dir.join(&filename);

    fs::create_dir_all(output_dir).await?;

    // Create temp directory for chunks
    let temp_dir = output_dir.join(format!(".tmp_{}", filename.replace('.', "_")));
    fs::create_dir_all(&temp_dir).await?;

    // Calculate optimal chunk count (up to MAX_CHUNKS)
    let optimal_chunks = ((total_size / MIN_CHUNK_SIZE).min(MAX_CHUNKS as u64).max(1)) as usize;
    let chunk_size = total_size / optimal_chunks as u64;

    let _ = progress_tx.send(DownloadProgress::Started {
        filename: filename.clone(),
        total_bytes: Some(total_size),
    });

    // Create chunk ranges
    let mut chunks: Vec<(u64, u64)> = Vec::with_capacity(optimal_chunks);
    for i in 0..optimal_chunks {
        let start = i as u64 * chunk_size;
        let end = if i == optimal_chunks - 1 {
            total_size - 1 // Last chunk goes to end
        } else {
            (i as u64 + 1) * chunk_size - 1
        };
        chunks.push((start, end));
    }

    // Spawn parallel downloads
    let mut handles = Vec::new();
    for (idx, (start, end)) in chunks.into_iter().enumerate() {
        let client = client.clone();
        let url = url.to_string();
        let chunk_temp_dir = temp_dir.clone();
        let chunk_paused = paused.clone();
        let handle = tokio::spawn(async move {
            download_chunk(client, url, start, end, idx, &chunk_temp_dir, chunk_paused).await
        });
        handles.push(handle);
    }

    // Collect all temp file paths
    let mut temp_paths: Vec<PathBuf> = Vec::with_capacity(optimal_chunks);
    for handle in handles {
        let temp_path = handle
            .await
            .map_err(|e| OnionError::DownloadFailed(format!("Join error: {}", e)))??;
        temp_paths.push(temp_path);
    }

    // Merge chunks into final file
    let mut output_file = fs::File::create(&filepath).await?;
    let mut total_downloaded: u64 = 0;

    for chunk_path in &temp_paths {
        let mut chunk_file = fs::File::open(chunk_path).await?;
        let mut buffer = vec![0u8; 64 * 1024]; // 64KB read buffer

        loop {
            // Check if paused during merge
            while paused.load(Ordering::Relaxed) {
                sleep(Duration::from_millis(100)).await;
            }
            
            match chunk_file.read(&mut buffer).await {
                Ok(0) => break,
                Ok(n) => {
                    output_file
                        .write_all(&buffer[..n])
                        .await
                        .map_err(|e| OnionError::DownloadFailed(format!("Write error: {}", e)))?;
                    total_downloaded += n as u64;
                    let _ = progress_tx.send(DownloadProgress::Progress {
                        downloaded: total_downloaded,
                        total: Some(total_size),
                    });
                }
                Err(e) => {
                    return Err(OnionError::DownloadFailed(format!("Read error: {}", e)));
                }
            }
        }
    }

    output_file.flush().await?;

    // Clean up temp files and directory
    for chunk_path in temp_paths {
        let _ = fs::remove_file(&chunk_path).await;
    }
    let _ = fs::remove_dir(&temp_dir).await;

    let _ = progress_tx.send(DownloadProgress::Completed {
        filepath,
        total_bytes: total_downloaded,
    });

    Ok(())
}

/// Fallback: Single-threaded download for servers without range support
async fn download_single(
    client: &Client,
    url: &str,
    output_dir: &Path,
    progress_tx: mpsc::UnboundedSender<DownloadProgress>,
    paused: Arc<AtomicBool>,
) -> Result<(), OnionError> {
    let response = client
        .get(url)
        .send()
        .await
        .map_err(|e| OnionError::DownloadFailed(e.to_string()))?;

    if !response.status().is_success() {
        let err = format!("HTTP {}", response.status());
        let _ = progress_tx.send(DownloadProgress::Failed { error: err.clone() });
        return Err(OnionError::DownloadFailed(err));
    }

    let total_bytes = response.content_length();
    let filename = extract_filename(url);

    fs::create_dir_all(output_dir).await?;

    let filepath = output_dir.join(&filename);
    let file = fs::File::create(&filepath).await?;
    let mut writer = BufWriter::new(file);

    let _ = progress_tx.send(DownloadProgress::Started {
        filename: filename.clone(),
        total_bytes,
    });

    let mut stream = response.bytes_stream();
    let mut downloaded: u64 = 0;

    while let Some(chunk) = stream.next().await {
        // Check if paused and wait
        while paused.load(Ordering::Relaxed) {
            sleep(Duration::from_millis(100)).await;
        }
        
        let chunk = chunk.map_err(|e| OnionError::DownloadFailed(e.to_string()))?;
        writer.write_all(&chunk).await?;
        downloaded += chunk.len() as u64;

        let _ = progress_tx.send(DownloadProgress::Progress {
            downloaded,
            total: total_bytes,
        });
    }

    writer.flush().await?;

    let _ = progress_tx.send(DownloadProgress::Completed {
        filepath,
        total_bytes: downloaded,
    });

    Ok(())
}

/// Download a file from a URL through the Tor proxy.
/// Uses parallel chunk downloading (up to 100 connections) if server supports Range requests.
pub async fn download_file(
    client: &Client,
    url: &str,
    output_dir: &Path,
    progress_tx: mpsc::UnboundedSender<DownloadProgress>,
    paused: Arc<AtomicBool>,
) -> Result<(), OnionError> {
    // Check if parallel download is possible (server supports ranges and file is large enough)
    if let Some(total_size) = supports_range(client, url).await {
        // Use parallel chunks for files >= MIN_CHUNK_SIZE (1 MB)
        if total_size >= MIN_CHUNK_SIZE {
            return download_parallel(client, url, output_dir, total_size, progress_tx, paused).await;
        }
    }

    // Fallback to single-threaded download
    download_single(client, url, output_dir, progress_tx, paused).await
}
