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

const MIN_CHUNK_SIZE: u64 = 1024 * 1024;

#[derive(Debug, Clone)]
pub enum DownloadProgress {
    Started {
        id: usize,
        filename: String,
        total_bytes: Option<u64>,
    },
    Progress {
        id: usize,
        downloaded: u64,
        total: Option<u64>,
    },
    Completed {
        id: usize,
        filepath: PathBuf,
        total_bytes: u64,
    },
    Failed {
        id: usize,
        error: String,
    },
}

pub fn extract_filename(url: &str) -> String {
    url.split('/')
        .next_back()
        .and_then(|s| {
            let s = s.split('?').next().unwrap_or(s);
            if s.is_empty() {
                None
            } else {
                Some(s.to_string())
            }
        })
        .unwrap_or_else(|| "download".to_string())
}

pub async fn extract_filename_safe(url: &str, output_dir: &Path) -> String {
    let base = extract_filename(url);
    let mut base_name = base.clone();
    let mut ext = String::new();
    if let Some(idx) = base.rfind('.') {
        ext = base[idx..].to_string();
        base_name = base[..idx].to_string();
    }

    let mut name = base.clone();
    let mut filepath = output_dir.join(&name);
    let mut counter = 1;

    while fs::try_exists(&filepath).await.unwrap_or(false) {
        name = format!("{} ({}){}", base_name, counter, ext);
        filepath = output_dir.join(&name);
        counter += 1;
    }

    name
}

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
    let mut temp_file = fs::File::create(&temp_path)
        .await
        .map_err(|e| OnionError::DownloadFailed(format!("Chunk {} temp file: {}", chunk_idx, e)))?;

    let mut stream = response.bytes_stream();
    let mut writer = BufWriter::new(&mut temp_file);

    while let Some(chunk) = stream.next().await {
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

#[allow(clippy::too_many_arguments)]
async fn download_parallel(
    client: &Client,
    url: &str,
    output_dir: &Path,
    total_size: u64,
    progress_tx: mpsc::UnboundedSender<DownloadProgress>,
    paused: Arc<AtomicBool>,
    download_id: usize,
    max_chunks: usize,
) -> Result<(), OnionError> {
    let filename = extract_filename_safe(url, output_dir).await;
    let filepath = output_dir.join(&filename);

    fs::create_dir_all(output_dir).await?;

    let temp_dir = output_dir.join(format!(
        ".tmp_{}_{}",
        download_id,
        filename.replace('.', "_")
    ));
    fs::create_dir_all(&temp_dir).await?;

    let optimal_chunks = ((total_size / MIN_CHUNK_SIZE).min(max_chunks as u64).max(1)) as usize;
    let chunk_size = total_size / optimal_chunks as u64;

    let _ = progress_tx.send(DownloadProgress::Started {
        id: download_id,
        filename: filename.clone(),
        total_bytes: Some(total_size),
    });

    let mut chunks: Vec<(u64, u64)> = Vec::with_capacity(optimal_chunks);
    for i in 0..optimal_chunks {
        let start = i as u64 * chunk_size;
        let end = if i == optimal_chunks - 1 {
            total_size - 1
        } else {
            (i as u64 + 1) * chunk_size - 1
        };
        chunks.push((start, end));
    }

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

    let mut temp_paths: Vec<PathBuf> = Vec::with_capacity(optimal_chunks);
    for handle in handles {
        let res = handle
            .await
            .map_err(|e| OnionError::DownloadFailed(format!("Join error: {}", e)));
        match res {
            Ok(Ok(temp_path)) => temp_paths.push(temp_path),
            Ok(Err(e)) => {
                let _ = fs::remove_dir_all(&temp_dir).await;
                return Err(e);
            }
            Err(e) => {
                let _ = fs::remove_dir_all(&temp_dir).await;
                return Err(e);
            }
        }
    }

    let output_file_res = fs::File::create(&filepath).await;
    if output_file_res.is_err() {
        let _ = fs::remove_dir_all(&temp_dir).await;
        return Err(OnionError::DownloadFailed(
            "Failed to create final file".to_string(),
        ));
    }
    let mut output_file = output_file_res.unwrap();

    let mut total_downloaded: u64 = 0;

    for chunk_path in &temp_paths {
        let chunk_file_res = fs::File::open(chunk_path).await;
        if chunk_file_res.is_err() {
            let _ = fs::remove_dir_all(&temp_dir).await;
            return Err(OnionError::DownloadFailed(
                "Failed to read chunk".to_string(),
            ));
        }
        let mut chunk_file = chunk_file_res.unwrap();
        let mut buffer = vec![0u8; 64 * 1024];

        loop {
            while paused.load(Ordering::Relaxed) {
                sleep(Duration::from_millis(100)).await;
            }

            match chunk_file.read(&mut buffer).await {
                Ok(0) => break,
                Ok(n) => {
                    if let Err(e) = output_file.write_all(&buffer[..n]).await {
                        let _ = fs::remove_dir_all(&temp_dir).await;
                        return Err(OnionError::DownloadFailed(format!("Write error: {}", e)));
                    }
                    total_downloaded += n as u64;
                    let _ = progress_tx.send(DownloadProgress::Progress {
                        id: download_id,
                        downloaded: total_downloaded,
                        total: Some(total_size),
                    });
                }
                Err(e) => {
                    let _ = fs::remove_dir_all(&temp_dir).await;
                    return Err(OnionError::DownloadFailed(format!("Read error: {}", e)));
                }
            }
        }
    }

    if let Err(e) = output_file.flush().await {
        let _ = fs::remove_dir_all(&temp_dir).await;
        return Err(OnionError::DownloadFailed(format!("Flush error: {}", e)));
    }

    let _ = fs::remove_dir_all(&temp_dir).await;

    let _ = progress_tx.send(DownloadProgress::Completed {
        id: download_id,
        filepath,
        total_bytes: total_downloaded,
    });

    Ok(())
}

async fn download_single(
    client: &Client,
    url: &str,
    output_dir: &Path,
    progress_tx: mpsc::UnboundedSender<DownloadProgress>,
    paused: Arc<AtomicBool>,
    download_id: usize,
) -> Result<(), OnionError> {
    let response = client
        .get(url)
        .send()
        .await
        .map_err(|e| OnionError::DownloadFailed(e.to_string()))?;

    if !response.status().is_success() {
        let err = format!("HTTP {}", response.status());
        let _ = progress_tx.send(DownloadProgress::Failed {
            id: download_id,
            error: err.clone(),
        });
        return Err(OnionError::DownloadFailed(err));
    }

    let total_bytes = response.content_length();
    let filename = extract_filename_safe(url, output_dir).await;

    fs::create_dir_all(output_dir).await?;

    let filepath = output_dir.join(&filename);
    let file = fs::File::create(&filepath).await?;
    let mut writer = BufWriter::new(file);

    let _ = progress_tx.send(DownloadProgress::Started {
        id: download_id,
        filename: filename.clone(),
        total_bytes,
    });

    let mut stream = response.bytes_stream();
    let mut downloaded: u64 = 0;

    while let Some(chunk) = stream.next().await {
        while paused.load(Ordering::Relaxed) {
            sleep(Duration::from_millis(100)).await;
        }

        let chunk = chunk.map_err(|e| OnionError::DownloadFailed(e.to_string()))?;
        writer.write_all(&chunk).await?;
        downloaded += chunk.len() as u64;

        let _ = progress_tx.send(DownloadProgress::Progress {
            id: download_id,
            downloaded,
            total: total_bytes,
        });
    }

    writer.flush().await?;

    let _ = progress_tx.send(DownloadProgress::Completed {
        id: download_id,
        filepath,
        total_bytes: downloaded,
    });

    Ok(())
}

pub async fn download_file(
    client: &Client,
    url: &str,
    output_dir: &Path,
    progress_tx: mpsc::UnboundedSender<DownloadProgress>,
    paused: Arc<AtomicBool>,
    download_id: usize,
    max_chunks: usize,
) -> Result<(), OnionError> {
    if let Some(total_size) = supports_range(client, url).await {
        if total_size >= MIN_CHUNK_SIZE {
            return download_parallel(
                client,
                url,
                output_dir,
                total_size,
                progress_tx,
                paused,
                download_id,
                max_chunks,
            )
            .await;
        }
    }

    download_single(client, url, output_dir, progress_tx, paused, download_id).await
}
