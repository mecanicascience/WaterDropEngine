use tokio::{runtime::Builder, net::windows::named_pipe, io::Interest};
use wde_logger::{trace, warn, info, throw};

/// Define an IPC message.
/// Raw message format: [channel identifier (uint 8), title (uint 8), payload size (uint 16), payload (raw bytes)]
#[derive(Debug, Clone)]
pub struct IPCMessage {
    /// The channel identifier of the IPC message.
    pub channel: u8,
    /// The title of the IPC message.
    pub title: u8,
    /// The payload size of the IPC message.
    pub payload_size: u16,
    /// The payload of the IPC message as raw bytes.
    pub payload: Vec<u8>
}

/// An IPC channel.
/// 
/// To use this IPC channel, you need to create a server that will send messages to this channel.
/// 
/// # Example
/// 
/// ```
/// let mut ipc = IPC::new(ipc_channel_name, IPCAccess::Read | Write, ipc_buffer_size);
/// match ipc.read() { // Read the last unread messages
///     Some(messages) => {
///         for message in messages {
///             // Handle message
///         }
///     },
///     None => {}
/// }
/// ```
pub struct IPC {
    /// The name of the IPC channel.
    name: String,
    /// The runtime for the IPC channel that receives messages.
    #[allow(dead_code)]
    runtime: tokio::runtime::Runtime,
    /// The shared list of received messages to read.
    shared_messages_read: std::sync::Arc<std::sync::Mutex<Vec<IPCMessage>>>,
    /// The shared list of received messages to write.
    shared_messages_write: std::sync::Arc<std::sync::Mutex<Vec<IPCMessage>>>
}

