use wde_logger::{trace, debug, error};

use crate::EditorError;
use crate::ipc::{IPC, IPCMessage, IPCChannelStatus};
use crate::shared_memory::SharedMemory;

enum EditorChannels {
    /// First channel = communicate shared memory index to editor.
    /// This also tells the editor that the game restarted.
    /// Title = 0 (send index) / 1 (confirmed index received)
    SharedMemoryIndex = 0
}

/// Handler for editor interactions.
/// 
/// # Example
/// 
/// ```
/// let mut editor_handler = EditorHandler::new();
/// 
/// if !editor_handler.started() {
///    panic!("Editor handler failed to start.");
/// }
/// 
/// loop {
///    // Process editor messages
///    editor_handler.process();
/// 
///    // Set last frame
///    editor_handler.set_current_frame(last_frame);
/// }
/// ```
pub struct EditorHandler {
    ipc: IPC,
    shared_memory: SharedMemory,
    status: bool // True if ok, false if not ok
}

impl EditorHandler {
    /// Create a new editor handler.
    /// 
    /// # Errors
    /// 
    /// * `EditorError::SharedMemoryFailed` - Shared memory failed.
    pub fn new() -> Result<Self, EditorError> {
        debug!("Starting editor handler.");

        // Create IPC write and read channels
        let mut ipc = IPC::new("editor".to_string(), 1_024);

        // Create shared memory
        let shared_memory = SharedMemory::new(4096);

        // Wait for editor to connect
        let ipc_running;
        let mut iterations = 200; // Stop after this amount of iterations
        loop {
            match ipc.status() {
                IPCChannelStatus::Running => {
                    ipc_running = true;
                    break;
                },
                IPCChannelStatus::NotRunning => {
                    ipc_running = false;
                    break;
                },
                IPCChannelStatus::Starting => {
                    // Wait for editor to connect
                    iterations -= 1;

                    // Check if iterations are over
                    if iterations == 0 {
                        error!("Editor took too long to start.");
                        return Err(EditorError::EditorNotRunning);
                    }
                }
            }
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
        if shared_memory.is_err() {
            return Err(EditorError::SharedMemoryFailed);
        }
        if !ipc_running {
            return Ok(EditorHandler {
                ipc,
                shared_memory: shared_memory.unwrap(),
                status: ipc_running
            });
        }

        // Send shared memory index to editor
        let identifier = shared_memory.as_ref().unwrap().get_index().to_string().as_bytes().to_vec();
        trace!("Sending shared memory index to editor: '{}'.", std::str::from_utf8(&identifier).unwrap());
        ipc.write(IPCMessage {
            channel: EditorChannels::SharedMemoryIndex as u8,
            title: 0,
            payload_size: identifier.len() as u16,
            payload: identifier
        });

        // Wait for confirmation from editor
        loop {
            let mut received = false;
            match ipc.read() {
                Ok(messages) => match messages {
                    Some(messages) => {
                        for message in messages {
                            if message.channel == EditorChannels::SharedMemoryIndex as u8 && message.title == 1 {
                                received = true;
                                break;
                            }
                        }
                    },
                    None => {}
                },
                Err(e) => {
                    return Err(e);
                }
            }
            if received {
                trace!("Received confirmation from editor.");
                break;
            }
            std::thread::sleep(std::time::Duration::from_millis(100));
        }

        // Return object
        Ok(EditorHandler {
            ipc,
            shared_memory: shared_memory.unwrap(),
            status: ipc_running
        })
    }

    /// Process messages from editor stored in the cache.
    /// Must be called every frame.
    /// 
    /// # Errors
    /// 
    /// * `EditorError::EditorNotRunning` - The editor is not running.
    pub fn process(&mut self) -> Result<(), EditorError> {
        // Check if editor is running
        if !self.status {
            error!("Editor is not running.");
            return Err(EditorError::EditorNotRunning);
        }

        trace!("Processing messages from editor.");
        match self.ipc.read() {
            Ok(messages) => match messages {
                Some(messages) => {
                    for message in messages {
                        match message.channel {
                            _ => {} // TODO
                        }
                    }
                },
                None => {}
            },
            Err(e) => {
                return Err(e);
            }
        }
        Ok(())
    }

    /// Write to shared memory the current frame.
    /// 
    /// # Arguments
    /// 
    /// * `data` - Data to write to shared memory
    /// 
    /// # Errors
    /// 
    /// * `EditorError::EditorNotRunning` - The editor is not running.
    pub fn set_current_frame(&mut self, data: &[u8]) -> Result<(), EditorError> {
        // Check if editor is running
        if !self.status {
            error!("Editor is not running.");
            return Err(EditorError::EditorNotRunning);
        }

        trace!("Writing current frame to shared memory.");
        self.shared_memory.write(&data);
        Ok(())
    }

    /// Check if editor handler started correctly.
    pub fn started(&self) -> bool {
        self.status
    }
}