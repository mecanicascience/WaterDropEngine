import './App.css'
import { createIPCServer } from './lib/Ipc'

// Start IPC communication
createIPCServer();

// Start the app
export default function App() {
    return (
        <>
            <p>Start here</p>
        </>
    )
}