impl IPC {
    /// Creates a new IPC channel.
    /// 
    /// # Arguments
    /// 
    /// * `name` - The name of the IPC channel.
    /// * `allocated_size` - The allocated size for the IPC channel.
    pub fn new(name: String, allocated_size: usize) -> Self {
        trace!("Creating an IPC channel with name '{}'.", name);

        // Create runtime
        let runtime = Builder::new_multi_thread().enable_all().build().unwrap();

        // Shared list of received messages
        let shared_messages_read = std::sync::Arc::new(
            std::sync::Mutex::new(Vec::<IPCMessage>::new())
        );
        let messages_read = shared_messages_read.clone();
        let shared_messages_write = std::sync::Arc::new(
            std::sync::Mutex::new(Vec::<IPCMessage>::new())
        );
        let messages_write = shared_messages_write.clone();
        let n = name.clone();

        // Spawn the root task
        runtime.spawn(async move {
            // Create pipe name
            let pipe_name: &str = &(r"\\.\pipe\wde\".to_owned() + &name);

            // Connect to pipe
            let client = match named_pipe::ClientOptions::new().open(pipe_name)  {
                Ok(client) => client,
                Err(e) if e.kind() == tokio::io::ErrorKind::NotFound => {
                    throw!("Server not found for IPC with name '{}'. Try starting the server first.", name);
                }
                Err(e) if e.raw_os_error() == Some(231) => { // Os error 231 (All pipe instances are busy)
                    throw!("Server busy for IPC with name '{}'.", name);
                }
                Err(e) => {
                    throw!("The pipe connection encountered an error for IPC with name '{}': {}", name, e);
                }
            };

            // Log ready
            info!("An IPC channel with name '{}' has been created.", name);
            loop {
                // Wait for the socket to be readable
                let ready = match client.ready(Interest::READABLE | Interest::WRITABLE).await {
                    Ok(ready) => ready,
                    Err(e) if e.kind() == tokio::io::ErrorKind::WouldBlock => {
                        warn!("Server not ready for IPC with name '{}'.", name);
                        return;
                    }
                    Err(e) => {
                        throw!("The pipe connection encountered an error for IPC with name '{}': {}", name, e);
                    }
                };

                // Check if server closed
                if ready.is_read_closed() || ready.is_write_closed() || ready.is_error() {
                    break;
                }

                // Check if the socket is ready to be read from or written to.
                if ready.is_readable() {
                    let mut data = vec![0; allocated_size];

                    // Try to read data
                    match client.try_read(&mut data) {
                        Ok(n) => {
                            // If n is 0, the other side closed the socket, so we'll just terminate
                            if n == 0 {
                                break;
                            }

                            // Read data (n bytes)
                            let data = data[..n].to_vec();

                            // Parse data
                            let mut index = 0;
                            let mut received_messages = Vec::<IPCMessage>::new();
                            while index < data.len() {
                                // Get channel identifier
                                let channel = data[index..index + 1].to_vec();
                                index += 1;

                                // Get title
                                let title = data[index..index + 1].to_vec();
                                index += 1;

                                // Get payload size
                                let payload_size = ((data[index] as u16) << 8) | (data[index + 1] as u16);
                                index += 2;

                                // Check if payload size is valid
                                if payload_size > allocated_size as u16 {
                                    throw!("Payload size '{}' is bigger than the allocated size '{}'.", payload_size, allocated_size);
                                }

                                // Get payload
                                let payload = data[index..index + payload_size as usize].to_vec();
                                index += payload_size as usize;

                                // Create message
                                let message = IPCMessage {
                                    channel: channel[0],
                                    title: title[0],
                                    payload_size,
                                    payload
                                };

                                // Push message
                                received_messages.push(message);
                            }
                            
                            // Push messages
                            let mut messages = messages_read.lock().unwrap();
                            messages.append(&mut received_messages);
                        }
                        Err(e) if e.kind() == tokio::io::ErrorKind::WouldBlock => {
                            // Hope that the other side will write something soon
                            continue;
                        }
                        Err(_) => {
                            // An error occurred, this might be because the other side closed the socket, so we'll just terminate
                            break;
                        }
                    }
                }

                // Check if the socket is ready to be written to.
                if ready.is_writable() {
                    // Lock messages
                    let mut messages = messages_write.lock().unwrap();

                    // Check if there are messages to send
                    if messages.len() > 0 {
                        // Create data
                        let mut data = Vec::<u8>::new();
                        for message in messages.iter() {
                            // Push channel identifier
                            data.push(message.channel);

                            // Push title
                            data.push(message.title);

                            // Push payload size
                            data.push((message.payload_size >> 8) as u8);
                            data.push((message.payload_size & 0xFF) as u8);

                            // Push payload
                            data.append(&mut message.payload.clone());
                        }

                        // Send data
                        match client.try_write(&data) {
                            Ok(n) => {
                                // If n is 0, the other side closed the socket, so we'll just terminate
                                if n == 0 {
                                    trace!("Server closed IPC channel named '{}'.", name);
                                    break;
                                }

                                // Clear messages
                                messages.clear();
                            }
                            Err(e) if e.kind() == tokio::io::ErrorKind::WouldBlock => {
                                // Hope that the other side will read something soon
                                continue;
                            }
                            Err(_) => {
                                // An error occurred, this might be because the other side closed the socket, so we'll just terminate
                                break;
                            }
                        }
                    }
                }
            }

            // Server stopped
            info!("IPC channel named '{}' closed.", name);
        });

        IPC {
            name: n,
            runtime,
            shared_messages_read,
            shared_messages_write
        }
    }


    /// Reads the messages from the IPC channel.
    /// Clears the messages after reading them.
    pub fn read(&mut self) -> Option<Vec<IPCMessage>> {
        // Lock messages
        let mut messages = self.shared_messages_read.lock().unwrap();

        // Return messages
        if messages.len() > 0 {
            let mut new_messages = Vec::<IPCMessage>::new();
            std::mem::swap(&mut new_messages, &mut *messages);
            messages.clear();
            Some(new_messages)
        } else {
            None
        }
    }

    /// Writes a message to the IPC channel.
    /// 
    /// # Arguments
    /// 
    /// * `message` - The message to write.
    pub fn write(&mut self, message: IPCMessage) {
        // Lock messages
        let mut messages = self.shared_messages_write.lock().unwrap();

        // Push message
        messages.push(message);
    }
}

impl Drop for IPC {
    fn drop(&mut self) {
        trace!("Dropping IPC channel with name '{}'.", self.name);
    }
}

