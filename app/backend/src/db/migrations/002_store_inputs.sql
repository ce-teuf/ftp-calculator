-- Migration: 002_store_inputs
-- Description: Add input matrices columns to executions so runs can be replayed.

ALTER TABLE executions
    ADD COLUMN IF NOT EXISTS outstanding_json TEXT,
    ADD COLUMN IF NOT EXISTS profiles_json    TEXT,
    ADD COLUMN IF NOT EXISTS rates_json       TEXT;
