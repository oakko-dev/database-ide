# Database IDE Engineering Instructions

- Keep database communication behind the backend/service layer.
- Never persist or log database passwords. Passwords belong in OS credential storage.
- Keep production connections visually distinct in any future UI.
- Prefer small, testable changes and run formatting, linting, type checks, and tests before handoff.
