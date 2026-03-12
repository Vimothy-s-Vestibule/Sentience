-- This file should undo anything in `up.sql`

ALTER TABLE "vestibule_users" DROP COLUMN "activities";
ALTER TABLE "vestibule_users" DROP COLUMN "domains";
ALTER TABLE "vestibule_users" ADD COLUMN "interest_domains" TEXT[] NOT NULL;
ALTER TABLE "vestibule_users" ADD COLUMN "interest_activities" TEXT[] NOT NULL;

