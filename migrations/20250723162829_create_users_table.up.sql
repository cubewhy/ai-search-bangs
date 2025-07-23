CREATE TABLE IF NOT EXISTS users (
    id BIGINT PRIMARY KEY,
    username TEXT NOT NULL,
    request_count INTEGER NOT NULL DEFAULT 0,
    last_request_date DATE NOT NULL
);
