use wde_logger::*;
use winapi::um::winnt::{FILE_SHARE_READ, FILE_SHARE_WRITE, FILE_SHARE_DELETE, FILE_ATTRIBUTE_TEMPORARY};
use std::os::windows::{fs::OpenOptionsExt, io::AsRawHandle};
use win_sys::{CreateFileMapping, HANDLE, PAGE_READWRITE, MapViewOfFile, FILE_MAP_READ, FILE_MAP_WRITE, ViewOfFile, FileMapping};

use crate::EditorError;

/// Shared memory wrapper.
/// 
/// # Example
/// 
/// ```
/// let mut mem = SharedMemory::new(shared_memory_size);
/// mem.write(string_message.as_bytes());
/// ```
pub struct SharedMemory {
    /// Path to the physical file.
    file_path: String,
    /// Index in the shared memory.
    shared_index: String,
    /// File mapping handle.
    #[allow(dead_code)]
    file_mapping: FileMapping,
    /// File view handle.
    file_view: ViewOfFile,
}

impl SharedMemory {
    /// Create a new shared memory space.
    /// 
    /// # Arguments
    /// 
    /// * `size` - Size of the shared memory to allocate.
    pub fn new(size: usize) -> Result<Self, EditorError> {
        trace!("Creating shared memory of {} bytes.", size);

        // Create random temporary file name
        let tmp_id = format!("wde_{:X}", rand::random::<u64>());
        let mut file_path = std::env::temp_dir();
        file_path.push("WaterDropEngine");
        if !file_path.is_dir() {
            if let Err(e) = std::fs::create_dir_all(&file_path) {
                error!("Unable to create temporary directory at {} : {e}.", file_path.to_str().unwrap());
                return Err(EditorError::SharedMemoryCannotCreateDir);
            }
        }
        file_path.push(tmp_id.clone());

        // Check if file already exists
        if file_path.exists() {
            error!("File already exists at '{}'.", file_path.to_str().unwrap());
            return Err(EditorError::SharedMemoryAlreadyExists);
        }

        // Create physical file
        let file = match std::fs::OpenOptions::new()
            .read(true) // Allow read
            .write(true) // Allow write
            .share_mode(FILE_SHARE_READ | FILE_SHARE_WRITE | FILE_SHARE_DELETE) // Allow other processes to read/write/delete
            .attributes(FILE_ATTRIBUTE_TEMPORARY) // Delete file when last handle is closed
            .create_new(true) // Create new file (fail if already exists)
            .open(&file_path) {
                Ok(file) => Some(file),
                Err(e) => {
                    error!("Unable to open physical file '{}' : {e}.", file_path.to_str().unwrap());
                    return Err(EditorError::SharedMemoryCannotCreateFile);
                }
            }.unwrap();
        trace!("Created temporary file at '{}'.", file_path.to_str().unwrap());

        // Set file size by writing to the last byte
        // This is the same as [high_size 0xFFFF FFFF 0000 0000 * map_size] + [low_size 0000 0000 FFFF FFFF * map_size]
        let high_size = ((size as u64 & 0xFFFF_FFFF_0000_0000_u64) >> 32) as u32; // Highest 32 bits (mask lowest 32 bits)
        let low_size = (size as u64 & 0xFFFF_FFFF_u64) as u32; // Lowest 32 bits (mask highest 32 bits)


        // Create file view of physical file : Returns handle (= windows pointer)
        let file_mapping = match CreateFileMapping(
            HANDLE(file.as_raw_handle() as _), // Handle to physical file from which to create the view
            None, // Security attributes : None = handle cannot be inherited by child processes
            PAGE_READWRITE, // Page protection : PAGE_READWRITE = allow view read and write access to file
            high_size,
            low_size,
            tmp_id.clone() // Name of the file view object in object namespace
        ) {
            Ok(file_mapping) => Some(file_mapping),
            Err(e) => {
                error!("Unable to create file view: {e}");
                return Err(EditorError::SharedMemoryCannotCreateFileView);
            }
        }.unwrap();

        // Map the file view into the adress space of the current process : Returns pointer to first byte of mapped view
        let file_view = match MapViewOfFile(
            file_mapping.as_handle(), // Handle to view object
            FILE_MAP_READ | FILE_MAP_WRITE, // Desired access : FILE_MAP_READ | FILE_MAP_WRITE = allow mapping to be read and written
            0, // File offset high : 0 = start at the beginning of the file
            0, // File offset low : 0 = start at the beginning of the file
            0  // Number of bytes to map : 0 = map the entire file
        ) {
            Ok(mapped_view) => Some(mapped_view),
            Err(e) => {
                error!("Unable to map file mapping into adress space: {e}");
                return Err(EditorError::SharedMemoryCannotMapFileView);
            }
        }.unwrap();
        trace!("Mapped file view into adress space at location '{tmp_id}'.");
        debug!("Created shared memory of {} bytes at location '{}'.", size, tmp_id);

        // Return shared memory
        Ok(Self {
            file_path: file_path.to_str().unwrap().to_string(),
            shared_index: tmp_id,
            file_view,
            file_mapping,
        })
    }


    /// Write data to the shared memory.
    /// 
    /// # Arguments
    /// 
    /// * `data` - Data to write to the shared memory.
    pub fn write(&mut self, data: &[u8]) {
        trace!("Writing {} bytes to shared memory.", data.len());

        // Write data to shared memory
        unsafe {
            std::ptr::copy_nonoverlapping(data.as_ptr(), self.file_view.as_mut_ptr() as *mut u8, data.len());
        }
    }


    /// Get the shared memory index.
    pub fn get_index(&self) -> &str {
        &self.shared_index
    }
}

impl Drop for SharedMemory {
    fn drop(&mut self) {
        debug!("Dropping shared memory at location '{}'.", self.shared_index);

        // Delete physical file
        if let Err(e) = std::fs::remove_file(&self.file_path) {
            throw!("Unable to delete physical file at '{}' : {e}.", self.file_path);
        }
    }
}
