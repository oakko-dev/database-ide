# Architecture

The Database IDE is a desktop application with a UI client and a Rust backend/service layer.

The UI owns presentation and user interaction only. The backend owns connection validation, credential access, PostgreSQL communication, and error translation. UI code must never connect directly to PostgreSQL or receive a password unless it is actively submitting one for storage.

Saved connection metadata is non-secret configuration. Passwords are stored under the OS credential manager using a stable per-connection service/account key. On macOS this is Keychain; supported OSes may use their native credential store through the `keyring` crate.

Milestone 1 currently provides the backend domain/service foundation. A later UI milestone can consume the service API and render `PRODUCTION` connections with a visually distinct warning treatment.
