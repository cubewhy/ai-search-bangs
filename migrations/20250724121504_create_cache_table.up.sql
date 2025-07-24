CREATE TABLE IF NOT EXISTS cache (
    query_prompt TEXT NOT NULL,
    search_engine TEXT NOT NULL,
    language TEXT NOT NULL,
    url TEXT NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (query_prompt, search_engine, language)
);
