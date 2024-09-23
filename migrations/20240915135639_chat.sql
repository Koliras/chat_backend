CREATE TABLE IF NOT EXISTS chat.chat (
	id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
	name VARCHAR(50) NOT NULL,
	admin_id UUID,
	FOREIGN KEY(admin_id) REFERENCES chat.user(id)
);

CREATE TABLE IF NOT EXISTS chat.user_chat (
	user_id UUID,
	chat_id UUID,
	PRIMARY KEY(user_id, chat_id),
	FOREIGN KEY(user_id) REFERENCES chat.user(id),
	FOREIGN KEY(chat_id) REFERENCES chat.chat(id)
);

CREATE TABLE IF NOT EXISTS chat.message (
	id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
	content VARCHAR(400) NOT NULL,
	user_id UUID,
	created_at TIMESTAMP DEFAULT(NOW()::timestamp),
	FOREIGN KEY(user_id) REFERENCES chat.user(id)
);

CREATE TABLE IF NOT EXISTS chat.message_chat (
	message_id UUID,
	chat_id UUID,
	PRIMARY KEY(message_id, chat_id),
	FOREIGN KEY(message_id) REFERENCES chat.message(id),
	FOREIGN KEY(chat_id) REFERENCES chat.chat(id)
);
