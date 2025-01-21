-- Add up migration script here
CREATE TABLE sms (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    sender TEXT NOT NULL,
    timestamp INTEGER NOT NULL CHECK(timestamp > 0),
    message TEXT NOT NULL CHECK(LENGTH(message) <= 4096),
    device TEXT NOT NULL,
    local_send BOOLEAN NOT NULL
);