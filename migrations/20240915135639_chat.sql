CREATE TABLE IF NOT EXISTS chat.chat (
	id SERIAL PRIMARY KEY,
	name VARCHAR(50) NOT NULL,
	admin_id INT,
	FOREIGN KEY(admin_id) REFERENCES chat.user(id)
);

CREATE TABLE IF NOT EXISTS chat.user_chat (
	user_id INT,
	chat_id INT,
	PRIMARY KEY(user_id, chat_id),
	FOREIGN KEY(user_id) REFERENCES chat.user(id),
	FOREIGN KEY(chat_id) REFERENCES chat.chat(id)
);

CREATE TABLE IF NOT EXISTS chat.message (
	id SERIAL PRIMARY KEY,
	content VARCHAR(400) NOT NULL,
	user_id INT,
	created_at TIMESTAMP DEFAULT(NOW()::timestamp),
	FOREIGN KEY(user_id) REFERENCES chat.user(id)
);

CREATE TABLE IF NOT EXISTS chat.message_chat (
	message_id INT,
	chat_id INT,
	PRIMARY KEY(message_id, chat_id),
	FOREIGN KEY(message_id) REFERENCES chat.message(id),
	FOREIGN KEY(chat_id) REFERENCES chat.chat(id)
);
