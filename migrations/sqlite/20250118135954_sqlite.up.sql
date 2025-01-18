-- Add up migration script here
-- Add up migration script here
CREATE TABLE
    messages (
        id INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
        uuid char(32) UNIQUE NOT NULL,
        message char(160) NOT NULL,
        mobile char(15) NOT NULL,
        status INTEGER DEFAULT 0,
        retries INTEGER DEFAULT 0,
        device string NULL,
        created_at TIMESTAMP default CURRENT_TIMESTAMP,
        updated_at TIMESTAMP
    );