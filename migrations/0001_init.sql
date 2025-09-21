-- Create table to store ShowCreated events
CREATE TABLE IF NOT EXISTS show_created_events (
    show_id NUMERIC(78,0) PRIMARY KEY,
    tx_hash TEXT,
    block_number NUMERIC(78,0),
    organizer TEXT NOT NULL,
    log_index NUMERIC(78,0),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE (tx_hash, log_index)
);
-- Enum for show status
CREATE TYPE SHOW_STATUS AS ENUM ('UPCOMING', 'ACTIVE', 'ENDED', 'CANCELLED');
-- Detailed parsed ShowCreated (1 row per show)
CREATE TABLE IF NOT EXISTS show_created_events_detail (
    show_id NUMERIC(78,0) PRIMARY KEY,
    start_time NUMERIC(78,0) NOT NULL,
    end_time NUMERIC(78,0) NOT NULL,
    total_tickets NUMERIC(78,0) NOT NULL,
    ticket_price NUMERIC(78,0) NOT NULL,
    decimal BIGINT NOT NULL,
    ticket_sold NUMERIC(78,0) NOT NULL DEFAULT 0,
    organizer TEXT NOT NULL,
    location TEXT NOT NULL,
    name TEXT NOT NULL,
    description TEXT NOT NULL,
    metadata_uri TEXT,
    status SHOW_STATUS NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS shows (
    id NUMERIC(78,0) PRIMARY KEY,
    name TEXT NOT NULL,
    description TEXT NOT NULL,
    location TEXT NOT NULL,
    event_time TIMESTAMPTZ NOT NULL,
    ticket_price BIGINT NOT NULL,
    max_tickets BIGINT NOT NULL,
    sold_tickets BIGINT NOT NULL DEFAULT 0,
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    organizer TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE INDEX idx_shows_event_time ON shows (event_time);
CREATE INDEX idx_shows_is_active ON shows (is_active);
CREATE INDEX idx_shows_organizer ON shows (organizer);