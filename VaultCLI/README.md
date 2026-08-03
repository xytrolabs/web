# Xytro Vault — Linux CLI & FUSE Mount

Mount and manage your Xytro Vault cloud storage from the Linux terminal — like Google Drive, but preconfigured for Xytro.

## One-Line Install

```bash
curl -sSL https://xytro.site/vault-cli | bash
```

This installs all three tools (`xytro-vault`, `xytro-vault-mount`, `xytro-vault-desktop`) into `~/.local/bin/` and adds a desktop entry so "Xytro Vault" appears in your app launcher.

**Requirements:** Python 3, pip. FUSE (`libfuse2`) optional for mount support.

### Manual Install

```bash
cd VaultCLI/
./install.sh
```

## CLI Tool (`xytro-vault`)

### Login
```bash
xytro-vault login
# Enter your Xytro username/email and password
# Session is saved for 7 days
```

### File Management
```bash
xytro-vault ls /                    # List root
xytro-vault ls /Documents           # List a folder
xytro-vault upload ./report.pdf /   # Upload a single file
xytro-vault upload ~/my-project/ /backups/  # Upload entire folder
xytro-vault download /photos/img.jpg ./downloads/  # Download
xytro-vault mkdir /NewFolder        # Create folder
xytro-vault rm /old-file.txt        # Move to trash
xytro-vault share /Documents/report.pdf  # Create share link
```

### Sync & Usage
```bash
xytro-vault sync ~/Documents/MyProject /MyProject/  # One-way sync
xytro-vault usage                   # Show storage usage bar
xytro-vault whoami                  # Show current user
xytro-vault logout                  # Log out
```

## FUSE Mount (`xytro-vault-mount`)

Mount The Vault as a local directory:

```bash
mkdir ~/Vault
xytro-vault-mount ~/Vault
# Now use it like a normal directory:
ls ~/Vault/
cp myfile.txt ~/Vault/Documents/
nano ~/Vault/notes.txt
```

To unmount:
```bash
fusermount -u ~/Vault
# or: xytro-vault-mount -u ~/Vault
```

### Features
- **Lazy loading** — files downloaded on first access
- **In-memory cache** — 100MB cache with automatic eviction
- **Directory listing** — cached for 60 seconds
- **Read/write/create/delete** — full CRUD support
- **Automatic authentication** — uses the same session as the CLI

### Limitations
- **No partial writes** — FUSE write() is stubbed; use cp/mv for whole-file operations
- **No cross-folder rename** — copies + deletes instead (can be slow for large files)
- **No real-time sync** — directory listings are cached for 60 seconds
- **Single-threaded** — not suitable for concurrent heavy I/O

## Folder Upload (Web UI)

The Vault web UI at https://cloud.xytro.site now supports folder uploads:
- Click **Upload Folder** to select a directory
- Preserves subfolder structure
- Shows progress bar with file count
- Batch uploads up to 500 files at once

---

Built for the Xytro ecosystem — PrismTechnologies
