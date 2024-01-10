use wde_logger::{trace, info, warn};

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
    pub fn new() -> Self {
        info!("Starting editor handler.");

        // Create IPC write and read channels
        let mut ipc = IPC::new("editor".to_string(), 1_024);

        // Create shared memory
        let shared_memory = SharedMemory::new(4096);

        // Wait for editor to connect
        let ipc_running;
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
                }
            }
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
        if !ipc_running {
            return EditorHandler {
                ipc,
                shared_memory, 
                status: ipc_running
            };
        }

        // Send shared memory index to editor
        let identifier = shared_memory.get_index().to_string().as_bytes().to_vec();
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
                Some(messages) => {
                    for message in messages {
                        if message.channel == EditorChannels::SharedMemoryIndex as u8 && message.title == 1 {
                            received = true;
                            break;
                        }
                    }
                },
                None => {}
            }
            if received {
                info!("Received confirmation from editor.");
                break;
            }
            std::thread::sleep(std::time::Duration::from_millis(100));
        }

        // Return object
        EditorHandler {
            ipc,
            shared_memory,
            status: ipc_running
        }
    }

    /// Process messages from editor stored in the cache.
    /// Must be called every frame.
    pub fn process(&mut self) {
        // Check if editor is running
        if !self.status {
            warn!("Editor is not running.");
        }

        trace!("Processing messages from editor.");
        match self.ipc.read() {
            Some(messages) => {
                for message in messages {
                    match message.channel {
                        _ => {} // TODO
                    }
                }
            },
            None => {}
        }
    }

    /// Write to shared memory the current frame.
    /// 
    /// # Arguments
    /// 
    /// * `data` - Data to write to shared memory
    pub fn set_current_frame(&mut self, data: &[u8]) {
        // Check if editor is running
        if !self.status {
            warn!("Editor is not running.");
        }

        trace!("Writing current frame to shared memory.");
        self.shared_memory.write(&data);
    }

    /// Check if editor handler started correctly.
    pub fn started(&self) -> bool {
        self.status
    }
}