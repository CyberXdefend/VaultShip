# Publishing Guide

This document covers the one-time setup required for each distribution channel. Once set up, every `git push --tags v*` automatically publishes everywhere.

## How releases work

```
git tag v1.2.3
git push origin v1.2.3
```

The `release.yml` workflow fires and runs these jobs **in order**:

```
binaries ─────────────────────────────┐
docker   (parallel)                   │
publish-crates (parallel)             │
                                      ▼
                              github-release
                             /    |    |    \    |     \     \
                    linux-pkgs  brew  snap  aur  choco  winget  scoop
```

---

## Required GitHub Secrets

Go to **Settings → Secrets and variables → Actions → New repository secret** for each:

| Secret name | Used by | How to get it |
|---|---|---|
| `DOCKERHUB_TOKEN` | docker | DockerHub → Account Settings → Security → New Access Token |
| `DOCKERHUB_USERNAME` | docker | Your DockerHub username (e.g. `cyberxdefend`) |
| `CRATES_IO_TOKEN` | publish-crates | crates.io → Account Settings → API Tokens → New Token |
| `HOMEBREW_TAP_TOKEN` | homebrew-tap | GitHub → Settings → Developer Settings → PAT (Classic) with `public_repo` scope |
| `SNAPCRAFT_TOKEN` | snap | See Snap section below |
| `AUR_SSH_KEY` | aur | See AUR section below |
| `CHOCOLATEY_API_KEY` | chocolatey | See Chocolatey section below |
| `WINGET_TOKEN` | winget | GitHub PAT (Classic) with `public_repo` scope |
| `SCOOP_BUCKET_TOKEN` | scoop | GitHub PAT (Classic) with `public_repo` scope |

---

## Channel Setup

### Docker Hub + GHCR

Docker Hub is already configured in `release.yml`. GHCR uses `GITHUB_TOKEN` automatically (no setup needed).

**One-time:**
1. Create DockerHub account at hub.docker.com
2. Create repo `cyberxdefend/vaultship` (public)
3. Add `DOCKERHUB_TOKEN` and `DOCKERHUB_USERNAME` secrets

### crates.io

**One-time:**
1. Sign in at crates.io with your GitHub account
2. Go to Account Settings → API Tokens → New Token (name: "github-actions-vaultship")
3. Add `CRATES_IO_TOKEN` secret
4. The first publish of each crate must be done locally:
   ```bash
   cargo login <your-token>
   cargo publish -p vaultship-harden
   cargo publish -p vaultship-encrypt
   cargo publish -p vaultship-sign
   cargo publish -p vaultship-license
   cargo publish -p vaultship-sdk
   cargo publish -p vaultship-cli
   ```
   Subsequent releases are handled by CI automatically.

### Homebrew (macOS / Linux)

**One-time:**
1. Create a new GitHub repo: `cyberxdefend/homebrew-vaultship` (public)
2. In that repo, create `Formula/vaultship.rb` with the content from `packaging/homebrew/Formula/vaultship.rb`
3. Create a GitHub PAT (Classic) with `public_repo` scope → add as `HOMEBREW_TAP_TOKEN`

**Users install with:**
```bash
brew tap cyberxdefend/vaultship
brew install vaultship
```

### Snap Store

**One-time:**
1. Create account at snapcraft.io
2. Register the snap name: `snapcraft register vaultship`
3. Export credentials:
   ```bash
   snapcraft export-login --snaps=vaultship --channels=stable - | base64
   ```
4. Add the base64 output as `SNAPCRAFT_TOKEN` secret

**Users install with:**
```bash
sudo snap install vaultship
```

### AUR (Arch Linux)

**One-time:**
1. Create an AUR account at aur.archlinux.org
2. Generate an SSH key for the release bot:
   ```bash
   ssh-keygen -t ed25519 -C "vaultship-release-bot" -f ~/.ssh/aur_vaultship
   ```
3. Add the public key to your AUR account: aur.archlinux.org → My Account → SSH Public Key
4. Register the package:
   ```bash
   ssh aur@aur.archlinux.org setup-repo vaultship-bin
   ```
5. Add the private key as `AUR_SSH_KEY` secret (paste the raw content of `~/.ssh/aur_vaultship`)

**Users install with:**
```bash
yay -S vaultship-bin
# or: paru -S vaultship-bin
```

### Chocolatey (Windows)

**One-time:**
1. Create account at community.chocolatey.org
2. Go to Account → API Key → Generate
3. Add as `CHOCOLATEY_API_KEY` secret
4. The first submission to Chocolatey requires moderation review (24–72 hours). Subsequent versions publish automatically.

**Users install with:**
```powershell
choco install vaultship
```

### winget (Windows Package Manager)

winget publishes to the community repo (`microsoft/winget-pkgs`) via automated PRs. Microsoft reviews new packages and updates.

**One-time:**
1. The `vedantmgoyal9/winget-releaser` action creates PRs to `microsoft/winget-pkgs` automatically
2. Create a GitHub PAT (Classic) with `public_repo` scope → add as `WINGET_TOKEN`
3. For the **first submission**, you may need to open the PR manually with the content from `packaging/winget/vaultship.yaml` (with real SHA256 filled in)

**Users install with:**
```powershell
winget install cyberxdefend.VaultShip
```

### Scoop (Windows)

Scoop requires a dedicated "bucket" repository.

**One-time:**
1. Create new GitHub repo: `cyberxdefend/scoop-vaultship` (public)
2. Copy `packaging/scoop/vaultship.json` to the root of that repo
3. Create a GitHub PAT (Classic) with `public_repo` scope → add as `SCOOP_BUCKET_TOKEN`

**Users install with:**
```powershell
scoop bucket add cyberxdefend https://github.com/cyberxdefend/scoop-vaultship
scoop install vaultship
```

### Linux packages (.deb / .rpm)

Built automatically by the `linux-packages` job using nfpm. The `.deb` and `.rpm` files are attached to the GitHub release.

**Users install with:**
```bash
# Debian/Ubuntu
wget https://github.com/cyberxdefend/vaultship/releases/latest/download/vaultship-latest-amd64.deb
sudo dpkg -i vaultship-*.deb

# RHEL/Fedora/CentOS
wget https://github.com/cyberxdefend/vaultship/releases/latest/download/vaultship-latest-x86_64.rpm
sudo rpm -i vaultship-*.rpm
```

### Nix

**Flake (source build)** — no setup required, works immediately:
```bash
nix profile install github:cyberxdefend/vaultship
```

**Binary derivation** (`packaging/nix/default.nix`) — SHA256 placeholders are updated by a post-release script. To use:
```bash
nix-env -if packaging/nix/default.nix
```

---

## Summary of repos to create

| Repo to create | Purpose |
|---|---|
| `cyberxdefend/homebrew-vaultship` | Homebrew tap |
| `cyberxdefend/scoop-vaultship` | Scoop bucket |

Both are regular GitHub repos. The CI jobs push to them automatically on each release.
