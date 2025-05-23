DROP TABLE IF EXISTS user;
DROP TABLE IF EXISTS rooms;
DROP TABLE IF EXISTS room_membership;
DROP TABLE IF EXISTS chat_history;

CREATE TABLE user (
  user_id INTEGER PRIMARY KEY AUTOINCREMENT,
  created TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
  username VARCHAR NOT NULL,
  email VARCHAR NOT NULL,
  email_verified BIT NOT NULL,
  role VARCHAR NOT NULL,
  password_hash INTEGER
);

CREATE TABLE room (
  room_id INTEGER PRIMARY KEY AUTOINCREMENT,
  created TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
  room_name VARCHAR NOT NULL,
  is_public BIT NOT NULL
);

CREATE TABLE room_membership (
  user_id INTEGER,
  room_id INTEGER,
  PRIMARY KEY (user_id, room_id),
  FOREIGN KEY (user_id) REFERENCES user(user_id),
  FOREIGN KEY (room_id) REFERENCES room(room_id)
);

CREATE TABLE chat_history(
  message_id INTEGER PRIMARY KEY AUTOINCREMENT,
  message_time TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
  message_text TEXT,
  sender_id INTEGER NOT NULL,
  room_id INTEGER NOT NULL,
  FOREIGN KEY (room_id) REFERENCES room(room_id)
);

