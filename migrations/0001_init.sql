-- Create table to store ShowCreated events
CREATE TABLE IF NOT EXISTS show_created_events (
    id BIGSERIAL PRIMARY KEY,
    tx_hash TEXT,
    block_number BIGINT,
    contract_address TEXT NOT NULL,
    log_index BIGINT,
    raw_event JSONB NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE (tx_hash, log_index)
);

-- Indexes for querying
CREATE INDEX IF NOT EXISTS idx_show_created_block ON show_created_events(block_number);
CREATE INDEX IF NOT EXISTS idx_show_created_contract ON show_created_events(contract_address);

-- Detailed parsed ShowCreated (1 row per show)
CREATE TABLE IF NOT EXISTS show_created_events_detail (
    show_id BIGINT PRIMARY KEY,
    organizer TEXT NOT NULL,
    name TEXT NOT NULL,
    start_time BIGINT NOT NULL,
    end_time BIGINT NOT NULL,
    venue TEXT NOT NULL,
    tx_hash TEXT,
    block_number BIGINT,
    log_index BIGINT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_show_created_detail_block ON show_created_events_detail(block_number);
