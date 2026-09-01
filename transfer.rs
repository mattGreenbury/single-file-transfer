fn mock_transfer_file(
    path: &Path,
    receiver: &mut MockReceiver,
    progress_map: &mut HashMap<PathBuf, TransferProgress>,
) -> Result<()> {
    // Folders are not sent as file bytes
    if !path.is_file() {
        return Ok(());
    }

    let file_size = std::fs::metadata(path)?.len();
    let source_hash = hash_file(path)?;

    // Resume from the last byte the server acknowledged or 0
    let progress = progress_map.entry(path.to_path_buf()).or_insert(
        TransferProgress {
            file_size,
            bytes_acked: receiver.acked.get(path).copied().unwrap_or(0),
            source_hash,
        },
    );
    progress.file_size = file_size;
    progress.source_hash = source_hash;

    // Empty file - no packets but still check the hash
    if file_size == 0 {
        receiver.files.insert(path.to_path_buf(), Vec::new());
        receiver.acked.insert(path.to_path_buf(), 0);
        return receiver.verify_file(path, source_hash);
    }

    let data = std::fs::read(path)?;
    let mut offset = progress.bytes_acked;

    while offset < file_size {
        let start = offset as usize;
        let end = ((offset + CHUNK_SIZE) as usize).min(data.len());
        let chunk = data[start..end].to_vec();

        // One packet - location/bytes/hash of bytes
        let packet = Packet {
            path: path.display().to_string(),
            offset,
            data: chunk,
            packet_hash: *blake3::hash(&chunk).as_bytes(),
        };

        let acked = receiver.accept_packet(&packet)?;
        progress.bytes_acked = acked;
        offset = acked;
    }

    // File sent - compare full hash with client
    receiver.verify_file(path, source_hash)?;
    progress_map.remove(path);
    Ok(())
}