mod ipc;
mod shared_memory;
mod editor_handler;

pub use editor_handler::*;

/// Define main errors
#[derive(Debug)]
pub enum EditorError {
    /// Editor is not running
    EditorNotRunning,
    /// IPC server is not started
    IPCServerNotStarted,
    /// IPC server failed to start
    IPCServerFailed,
    /// Shared memory already exists
    SharedMemoryAlreadyExists,
    /// Shared memory failed
    SharedMemoryFailed,
    /// Shared memory cannot create directory
    SharedMemoryCannotCreateDir,
    /// Shared memory cannot create file
    SharedMemoryCannotCreateFile,
    /// Shared memory cannot create file view
    SharedMemoryCannotCreateFileView,
    /// Shared memory cannot map file view
    SharedMemoryCannotMapFileView
}
