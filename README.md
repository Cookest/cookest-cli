# Cookest CLI

🍳 `cookest` is a command-line tool written in Rust to easily deploy, configure, and manage a self-hosted Cookest instance.

It wraps Docker Compose operations and handles interactive setup wizards, secret generation, backups, updates, and configuration management.

---

## Prerequisites

- **Docker Engine** 24.0+
- **Docker Compose** v2.20+
- (Optional, for compilation) **Rust toolchain** (1.87+)

---

## Installation

### From Source (Recommended)

To compile the CLI tool from source, clone the repository and run:

```bash
cargo build --release
```

The compiled binary will be available at `target/release/cookest`. You can move it to your system path:

```bash
sudo cp target/release/cookest /usr/local/bin/
```

### Using Docker

If you do not want to install Rust, you can build and run the CLI inside Docker. Build the image:

```bash
docker build -t cookest-cli .
```

Run CLI commands by mounting the Docker socket and working directory:

```bash
docker run --rm -it \
  -v /var/run/docker.sock:/var/run/docker.sock \
  -v $(pwd):/work \
  -w /work \
  cookest-cli init
```

---

## Commands

### `init`
Launches the interactive setup wizard. It will guide you through:
- Setting the instance name and domain name (e.g., `localhost` or a custom domain).
- Enabling HTTPS via Let's Encrypt (using Caddy).
- Configuring the default Admin user credentials.
- Choosing local AI models (Ollama chat and vision models).
- Toggling optional features (Stripe, PDF scraping pipeline, outbound email via Resend).

**Output files generated:**
- `cookest.toml` (Persistent configuration)
- `docker-compose.yml` (Docker Compose services definition)
- `.env` (Environment variables for the containers)
- `Caddyfile` (If HTTPS was selected)
- `data/` and `backups/` directories

```bash
cookest init
```

### `up`
Start the self-hosted Cookest stack. After the containers are healthy, automatically
provisions the admin account using the credentials collected during `cookest init`.

```bash
cookest up
```

On first run the CLI will:
1. Start all Docker Compose services
2. Wait for the App API to pass its health check
3. Call `POST /admin/setup` with the credentials from `cookest.toml`
4. Print the admin email so you know it worked

If the admin account already exists (subsequent `cookest up` runs), step 3 is a no-op.

### `down`
Stop the self-hosted Cookest stack. Equivalent to running `docker compose down`.

```bash
cookest down
```

### `status`
Inspect the health and status of the Cookest services and databases.

```bash
cookest status
```

### `logs`
Tail the logs of the running services.

```bash
cookest logs
```

### `update`
Pull the latest Docker images from the GitHub Container Registry (GHCR) and restart the containers with no data loss.

```bash
cookest update
```

### `backup`
Create a gzipped SQL dump of the Cookest databases (both `app-db` and `food-db`) and save them inside the `backups/` directory.

```bash
cookest backup
```

### `restore`
Restore databases from a specific backup file located in the `backups/` directory.

```bash
cookest restore
```

### `config`
Read or modify values in your `cookest.toml` configuration.

```bash
# Read a config key
cookest config network.admin_port

# Write/update a config key
cookest config ai.enabled false
```
