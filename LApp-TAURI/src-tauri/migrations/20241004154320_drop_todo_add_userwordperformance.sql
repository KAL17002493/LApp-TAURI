-- Add migration script here
DROP TABLE IF EXISTS todos;

CREATE TABLE user_word_performance (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    fail_count INTEGER DEFAULT 0,
    word_id INTEGER NOT NULL,
    FOREIGN KEY (word_id) REFERENCES word(id) ON DELETE CASCADE
);