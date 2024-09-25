DROP TABLE IF EXISTS chat.message_chat;

ALTER TABLE chat.message
	ADD COLUMN chat_id UUID;

ALTER TABLE chat.message
	ADD CONSTRAINT chat_message_fk FOREIGN KEY (chat_id) REFERENCES chat.chat (id);
