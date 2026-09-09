# Backend/UI API Contract

The desktop bridge should expose these backend commands to the UI:

| Command | Input | Result |
| --- | --- | --- |
| `connections.list` | none | `SavedConnection[]` |
| `connections.create` | `ConnectionInput` | `SavedConnection` |
| `connections.update` | `{ id, input }` | `SavedConnection` |
| `connections.delete` | `{ id }` | `void` |
| `connections.test` | `{ id }` | `{ success: true }` or safe error |
| `schema.list` | `{ id }` | `SchemaInfo[]` |
| `schema.relations` | `{ id, schema_name }` | `RelationInfo[]` |
| `schema.describe` | `{ id, schema_name, relation_name }` | `RelationDetails` |

`ConnectionInput.password` is accepted only for create/update and is forwarded directly to the Rust service. It must not be returned in any result, stored by the UI, written to logs, or serialized into metadata files.

The bridge owns error translation. Errors should preserve actionable categories such as validation, credential storage, not found, and database connection failure while redacting connection strings and credentials.

The Tauri shell exposes these commands from `src-tauri/src/main.rs`. `ui/bridge.js` calls the native command bridge when hosted by Tauri and uses an in-memory fallback when opened directly in a browser.
