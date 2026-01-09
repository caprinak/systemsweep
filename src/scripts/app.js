// systemsweep/src/scripts/app.js

const { invoke } = window.__TAURI__.tauri;

document.getElementById('quick-scan').addEventListener('click', async () => {
    const statusBar = document.getElementById('status-bar');
    statusBar.textContent = 'Scanning...';

    try {
        const scanId = await invoke('start_scan');
        statusBar.textContent = `Scan started: ${scanId}`;
    } catch (e) {
        statusBar.textContent = `Error: ${e}`;
    }
});
