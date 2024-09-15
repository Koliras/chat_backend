CREATE TABLE IF NOT EXISTS chat.chat (
	id SERIAL PRIMARY KEY,
	name VARCHAR(50) NOT NULL,
	admin_id INT,
	FOREIGN KEY(admin_id) REFERENCES chat.user(id)
)

CREATE TABLE IF NOT EXISTS chat.user_chat (
	user_id INT,
	chat_id INT,
	PRIMARY KEY(user_id, chat_id),
	FOREIGN KEY(user_id) REFERENCES chat.user(id),
	FOREIGN KEY(chat_id) REFERENCES chat.chat(id)
);
