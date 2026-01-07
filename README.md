<p align="center">
  <img src="src-tauri/icons/128x128.png" alt="Yomiyougu Logo" />
</p>

<h1 align="center">Yomiyougu</h1>

<p align="center">
  A modern, cross-platform manga and comic reader built with Tauri and SvelteKit.
</p>

---

## About

**Yomiyougu** (読み用具 - "reading tool") is an open-source manga/comic reader designed for desktop and Android. It supports popular formats like CBZ/ZIP, CBR/RAR, with features like cloud sync via Google Drive, customizable reading modes, and a clean, modern interface.

This project was created by a university student as part of an academic project.

## Features

- 📚 Support for CBZ/ZIP and CBR/RAR formats
- ☁️ Optional cloud sync with Google Drive
- 🎨 Custom themes and reading modes
- 📱 Cross-platform: Linux desktop and Android
- 🔖 Bookmarks and reading progress tracking
- 📂 Library organization with collections

## Prerequisites

Before building, make sure you have the required dependencies installed. Follow the official Tauri setup guide for your platform:

- **[Tauri Prerequisites](https://tauri.app/start/prerequisites/)**

## Development

```bash
# Install dependencies
pnpm install

# Run in development mode
pnpm tauri dev

# Run in development mode for Android
pnpm tauri android dev

# Build for production
pnpm tauri build
```

## Recommended IDE Setup

[VS Code](https://code.visualstudio.com/) + [Svelte](https://marketplace.visualstudio.com/items?itemName=svelte.svelte-vscode) + [Tauri](https://marketplace.visualstudio.com/items?itemName=tauri-apps.tauri-vscode) + [rust-analyzer](https://marketplace.visualstudio.com/items?itemName=rust-lang.rust-analyzer)

## License

This project is open source. See [LICENSE](LICENSE) for details.
