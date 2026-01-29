<p align="center">
  <img src="assets/hero.png" alt="Clitter" width="400">
</p>

<h1 align="center">Clitter</h1>

<p align="center">
  A modern clipboard management application for Windows, built with Tauri 2 and Svelte.
</p>

## Features

- **Clipboard History** - Automatically saves text and images you copy
- **Category Filtering** - Filter by text, images, numbers, or secure content
- **Whiteboard Mode** - Organize frequently used items with drag-and-drop
- **Groups** - Create hierarchical folders to organize whiteboard items
- **Keyboard Shortcuts** - Quick access with customizable shortcuts
- **Secure Storage** - Sensitive content (passwords, API keys) detected and encrypted
- **Global Hotkey** - Press `Alt+V` to open from anywhere

## Screenshots

### List View
Quick access to recent clipboard history with category filtering.

### Whiteboard View
Organize permanent items into groups with custom shortcuts.

## Installation

### From Release
Download the latest installer from [Releases](../../releases).

### Build from Source

#### Prerequisites
- [Node.js](https://nodejs.org/) (LTS)
- [Rust](https://rustup.rs/)
- Visual Studio Build Tools with "Desktop development with C++"

#### Steps
```bash
# Clone the repository
git clone https://github.com/FatmanBros/clitter.git
cd clitter

# Install dependencies
npm install

# Run in development mode
npm run tauri:dev

# Build for production
npm run tauri:build
```

Build outputs:
- `src-tauri/target/release/clitter.exe` - Standalone executable
- `src-tauri/target/release/bundle/nsis/` - NSIS installer (recommended)
- `src-tauri/target/release/bundle/msi/` - MSI installer

## Usage

### Keyboard Shortcuts

#### Global
| Key | Action |
|-----|--------|
| `Alt+V` | Open Clitter |

#### List View
| Key | Action |
|-----|--------|
| `1-5` | Paste item by number |
| `↑` or `W` | Switch to Whiteboard |
| `←` | Filter: Images |
| `↓` | Filter: Text |
| `→` | Filter: Numbers |
| `A` | Filter: All |
| `S` | Filter: Secure |
| `Esc` | Hide window |

#### Whiteboard View
| Key | Action |
|-----|--------|
| `↓` or `L` | Switch to List |
| `←` | Exit current group |
| `→` | Re-enter last group |
| `Enter` | Select matched item/group |
| `/` | Enter group (after typing name) |
| `Esc` | Clear input / Exit group / Switch to List |
| `A-Z, 0-9` | Type to search shortcuts |

### Context Menu (Right-click)
- **Set Shortcut** - Assign a keyboard shortcut
- **Set Value** - Create a key-value pair (label + value)
- **Set Color** - Change group color
- **Create Group** - Create a new folder
- **Import/Export JSON** - Backup and restore whiteboard data

## Tech Stack

- **Frontend**: Svelte 5 + TypeScript + Vite
- **Backend**: Tauri 2 (Rust)
- **Database**: SQLite (sqlx)
- **Encryption**: AES-GCM + Argon2

## Project Structure

```
clitter/
├── src/                 # Frontend (Svelte)
│   ├── App.svelte
│   ├── lib/
│   │   ├── components/  # UI components
│   │   ├── stores/      # Svelte stores
│   │   └── types/       # TypeScript types
│   └── styles/
├── src-tauri/           # Backend (Rust)
│   ├── src/
│   │   ├── clipboard/   # Clipboard monitoring
│   │   ├── crypto/      # Encryption
│   │   ├── storage/     # Database
│   │   └── commands.rs  # Tauri commands
│   └── tauri.conf.json
└── package.json
```

## License

[MIT License](LICENSE)

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.
