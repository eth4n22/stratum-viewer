# Stratum — Point Cloud Viewer

A Rust-based out-of-core point cloud viewer for massive LAS, LAZ, and PLY files.
Forked from [cartographer-project/point_cloud_viewer](https://github.com/cartographer-project/point_cloud_viewer) with added native LAS/LAZ support.

## What's Different from Upstream

- Native LAS/LAZ file support (no pre-conversion needed)
- RGB colour rendering from LAS point formats 2, 3, 5, 7, 8, 10
- Intensity fallback for LAS formats without colour (0, 1, 4, 6, 9)
- Viewer window renamed to Stratum

---

## Requirements

- Windows 11 with **WSL2** (Ubuntu 22.04 or 24.04)
- Rust (managed via rustup — see below)
- SDL2 libraries for the native viewer

---

## Setup

### 1. Install WSL2

Open PowerShell as Administrator and run:

```powershell
wsl --install
```

Restart when prompted. Ubuntu will be your default distro.

### 2. Install Rust

Inside WSL:

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env
```

### 3. Pin the Rust version

This project requires Rust 1.72.0 due to dependency constraints:

```bash
cd /path/to/stratum-viewer
rustup override set 1.72.0
```

### 4. Install system dependencies

```bash
sudo apt update
sudo apt install -y \
    libsdl2-dev \
    libsdl2-ttf-dev \
    build-essential \
    cmake \
    pkg-config
```

### 5. Clone and build

```bash
git clone https://github.com/eth4n22/stratum-viewer.git
cd stratum-viewer
rustup override set 1.72.0
cargo build --release -p point_viewer
cargo build --release -p sdl_viewer
```

---

## Usage

### Step 1 — Build an octree from your point cloud

The viewer works by first indexing your file into an octree, then streaming it for display.

**Always output the octree to the Linux filesystem** (`/tmp`) — writing to the Windows-mounted drive (`/mnt/c/`) causes I/O errors on large files.

**From a LAS or LAZ file:**
```bash
./target/release/build_octree \
  --output-directory /tmp/my_octree \
  --resolution 0.001 \
  /path/to/your/scan.las
```

**From a PLY file:**
```bash
./target/release/build_octree \
  --output-directory /tmp/my_octree \
  --resolution 0.001 \
  /path/to/your/scan.ply
```

`--resolution` controls point density (metres). Start with `0.001` for LIDAR scans.

### Step 2 — Open in the viewer

```bash
./target/release/sdl_viewer /tmp/my_octree
```

### Viewer Controls

| Key | Action |
| --- | ------ |
| W / A / S / D | Move forward / left / back / right |
| Q / Z | Move up / down |
| Arrow keys | Turn |
| Left mouse drag | Rotate |
| Right mouse drag | Pan |
| Scroll wheel | Adjust movement speed |
| 0 / 9 | Increase / decrease point size |
| 8 / 7 | Brighten / darken scene |
| O | Show octree nodes |
| Ctrl + 0–9 | Load saved camera position |
| Shift + Ctrl + 0–9 | Save current camera position |

---

## Web Viewer (share without installing SDL2)

The web viewer serves your octree over HTTP so anyone can view it in a browser — no native build needed on their end.

```bash
cargo build --release -p octree_web_viewer
./target/release/points_web_viewer /tmp/my_octree
```

Then open **http://127.0.0.1:5433** in your browser.

To share with someone on the same network, replace `127.0.0.1` with your machine's local IP and pass it via the `--ip` flag:

```bash
./target/release/points_web_viewer --ip 0.0.0.0 /tmp/my_octree
```

---

## Supported LAS Point Formats

| Format | Colour Source |
| ------ | ------------- |
| 0, 1 | Intensity (greyscale) |
| 2, 3, 5 | RGB |
| 6 | Intensity (greyscale) |
| 7, 8, 10 | RGB |
| 4, 9 | Intensity (greyscale) |

---

## License

Apache 2.0 — see [LICENSE](LICENSE).
Original work by the [Cartographer Authors](https://github.com/cartographer-project/point_cloud_viewer).
