-- Add up migration script here
CREATE TABLE sms (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    sender TEXT,
    receiver TEXT,
    timestamp timestamp NOT NULL,
    message TEXT NOT NULL,
    device TEXT NOT NULL,
    local_send BOOLEAN NOT NULL
);