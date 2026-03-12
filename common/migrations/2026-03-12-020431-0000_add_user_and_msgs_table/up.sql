-- Your SQL goes here

CREATE TABLE "messages"(
	"message_id" TEXT NOT NULL PRIMARY KEY,
	"username" TEXT NOT NULL,
	"user_id" TEXT NOT NULL,
	"content" TEXT NOT NULL,
	"created_at" TIMESTAMPTZ NOT NULL
);

CREATE TABLE "vestibule_users"(
	"discord_user_id" TEXT NOT NULL PRIMARY KEY,
	"discord_username" TEXT NOT NULL,
	"yt_username" TEXT,
	"yt_display_name" TEXT,
	"intro_message_id" TEXT,
	"honesty_humility" FLOAT8 NOT NULL,
	"emotionality" FLOAT8 NOT NULL,
	"extraversion" FLOAT8 NOT NULL,
	"agreeableness" FLOAT8 NOT NULL,
	"conscientiousness" FLOAT8 NOT NULL,
	"openness_to_experience" FLOAT8 NOT NULL,
	"agency" FLOAT8 NOT NULL,
	"communion" FLOAT8 NOT NULL,
	"self_direction" FLOAT8 NOT NULL,
	"stimulation" FLOAT8 NOT NULL,
	"hedonism" FLOAT8 NOT NULL,
	"achievement" FLOAT8 NOT NULL,
	"power" FLOAT8 NOT NULL,
	"security" FLOAT8 NOT NULL,
	"conformity" FLOAT8 NOT NULL,
	"tradition" FLOAT8 NOT NULL,
	"benevolence" FLOAT8 NOT NULL,
	"universalism" FLOAT8 NOT NULL,
	"interest_domains" TEXT[] NOT NULL,
	"interest_activities" TEXT[] NOT NULL,
	"intro_embedding" VECTOR,
	"intro_diagram" BYTEA,
	"status" TEXT NOT NULL,
	"created_at" TIMESTAMPTZ NOT NULL,
	"updated_at" TIMESTAMPTZ NOT NULL,
	FOREIGN KEY ("intro_message_id") REFERENCES "messages"("message_id")
);
