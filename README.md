# Kill Process - Process Management Tool

A clean and beautiful cross-platform desktop application for detecting and killing processes occupying ports.

## Features

- 🔍 **Port Detection**: Quickly detect if a specific port is occupied
- 📊 **Process Information**: Display detailed information of processes occupying ports (PID, process name)
- ⚡ **One-Click Kill**: Safely terminate processes occupying ports
- 🎨 **Modern UI**: Clean and beautiful user interface
- 🖥️ **Cross-Platform**: Support for macOS, Windows, Linux

## Tech Stack

- **Frontend**: Vue 3 + TypeScript
- **Backend**: Tauri (Rust)
- **Build**: Cross-platform desktop application

## Development Environment

### Prerequisites

- Node.js 18+
- Rust 1.70+
- pnpm (recommended) or npm

### Install Dependencies

```bash
pnpm install
```

### Development Mode

```bash
pnpm tauri dev
```

### Build Application

```bash
# Build for all platforms
pnpm tauri build

# Build for macOS only
pnpm tauri build --bundles dmg
```

## Usage

1. **Launch Application**: Run `pnpm tauri dev` or open the built application directly
2. **Enter Port**: Enter the port number to check in the input field (e.g., 3000, 8080)
3. **Check Port**: Click the "Check Port" button or press Enter
4. **View Results**: 
   - If the port is not occupied, it will show "Port Available"
   - If the port is occupied, it will display detailed information of the occupying processes
5. **Kill Process**: Click the "Kill Process" button for the corresponding process to terminate it

## Project Structure

```
kill-process/
├── src/                    # Vue frontend code
│   ├── App.vue           # Main application component
│   ├── main.ts           # Application entry point
│   └── style.css         # Global styles
├── src-tauri/            # Tauri backend code
│   ├── src/
│   │   ├── main.rs       # Application entry point
│   │   └── lib.rs        # Core functionality implementation
│   └── Cargo.toml        # Rust dependencies configuration
├── public/               # Static resources
└── package.json          # Project configuration
```

## Core Functionality Implementation

### Rust Backend Functions

- `check_port(port: String)`: Check port occupation status
- `kill_process(pid: String)`: Terminate specified process

### Vue Frontend Features

- Responsive port input and query
- Real-time process information display
- Elegant loading states and error handling
- Modern user interface

## Build and Deployment

### Development Build

```bash
pnpm tauri dev
```

### Production Build

```bash
# Build for all platforms
pnpm tauri build

# Build for specific platforms
pnpm tauri build --bundles dmg    # macOS
pnpm tauri build --bundles msi    # Windows
pnpm tauri build --bundles deb    # Linux
```

### Test

```bash
# run all test
cargo test

# run lib tests only
cargo test --test lib_tests

# run specific test
cargo test test_parse_lsof_output_deduplication

# run with debug output
cargo test -- --nocapture
```


Build artifacts will be located in the `src-tauri/target/release/bundle/` directory.

## Notes

- The application requires system permissions to execute process termination operations
- On macOS, the application uses `lsof` and `kill` commands
- It is recommended to confirm process information before terminating processes to avoid accidentally killing important processes

## License

MIT License
