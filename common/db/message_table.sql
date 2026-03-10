CREATE TABLE messages (
  message_id INTEGER PRIMARY KEY,
  user_id TEXT NOT NULL,
  content TEXT NOT NULL,
  FOREIGN KEY (user_id) REFERENCES scores(user_id) ON DELETE CASCADE
);