-- Add migration script here

-- Wrap the entire query in a transaction, just in case any part of the query fails
-- then the parts that succeeded will be rollbacked.
BEGIN;
    -- Backfill `status` for historical records
    UPDATE subscriptions
        SET status = 'confirmed'
        WHERE status IS NULL;
    -- Make `status` mandatory
    ALTER TABLE subscriptions ALTER COLUMN status SET NOT NULL;
COMMIT;
