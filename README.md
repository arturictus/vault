
# Secret Vault - Encrypted Local Secret Storage

![Tauri](https://img.shields.io/badge/Tauri-2.0.0-blue?logo=tauri)
![License](https://img.shields.io/badge/License-MIT-green)

Secret Vault is a secure, lightweight desktop application built using the **Tauri framework** that allows you to store and manage your sensitive information locally. With a focus on **data ownership** and **security**, all secrets are encrypted using robust encryption methods before being stored on your device. Your data never leaves your machine, ensuring complete privacy and control.

## Features

- **Local Storage**: All secrets are stored locally on your device, ensuring that your data never leaves your machine.
- **Strong Encryption**: Secrets are encrypted using industry-standard encryption methods before being saved to disk.
- **Data Ownership**: You retain full ownership of your data. No third-party servers or cloud storage are involved.
- **Cross-Platform**: Built with Tauri, the application runs seamlessly on Windows, macOS, and Linux.
- **Simple & Intuitive UI**: A clean and user-friendly interface makes it easy to manage your secrets.

## How It Works

1. **Add Secrets**: Enter your sensitive information (e.g., passwords, API keys, notes) into the application.
2. **Encryption**: The application encrypts your data using strong encryption algorithms before saving it to a local file.
3. **Secure Storage**: Encrypted data is stored in a local file on your device, accessible only through the application.
4. **Decryption on Demand**: When you need to access your secrets, the application decrypts the data securely and displays it to you.

## Why Secret Vault?

- **Privacy First**: Your data is yours alone. No cloud storage means no risk of third-party breaches.
- **Lightweight**: Built with Tauri, the application is fast, efficient, and has a small footprint.
- **Open Source**: Fully transparent codebase, so you can verify the security and functionality yourself.


## TODO

- [ ] Improve UI
- [ ] Private key backup
- [ ] Password recovery
- [ ] Yubikeys for encryption and or authentication
- [ ] Share functionally, using receiver public key
- [ ] Multi Vault, different security management for each vault

## Development

### Stack

- tauri v2
  - rust backend
  - TS front-end
- svelte v5, with sveltekit

### Setup

__frontend:__

```bash
bun install
```

__backend:__

```bash
cd src-tauri
cargo install
```

### Launch

- development application

```bash
bun tauri dev
```