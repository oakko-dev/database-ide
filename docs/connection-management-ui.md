# Connection Management UI

## Design direction

The connection workspace is a repeated-use developer tool: dense, quiet, and scannable. The first viewport shows the saved connection list and a primary “New connection” action. The memorable detail is a strong amber production rail and `PRODUCTION` label that remain visible without relying on color alone.

## Connection list

Each row displays:

- Connection name
- Environment label: `LOCAL`, `UAT`, or `PRODUCTION`
- Host and database
- Read-only badge when enabled
- Test, edit, and delete actions

Production rows use an amber left border, warning icon, and text label. Do not use a red destructive treatment for the row itself; deletion remains the destructive action.

## Form behavior

Fields are name, host, port, database, username, password, environment, and read-only mode. Password input is write-only: the UI may submit it to the backend but must never render or persist it after submission. Editing without changing the password must leave the existing Keychain entry untouched.

Validation errors should appear next to the relevant field. Connection-test errors should identify the operation and provide an actionable message without exposing credentials.

## Backend boundary

The UI consumes `SavedConnection` metadata and calls the connection service for CRUD and testing. It never builds a PostgreSQL client, reads the credential store, or logs request payloads.

## Accessibility and interaction checklist

- Production status is conveyed by text, icon, and border—not color alone.
- All form controls have visible labels and keyboard focus states.
- Delete requires confirmation and identifies the connection by name.
- Test connection exposes pending, success, and failure states.
- Long names and hosts wrap without shifting action controls.
- Destructive and production-warning actions have distinct visual treatments.
