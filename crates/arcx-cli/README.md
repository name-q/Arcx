# arcx-cli

CLI scaffolding tool for [Arcx](https://github.com/name-q/Arcx) framework.

## Install

```bash
cargo install arcx-cli
```

## Commands

### Create a new project

```bash
arcx new my-app
cd my-app
cargo run
```

Generated structure:

```
my-app/
├── src/
│   ├── main.rs           # 3-line entry point
│   ├── router.rs         # Centralized route declarations
│   ├── controller/
│   │   └── home.rs       # Example controller
│   └── service/
├── config/
│   ├── config.default.toml
│   └── config.prod.toml
├── Cargo.toml
└── README.md
```

### Generate code

```bash
# Generate a controller (auto-registered in mod.rs + router.rs)
arcx g c user

# Generate a service
arcx g s user

# Generate a model (SeaORM entity)
arcx g m user

# Generate a scheduled job
arcx g j cleanup
```

### Development server

```bash
# Start with hot-reload (watches src/ and config/)
arcx dev

# Specify port
arcx dev -p 8080
```

Hot-reload behavior:
- `.rs` files changed → incremental compile + restart
- `config/*.toml` changed → signal process to reload config
- `Cargo.toml` changed → full rebuild + restart
- Compile errors → old process keeps running

### Project info

```bash
arcx info
```

## How code generation works

`arcx g c user` does three things:

1. Creates `src/controller/user.rs` with RESTful handlers
2. Adds `pub mod user;` to `src/controller/mod.rs`
3. Adds `r.resources("/api/user", controller::user::handlers());` to `src/router.rs`

Zero manual wiring needed.

## License

MIT
