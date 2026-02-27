## Software Requirements Specification (SRS)

**Project Name:** LDB-Engine (Linux Database Engine)
**Core Philosophy:** A lightweight, GUI-driven local database manager for Linux developers, providing a "1-click" setup experience powered by Docker under the hood.

---

### 1. Technology Stack

* **Backend (Core Daemon):** Rust
* **Desktop Framework:** Tauri v2
* **Frontend (UI):** Svelte (latest), Tailwind CSS v4, Shadcn/ui (https://www.shadcn-svelte.com/)
* **Container Runtime Engine:** Docker Engine API (via Rust Docker SDK)
* **Local Storage (State):** SQLite or equivalent Rust crate (for persisting instance metadata)

---

### 2. Functional Requirements (FR)

**FR1. Docker Hub Integration & Image Discovery**

* The system must fetch and display official repository tags from Docker Hub for supported databases (e.g., PostgreSQL, Redis, MySQL, MongoDB).
* The system must allow users to select specific versions (e.g., `postgres:16-alpine`, `redis:7.2`).
* The system must display real-time download/pull progress when a new image is being fetched.

**FR2. Instance Creation & Configuration**

* The system must provide a unified form to create a new database instance.
* Required inputs: Instance Name, Database Type, Version, and Root Password.
* The system must automatically detect occupied host ports and suggest the next available port (e.g., if 5432 is taken, suggest 5433).
* The system must map user-defined passwords to the correct Docker environment variables automatically (e.g., `POSTGRES_PASSWORD`, `MYSQL_ROOT_PASSWORD`, or Redis `--requirepass` arguments).

**FR3. Instance Lifecycle Management**

* The system must allow users to Start, Stop, Restart, and Delete individual instances.
* The system must display the current execution state of each instance (Running, Stopped, Error) by polling the Docker daemon.

**FR4. Persistent Data Management**

* The system must automatically create and map local host directories to container volumes to ensure database data persists across container restarts.
* Upon deleting an instance, the system must explicitly prompt the user with a checkbox: "Delete associated volume data?" to prevent accidental data loss.

**FR5. Connection Utilities**

* The system must automatically generate a standard connection string for running instances.
* The system must provide a one-click "Copy Connection String" button (e.g., `postgresql://user:password@127.0.0.1:5432/db`).

**FR6. Real-time Log Viewer**

* The system must include an embedded terminal view (e.g., using Xterm.js) to stream standard output (stdout) and standard error (stderr) logs directly from the selected Docker container.

---

### 3. Non-Functional Requirements (NFR)

**NFR1. Window Manager & Wayland Compatibility (Niri Focus)**

* The application must run natively on Wayland without relying on XWayland.
* The UI layout must respond gracefully to dynamic resizing, specifically optimized for the column-based, scrollable-tiling behavior of Niri.
* Popups, dropdowns (like Shadcn/ui selects), and modals must not render as disjointed floating windows that break Niri's tiling rules; they must be constrained within the main application window context.
* The application must support comprehensive keyboard navigation to align with keyboard-driven workflow preferences typical of Niri users.

**NFR2. Performance & Resource Constraints**

* The application binary must remain lightweight (under 50MB).
* The background Go daemon and the Wails webview must have a minimal memory footprint. It must operate efficiently and smoothly alongside Docker Engine, code editors, and browsers, ensuring stable performance on machines with restricted hardware resources, such as those with 8GB of RAM.
* The application must start and become interactive within 2 seconds.

**NFR3. User Interface & Experience (UI/UX)**

* The interface must follow a clean, developer-focused aesthetic.
* The application must strictly adhere to the system's color scheme, defaulting to a high-contrast Dark Mode.
* Status indicators must use clear color coding (Green for running, Gray for stopped, Red for error).

**NFR4. Deployment & Distribution**

* The application must be packagable as a standalone binary for Linux.
* The primary distribution method should be an Arch User Repository (AUR) package (e.g., `ldb-engine-bin`), with a secondary option for Flatpak to ensure cross-distribution compatibility.

---

### 4. System Architecture Flow (Creating an Instance)

1. User submits the "Create Instance" form via the Svelte frontend.
2. Svelte frontend calls a Tauri command: `create_db_instance(config)`.
3. Rust backend receives the struct, validates the requested host port, and prepares the Docker container configuration.
4. Rust backend connects to `/var/run/docker.sock` via the Docker SDK to pull the image (if missing) and create the container with the specified volume mounts and environment variables.
5. Rust backend starts the container and saves the instance metadata (ID, Name, Port) to the local SQLite database.
6. Rust backend returns a success signal to the frontend, which updates the UI state.

---
