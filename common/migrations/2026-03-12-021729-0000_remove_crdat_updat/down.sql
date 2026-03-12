-- This file should undo anything in `up.sql`

ALTER TABLE "vestibule_users" ADD COLUMN "created_at" TIMESTAMPTZ NOT NULL;
ALTER TABLE "vestibule_users" ADD COLUMN "updated_at" TIMESTAMPTZ NOT NULL;

