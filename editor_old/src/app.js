import ipc from 'node-ipc';
import shared_memory from '@markusjx/shared_memory';

let interval = null;
function receivedMessage({ identifier, title, payload }, socket) {
    // Check identifier and title
    if (identifier == 0 && title == 0) {
        // Shared memory config
        // Read payload as string
        const path = payload.toString('utf8', 0, payload.length);
        console.log('Reading shared memory at path : ', path);
        const memory = new shared_memory(path, 4096, false, false);

        // Answer to client
        ipc.server.emit(
            socket,
            Buffer.from([0, 1, 0, 0])
        );

        // Read and convert every second
        interval = setInterval(function () {
            // Read from memory
            const buf = memory.readBuffer();
            const str = buf.toString('utf8', 0, buf.length);
            console.log('Memory : ', str);
        }, 1000);
    }



    // setInterval(function () {
    //     const identifier = 9;
    //     const title = 9;
    //     const payload = ("Hello from editor at time : " + new Date()).split('').map(function (char) {
    //         return char.charCodeAt(0);
    //     });
    //     const size = [(payload.length >> 8) & 0xff, payload.length & 0xff];

    //     // Raw message format: [channel identifier (uint 8), title (uint 8), payload size (uint 16), payload (raw bytes)]
    //     const message = Buffer.concat([
    //         Buffer.from([identifier]),
    //         Buffer.from([title]),
    //         Buffer.from([(payload.length >> 8) & 0xff, payload.length & 0xff]),
    //         Buffer.from(payload)
    //     ]);
    //     console.log('Message : ', message);

    //     // Send message to client
    //     ipc.server.emit(
    //         socket,
    //         message
    //     );
    // }, 1000);
}

function createIPCServer() {
    // IPC config
    ipc.config.retry = 1500;
    ipc.config.silent = true;
    ipc.config.rawBuffer = true;
    ipc.config.encoding = 'hex';

    // IPC start server
    ipc.serve(
        'wde\\editor',
        function () {
            // Read from client
            ipc.server.on(
                'data',
                (data, socket) => {
                    // Parse data :
                    // Raw message format: [channel identifier (uint 8), title (uint 8), payload size (uint 16), payload (raw bytes)]
                    const identifier = data[0];
                    const title = data[1];
                    const payload = data.subarray(4);
                    receivedMessage({ identifier, title, payload }, socket);
                }
            );

            // Client connected
            ipc.server.on(
                'connect',
                function (socket) {
                    console.log('LOG : client connected to editor!');
                }
            );

            // Client disconnected
            ipc.server.on(
                'socket.disconnected',
                function (socket, destroyedSocketID) {
                    console.log('LOG : client has disconnected!');

                    // Clear interval
                    clearInterval(interval);
                }
            );
        }
    );

    ipc.server.start();
}

createIPCServer();
