CREATE TABLE IF NOT EXISTS chat.user (
	id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
	username VARCHAR(40) UNIQUE NOT NULL,
	email VARCHAR(255) UNIQUE NOT NULL,
	password VARCHAR(60) NOT NULL
);
