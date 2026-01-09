// src-tauri/src/cleanup/secure_delete.rs
use std::fs::{self, File, OpenOptions};
use std::io::{Seek, SeekFrom, Write};
use std::path::Path;

pub fn secure_delete(path: &Path) -> std::io::Result<()> {
    if path.is_dir() {
        for entry in fs::read_dir(path)? {
            let entry = entry?;
            secure_delete(&entry.path())?;
        }
        fs::remove_dir(path)?;
    } else {
        let metadata = fs::metadata(path)?;
        let size = metadata.len() as usize;

        if size > 0 {
            let mut file = OpenOptions::new()
                .write(true)
                .open(path)?;

            overwrite_file(&mut file, size, 0x00)?;
            overwrite_file(&mut file, size, 0xFF)?;
            overwrite_with_random(&mut file, size)?;

            file.sync_all()?;
        }

        fs::remove_file(path)?;
    }

    Ok(())
}

fn overwrite_file(file: &mut File, size: usize, byte: u8) -> std::io::Result<()> {
    file.seek(SeekFrom::Start(0))?;
    
    let buffer = vec![byte; 65536.min(size)];
    let mut remaining = size;
    
    while remaining > 0 {
        let to_write = remaining.min(buffer.len());
        file.write_all(&buffer[..to_write])?;
        remaining -= to_write;
    }
    
    file.sync_all()?;
    Ok(())
}

fn overwrite_with_random(file: &mut File, size: usize) -> std::io::Result<()> {
    use std::collections::hash_map::RandomState;
    use std::hash::{BuildHasher, Hasher};
    
    file.seek(SeekFrom::Start(0))?;
    
    let mut buffer = vec![0u8; 65536.min(size)];
    let hasher_builder = RandomState::new();
    let mut remaining = size;
    let mut seed: u64 = 0;
    
    while remaining > 0 {
        for chunk in buffer.chunks_mut(8) {
            let mut hasher = hasher_builder.build_hasher();
            hasher.write_u64(seed);
            seed = hasher.finish();
            let bytes = seed.to_le_bytes();
            for (i, b) in chunk.iter_mut().enumerate() {
                if i < bytes.len() {
                    *b = bytes[i];
                }
            }
        }
        
        let to_write = remaining.min(buffer.len());
        file.write_all(&buffer[..to_write])?;
        remaining -= to_write;
    }
    
    file.sync_all()?;
    Ok(())
}
