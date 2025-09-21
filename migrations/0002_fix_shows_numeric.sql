-- Align shows table numeric types with repository's DbU256 encoding (NUMERIC)
-- Convert event_time (TIMESTAMPTZ) -> NUMERIC(78,0) using epoch seconds
ALTER TABLE shows
    ALTER COLUMN event_time TYPE
NUMERIC
(78,0)
        USING EXTRACT
(EPOCH FROM event_time)::numeric;

-- Convert bigint columns -> NUMERIC(78,0)
ALTER TABLE shows
    ALTER COLUMN ticket_price TYPE
NUMERIC
(78,0)
        USING ticket_price::numeric,
ALTER COLUMN max_tickets TYPE NUMERIC
(78,0)
        USING max_tickets::numeric,
ALTER COLUMN sold_tickets TYPE NUMERIC
(78,0)
        USING sold_tickets::numeric;
