# Roadmap

## Milestone 0 — Foundation

- [x] Establish repository, documentation, and backend/service boundary.
- [x] Add baseline domain error handling and testable service seams.

## Milestone 1 — Connection Management

- [x] PostgreSQL connection model and validation.
- [x] Secure password storage via OS credential storage.
- [x] Create, edit, delete, list, and test saved connections.
- [x] Environment metadata and read-only mode.
- [x] Safe error messages and no password logging.
- [x] Durable non-secret connection metadata storage.
- [x] UI design contract for connection list, form, and production warning.
- [x] Backend/UI command contract.
- [x] Tauri desktop shell scaffold.
- [x] Tauri commands wired to the connection service.
- [x] Credential and PostgreSQL input hardening.
- [x] UI implementation, including production indicator.

## Milestone 2 — Schema Exploration

- [x] Define typed schema metadata models.
- [x] Add read-only schema, relation, and relation-details queries.
- [x] Expose schema exploration through Tauri commands.
- [ ] Add schema browser UI.
- [ ] Add PostgreSQL integration coverage against a disposable database.
