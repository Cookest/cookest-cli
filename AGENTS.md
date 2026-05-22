# Cookest CLI — Agent Instructions

You are working on the **Cookest CLI**, a Rust binary for deploying and managing self-hosted Cookest instances.

## Quick Reference

| Attribute | Value |
|-----------|-------|
| Language | Rust (edition 2024) |
| CLI Framework | clap 4 (derive) |
| Interactive | dialoguer |
| Output | colored + indicatif |
| Config | TOML (serde) |
| HTTP | reqwest |

## Architecture

```
src/
├── main.rs           ← CLI entry, clap command dispatch
├── config.rs         ← CookestConfig struct, load/save, secret generation
├── docker.rs         ← Docker Compose wrappers (up, down, ps, backup, restore)
├── templates.rs      ← docker-compose.yml, Caddyfile, .env generators
└── commands/
    ├── mod.rs
    ├── init.rs       ← Interactive setup wizard
    ├── up.rs         ← Start services
    ├── down.rs       ← Stop services
    ├── status.rs     ← Health checks + feature status
    ├── logs.rs       ← Tail docker compose logs
    ├── update.rs     ← Pull + restart
    ├── backup.rs     ← pg_dump both databases
    ├── restore.rs    ← pg_restore from backup
    └── config.rs     ← Get/set config values
```

## Key Rules

1. **Config is `cookest.toml`** — TOML file in the instance directory
2. **Docker Compose orchestration** — all services managed via generated docker-compose.yml
3. **Secrets auto-generated** — JWT, DB passwords created with cryptographic randomness
4. **Optional services** — AI, image-gen, Stripe, PDF pipeline toggled per-instance
5. **HTTPS via Caddy** — auto-TLS when domain is not localhost

## Commands

| Command | Description |
|---------|-------------|
| `cookest init` | Interactive setup wizard |
| `cookest up` | Start all services |
| `cookest down` | Stop services |
| `cookest status` | Health checks |
| `cookest logs` | Tail logs |
| `cookest update` | Pull latest + restart |
| `cookest backup` | Dump databases |
| `cookest restore` | Restore from backup |
| `cookest config show/get/set` | Manage configuration |

## Commit Format

```
<type>(<scope>): <description>
```

Types: `feat`, `fix`, `docs`, `refactor`, `test`, `perf`, `build`, `ci`, `chore`
Scopes: `init`, `up`, `down`, `status`, `backup`, `config`, `docker`, `templates`
