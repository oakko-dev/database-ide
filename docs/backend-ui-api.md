# Backend/UI API Contract

The desktop bridge should expose these backend commands to the UI:

| Command | Input | Result |
| --- | --- | --- |
| `connections.list` | none | `SavedConnection[]` |
| `connections.create` | `ConnectionInput` | `SavedConnection` |
| `connections.update` | `{ id, input }` | `SavedConnection` |
| `connections.delete` | `{ id }` | `void` |
| `connections.test` | `{ id }` | `{ success: true }` or safe error |

`ConnectionInput.password` is accepted only for create/update and is forwarded directly to the Rust service. It must not be returned in any result, stored by the UI, written to logs, or serialized into metadata files.

The bridge owns error translation. Errors should preserve actionable categories such as validation, credential storage, not found, and database connection failure while redacting connection strings and credentials.

The browser prototype in `ui/` intentionally does not implement this bridge yet. A desktop adapter should replace its in-memory actions without changing the layout or production-warning behavior.
