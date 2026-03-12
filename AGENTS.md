# Agent Operational Instructions

This document contains critical constraints and rules for AI agents and developers working on this project.

## Critical Mandates

### Diesel & Database Management

- **DO NOT** edit `common/src/diesel_schema.rs`. This file is automatically managed by the Diesel CLI.
- **DO NOT** edit existing migration files in the `migrations/` directory (e.g., `up.sql`, `down.sql`).

### Architectural Rules

- **Backend/Frontend Isolation**: The `backend` crate must not depend on or reference the `frontend` crate.
- **Shared Logic**: Use the `common` crate for models, schemas, and logic shared between the frontend and backend.
- **Idiomatic Rust**: Prefer idiomatic Rust patterns (e.g., using enums for status fields, proper struct embedding for Diesel models).

## Development Workflow

- Always use `cargo check` or `cargo build` in the relevant crate after making changes to verify compilation.
- Adhere strictly to existing project naming conventions and formatting.
