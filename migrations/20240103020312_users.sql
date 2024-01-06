CREATE TABLE users (
    id BLOB PRIMARY KEY,
    display_name TEXT NOT NULL,
    username TEXT NOT NULL,
    password_hash TEXT NOT NULL,
    created_at INTEGER NOT NULL DEFAULT (DATETIME('now', 'localtime')),
    updated_at INTEGER NOT NULL DEFAULT (DATETIME('now', 'localtime'))
);

CREATE TRIGGER trigger_updated_at_for_users AFTER UPDATE ON users
BEGIN
    UPDATE users SET updated_at = DATETIME('now', 'localtime') WHERE rowid == NEW.rowid;
END;