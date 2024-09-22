-- Add migration script here
CREATE TABLE word (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    english_word TEXT NOT NULL,
    german_word TEXT NOT NULL,
    date_added TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);