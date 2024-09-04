CREATE TABLE IF NOT EXISTS chat.user (
	id SERIAL PRIMARY KEY,
	username VARCHAR(40) UNIQUE NOT NULL,
	email VARCHAR(255) UNIQUE NOT NULL,
	password VARCHAR(60) NOT NULL
);
