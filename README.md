# SystemSweep

SystemSweep is a modern, high-performance cross-platform desktop cleaner built with Tauri 2.0, Rust, and React. It provides a robust set of tools to scan, categorize, and safely clean up system clutter while ensuring data safety through restore points and secure deletion.

## Features

- **Hybrid Scanning Engine**: Combines a high-performance parallel file traverser with a sophisticated rule-based categorization engine.
- **Smart Categorization**: Automatically identifies temporary files, caches, logs, thumbnails, and more.
- **Duplicate Detection**: A multi-phase hashing system (Size -> Quick Hash -> Full Hash) to accurately find duplicate files.
- **Large File Finder**: Quickly identify space-consuming files with configurable size thresholds.
- **Platform-Specific Optimization**:
  - **Startup Manager**: Manage startup applications on Windows (Registry) and Linux (.desktop files).
  - **System Metrics**: Real-time monitoring of CPU, Memory, and Disk usage.
- **Safe & Secure Deletion**:
  - **Trash Integration**: Safely move files to the system trash.
  - **Restore Points**: Automatically back up files to a local SQLite-tracked storage before deletion for easy restoration.
  - **Secure Delete**: Industrial-grade 3-pass file overwriting (Zero-pass, One-pass, Random-pass).
- **Modern UI**: A sleek, responsive dashboard built with React and Vite.

## Tech Stack

- **Backend**: Rust, Tauri 2.0, Tokio (async), Rayon (parallelism), rusqlite (SQLite), sysinfo.
- **Frontend**: React, TypeScript, Vite, TailwindCSS (optional).
- **Architecture**: Modular domain-driven design (Scanner, Cleaner, System, Startup).

## Getting Started

### Prerequisites

- [Rust](https://www.rust-lang.org/tools/install)
- [Node.js](https://nodejs.org/)
- [Tauri CLI](https://tauri.app/v1/guides/getting-started/prerequisites)

### Installation

1. Clone the repository:
   ```bash
   git clone <repository-url>
   cd systemsweep
   ```

2. Install dependencies:
   ```bash
   npm install
   ```

3. Run in development mode:
   ```bash
   npm run tauri dev
   ```

## Development

### Backend Structure
- `src-tauri/src/scanner`: File traversal and categorization.
- `src-tauri/src/cleanup`: Safe deletion, secure deletion, and restoration.
- `src-tauri/src/startup`: Platform-specific startup management.
- `src-tauri/src/system`: Hardware and OS metrics provider.
- `src-tauri/src/commands`: Tauri command bridge for the frontend.

### Frontend
- Located in `src/`. Uses `useBackend` hook for interacting with the Rust core.

## License

MIT License - See [LICENSE](LICENSE) for details.
