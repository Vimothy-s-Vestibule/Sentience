-- Your SQL goes here

ALTER TABLE "vestibule_users" DROP COLUMN "interest_domains";
ALTER TABLE "vestibule_users" DROP COLUMN "interest_activities";
ALTER TABLE "vestibule_users" ADD COLUMN "activities" TEXT[] NOT NULL;
ALTER TABLE "vestibule_users" ADD COLUMN "domains" TEXT[] NOT NULL;

