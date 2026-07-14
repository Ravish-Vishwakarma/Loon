<h1 align="center">Loon</h1>

<p align="center">
  <img src="Resources/Loon.png" alt="Loon" width="300">
</p>

<p align="center">
  A fast, local speech-to-text transcription app for Windows. Press a shortcut, speak, get text.
</p>

---

## Features

- **Toggle recording** — Press a global shortcut to start recording, press again to stop
- **Local transcription** — Powered by [whisper.cpp](https://github.com/ggerganov/whisper.cpp), everything runs on your machine. No cloud, no API keys
- **AI polish** — Optionally clean up transcriptions with a local [Ollama](https://ollama.com/) LLM (spelling, grammar, filler words)
- **Configurable AI prompt** — Customize the system prompt to control how the AI polishes your transcriptions
- **Multiple Whisper models** — Choose from Tiny to Large V3 Turbo depending on your speed/accuracy needs
- **Clipboard & paste** — Copy to clipboard, paste at cursor, or both
- **Floating launcher** — A tiny always-on-top pill that shows recording/transcription status
- **System tray** — Runs quietly in the tray with quick access to settings
- **Transcription history** — Browse and manage all past transcriptions
- **Built-in API server** — The runtime exposes a local HTTP API (`localhost:15000`) that other apps can use for transcription and model management

## How It Works

1. Press **Ctrl+Shift+Space** (configurable) to start recording
2. Press the same shortcut again to stop recording
3. Loon sends the audio to a local runtime server, which runs whisper.cpp
4. The transcribed text is copied to your clipboard (or pasted at your cursor)
5. Optionally, Ollama polishes the text for better readability

## Tech Stack

| Layer | Technology |
|---|---|
| Desktop framework | [Tauri v2](https://tauri.app/) (Rust + WebView) |
| Frontend | React 19 + TypeScript + Vite + Tailwind CSS |
| Transcription | [whisper.cpp](https://github.com/ggerganov/whisper.cpp) (bundled CLI) |
| AI polish | [Ollama](https://ollama.com/) (local LLM, optional) |
| Audio recording | [cpal](https://github.com/RustAudio/cpal) |
| Database | SQLite via rusqlite |
| Clipboard | arboard + enigo (simulate paste) |

## Requirements

- **Windows 10/11**
- **[Ollama](https://ollama.com/)** (only if you want AI polish — not required for basic transcription)

## Getting Started

### Download

Download the latest release from the [Releases](https://github.com/Ravish-Vishwakarma/loon/releases) page.

### Build from Source

Prerequisites:
- [Node.js](https://nodejs.org/) (v18+)
- [Rust](https://www.rust-lang.org/tools/install) (stable)
- [Tauri CLI](https://v2.tauri.app/start/prerequisites/)

```bash
# Clone the repo
git clone https://github.com/Ravish-Vishwakarma/loon.git
cd loon

# Install frontend dependencies
npm install

# Run in dev mode
npm run tauri dev

# Build for production
npm run tauri build
```

## Built-in API

Loon runs a local HTTP server on `http://localhost:15000` that other apps can use to transcribe audio or manage models.

| Endpoint | Method | Description |
|---|---|---|
| `/v1/health` | GET | Health check |
| `/v1/transcribe` | POST | Transcribe audio (multipart form: `file` + `model_id`) |
| `/v1/models/available` | GET | List all available models |
| `/v1/models/downloaded` | GET | List downloaded models |
| `/v1/models/download` | POST | Download a model (`{ "model_id": "..." }`) |
| `/v1/models/{model_id}` | DELETE | Delete a model |

Example — transcribe a WAV file with curl:

```bash
curl -X POST http://localhost:15000/v1/transcribe \
  -F "file=@recording.wav" \
  -F "model_id=whisper-base"
```

## Configuration

Open Settings by right-clicking the system tray icon → **Setting**.

| Setting | Description |
|---|---|
| **Keyboard Shortcut** | Global shortcut to start/stop recording (default: `Ctrl+Shift+Space`) |
| **Output Mode** | Copy to clipboard, paste at cursor, or both |
| **Active Model** | Whisper model used for transcription — download and select one |
| **AI Model** | Ollama model for polishing (requires Ollama running) |
| **Auto Polish** | Automatically polish transcriptions after recording |
| **Polish Prompt** | Custom prompt template for the AI polish step |

## Whisper Models

| Model | Size | Speed | Accuracy |
|---|---|---|---|
| whisper-tiny | 75 MB | Fastest | Basic |
| whisper-base | 142 MB | Fast | Good |
| whisper-small | 466 MB | Medium | Better |
| whisper-medium | 1.5 GB | Slow | Great |
| whisper-large-v3 | 3.1 GB | Slowest | Best |
| whisper-large-v3-turbo | 1.5 GB | Medium | Best (optimized) |

Models are downloaded on-demand from [Hugging Face](https://huggingface.co/ggerganov/whisper.cpp) and stored in the app data directory.

## Project Structure

```
loon/
├── src/                    # Frontend (React + TypeScript)
│   ├── pages/
│   │   ├── launcher.tsx    # Floating pill launcher UI
│   │   ├── homepage.tsx    # Transcription history
│   │   └── settings.tsx    # Settings page
│   └── components/         # Shared UI components
├── src-tauri/              # Tauri backend (Rust)
│   └── src/
│       ├── lib.rs          # App setup, tray, window management
│       ├── shortcut.rs     # Global shortcut + transcription flow
│       ├── recorder.rs     # Audio recording + WAV processing
│       ├── config.rs       # Config load/save
│       ├── db.rs           # SQLite database
│       ├── ollama.rs       # Ollama AI polish client
│       ├── clipboard.rs    # Clipboard + simulated paste
│       └── proxy.rs        # IPC proxy for runtime server
├── runtime/                # Local transcription server (Rust/Axum)
│   └── src/
│       ├── api/            # HTTP endpoints (transcribe, models, health)
│       └── backends/       # whisper.cpp CLI backend
└── whisper/                # Bundled whisper.cpp binaries
```

## License

MIT
