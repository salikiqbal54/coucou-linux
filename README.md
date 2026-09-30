<div align="center">

<img src="src-tauri/icons/128x128.png" width="96" alt="Coucou icon">

# Coucou for Linux

**A small desktop island for Claude Code, Mochi, and your connected services.**

Coucou lives at the top of your screen and gives you a lightweight way to watch Claude Code sessions, respond to permission requests, chat with Claude, drop files, and keep an eye on connected services without leaving what you're doing.

![Linux](https://img.shields.io/badge/Linux-X11%20%2F%20XWayland-FCC624?logo=linux\&logoColor=black)
![Tauri 2](https://img.shields.io/badge/Tauri-2-FFC131?logo=tauri\&logoColor=black)
![Rust](https://img.shields.io/badge/Rust-backend-000?logo=rust)
![TypeScript](https://img.shields.io/badge/TypeScript-frontend-3178C6?logo=typescript\&logoColor=white)
![License: MIT](https://img.shields.io/badge/license-MIT-green)

</div>

<img src="screenshots/greeting.png" width="640" alt="Mochi waving hello at launch">

---

## Status

This repository is the **Linux port of Coucou**.

The current implementation is working on **X11** and on **Wayland sessions through XWayland**.

Native Wayland window management is not supported yet. Coucou currently relies on X11/XWayland for the top-center positioning and always-on-top behavior required by the desktop island.

Linux packaging and a finished installer are still work in progress.

## What is Coucou?

Coucou is a desktop companion built around **Mochi**, a small animated character that lives in a compact top-center island.

Instead of opening another large application window every time something happens, Coucou keeps useful information in a small overlay that expands when you need it.

It can:

* Show Claude Code session activity
* Display Claude Code permission requests
* Let you allow or deny requests
* Provide a compact chat interface for Claude
* Accept files dragged onto the island
* Display integration updates
* Open project folders in VS Code
* Provide a system tray menu
* Automatically collapse when inactive
* Wake again from its small top-edge strip

## Using it

<img src="screenshots/compact.png" width="292" alt="The compact island, with integration pills as mini Mochis">
<img src="screenshots/overview.png" width="640" alt="The overview with the focused integration and other service pills">
<img src="screenshots/approval.png" width="640" alt="A Claude Code permission request with Deny and Allow">
<img src="screenshots/chat.png" width="640" alt="Chatting with Claude from the island">
<img src="screenshots/drop.png" width="640" alt="Mochi waiting for a dropped file">

| What you do                           | What happens                                         |
| ------------------------------------- | ---------------------------------------------------- |
| Move the mouse to the top-center area | Mochi wakes the island                               |
| Click the island                      | It expands                                           |
| Click Mochi                           | Mochi reacts                                         |
| Hover over Mochi                      | Interactive reactions can appear                     |
| Drag a file onto the island           | The file is received and can be used as chat context |
| `Esc`                                 | Closes the expanded island                           |
| Tray icon                             | Open, Settings, Pause, or Quit                       |

Claude Code events can also open the island automatically when something requires attention.

## Claude Code

<img src="screenshots/settings.png" width="562" alt="Coucou Settings window">

Coucou integrates with Claude Code through its hook system.

Open **Settings → Claude Code → Install hooks** to review the changes before they are written to your Claude Code configuration.

The Linux hook relay communicates with Coucou through a Unix domain socket:

```text
$XDG_RUNTIME_DIR/coucou.sock
```

A temporary `/tmp` location is used when the runtime directory is unavailable.

The relay is designed to fail gracefully if Coucou is unavailable, so Claude Code can continue working normally.

Coucou can receive Claude Code events including session activity, tool usage, notifications, permission requests, and completion events.

For permission requests, Coucou can present **Allow** and **Deny** controls directly in the island.

The Claude Code configuration is normally located at:

```text
~/.claude/settings.json
```

Coucou previews hook changes before applying them and aims to modify only its own entries.

## Chat and credentials

Coucou includes a compact Claude chat interface.

The Anthropic API key is handled by the Rust backend rather than being exposed directly to the frontend.

Credentials for supported services are stored through the system keyring.

Current credential integrations include:

* Anthropic
* GitHub
* Vercel
* Stripe
* Resend
* Notion
* Cal.com
* n8n

No API keys are included in this repository.

## Integrations

Coucou currently includes integration support for:

* GitHub
* Vercel
* Stripe
* Resend
* Notion
* Cal.com
* n8n

Integration cards can appear alongside Mochi and can be configured through Settings.

## File drop

Files can be dragged onto the island.

Coucou receives the dropped file, copies it into its local inbox, and can use it as context for a Claude chat interaction.

A development upload preview is also included:

```text
dev/upload-preview.html
```

This makes it possible to work on the file-drop animation without performing a real desktop drag for every test.

## Settings

The Settings window provides controls for:

* Claude configuration
* Integration credentials
* Enabled integrations
* Sound
* Volume
* Automatic closing
* Autostart
* Claude Code hooks

## Linux desktop behavior

The main island window is a transparent, borderless desktop overlay positioned at the top-center of the primary display.

The current Linux implementation uses X11/XWayland because the application needs desktop window behavior that the current native Wayland implementation does not provide.

When running from the development environment, the project automatically starts Tauri with:

```text
GDK_BACKEND=x11
```

This allows Coucou to work from a normal Wayland desktop session through XWayland.

Native Wayland support is planned for a future stage of the port.

## Build and run

### Requirements

You need:

* Linux
* X11 or XWayland
* Rust
* Node.js 20+
* The Linux development libraries required by Tauri/WebKitGTK

For Claude Code integration, Claude Code should also be installed and configured.

Install JavaScript dependencies:

```bash
npm install
```

Run the desktop application:

```bash
npm run tauri dev
```

Build the frontend:

```bash
npm run build
```

Run the frontend by itself:

```bash
npm run dev
```

The browser-only development mode is useful for working on the island UI without starting the Tauri desktop shell.

## Project structure

```text
.
├── src/
│   ├── core/             Shared state, layout, bridge and sound
│   ├── island/           Island UI, input and state machine
│   ├── mochi/            Mochi rendering and animation
│   ├── views/            Island views
│   ├── settings/         Settings frontend
│   └── upload/           File-drop animation
│
├── src-tauri/
│   ├── src/
│   │   ├── claude.rs     Claude API/chat logic
│   │   ├── hooks.rs      Claude Code hook management
│   │   ├── integrations.rs
│   │   ├── island.rs     Native desktop window behavior
│   │   ├── pipe.rs       Unix-socket communication
│   │   ├── secrets.rs    System keyring credentials
│   │   └── ...
│   └── tauri.conf.json
│
├── hook/                 Claude Code hook relay
├── screenshots/          UI screenshots
├── scripts/              Build helpers
├── index.html            Main island entry point
└── settings.html         Settings entry point
```

## Technology

Coucou uses:

* **Tauri 2** for the desktop shell
* **Rust** for the backend
* **TypeScript** for the frontend
* **Vite** for the frontend build
* **Canvas 2D** for Mochi and visual animations
* **Tokio** for asynchronous IPC
* **Reqwest** for network requests
* **System keyring support** for credentials

The frontend is implemented directly in TypeScript without a large UI framework.

Rust handles the platform-facing parts of the application, including Claude communication, integrations, secrets, hooks, IPC, settings, and the system tray.

## Development notes

Generated files are intentionally excluded from the repository.

These include:

```text
node_modules/
dist/
target/
```

Local secrets and application data are also excluded.

The repository contains several files retained from the original platform implementation while the Linux port is being cleaned up. Files with names such as `.windows-backup` exist as development references and are not part of the Linux runtime.

## Current limitations

The Linux port is still under development.

Current limitations include:

* Native Wayland window management is not implemented yet.
* X11/XWayland is currently required for the desktop island behavior.
* The island currently targets the primary display.
* A finished Linux installer/package has not been published yet.
* Some platform-specific legacy source files remain in the repository while the port is being cleaned up.
* More testing is needed across different Linux distributions and desktop environments.

## Roadmap

The Linux port is being developed incrementally.

Planned work includes:

* Native Wayland window support
* Better multi-monitor behavior
* Linux packaging and distribution
* Further cleanup of legacy platform-specific code
* Testing across more Linux desktop environments

## About the project

Coucou was originally designed around a macOS-style desktop island concept.

The Linux version keeps the same core idea while replacing platform-specific components with Linux equivalents where practical:

**a small animated assistant that stays at the top of your screen instead of becoming another large application window.**

## License

MIT
