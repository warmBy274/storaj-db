# Storaj-DB

⚠️ This project is in the early stages of development.

Storaj-DB is a database management system written in Rust, aiming for high performance and flexibility. It currently supports in-memory storage with basic table operations and network request handling. Persistent storage, global database management and more advanced features are actively being developed.

## Architecture Overview

- Backends:
  - MemoryBackend — fully functional (in-memory storage)
  - StorageBackend — in development (persistent on-disk storage)
  - CombinedBackend — in development (hybrid memory + persistent)
- Network Layer — fully implemented:
  - Accepting connections
  - Request parsing and processing
  - Response generation and sending
- Table Operations — basic operations are implemented
- Access Control — roles and users system are implemented
- Global Operations — database-wide commands (create/drop tables, manage users/roles/permissions etc.) are under development
- Interactive blocking CLI mode — type commands directly in the running program, similar to a simple REPL is under development

## Current Status

### ✅ Fully Implemented
- Project structure and modular organization
- Network connection handling
- Request parsing and processing pipeline
- Response generation and sending
- Basic table operations
- Access Control
- MemoryBackend (complete in-memory storage backend)

### 🔄 In Active Development
- StorageBackend (persistent storage)
- CombinedBackend (memory + storage hybrid)
- Users and roles management system
- Permissions / access control system (not yet working)
- Table storage and access layer
- Global database operations:
  - Create / drop tables
  - Add / remove users
  - Manage roles and permissions
  - Other database management commands

### 📋 Planned Features
- Interactive blocking CLI mode (run the program and enter commands directly in terminal)
- Command-line arguments for startup:
  - Database path
  - Create new or open existing database
  - Server port
  - Other initial settings
- Configuration saving/loading with integrity checks (e.g. hash verification)
- Support for additional data types (currently only numeric types)
- CLI commands mirroring global operations (usable both via network and interactive mode)
- Table-level operations (INSERT, SELECT, UPDATE, DELETE) — significantly lower priority, to be implemented much later

## Contributing & Suggestions

Ideas, feature requests and bug reports are very welcome!  
Please open an Issue on GitHub.

## License

MIT License (to be formally added when appropriate)