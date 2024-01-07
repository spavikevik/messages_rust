CREATE TABLE messages (
    id BLOB PRIMARY KEY,
    user_id BLOB NOT NULL,
    content TEXT NOT NULL,
    parent_message_id BLOB ,
    created_at TEXT NOT NULL DEFAULT (DATETIME('now', 'localtime')),
    updated_at TEXT NOT NULL DEFAULT (DATETIME('now', 'localtime')),
    FOREIGN KEY(user_id) REFERENCES users(`id`) ON DELETE CASCADE,
    FOREIGN KEY(parent_message_id) references messages(`id`) ON DELETE CASCADE
);

CREATE TRIGGER trigger_updated_at_for_messages AFTER UPDATE ON messages
BEGIN
    UPDATE messages SET updated_at = DATETIME('now', 'localtime') WHERE rowid == NEW.rowid;
END;

CREATE INDEX IF NOT EXISTS idx_by_parent_message_id ON messages(parent_message_id);
CREATE INDEX IF NOT EXISTS idx_by_user_id ON users(id);