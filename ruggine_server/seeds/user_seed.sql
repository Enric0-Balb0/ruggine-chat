-- Connettiti al database ruggine

-- Crea ENUM user_type, user_status, current_action, gender solo se non esistono già
DO $$
BEGIN
    IF NOT EXISTS (SELECT 1 FROM pg_type WHERE typname = 'user_type') THEN
        CREATE TYPE user_type AS ENUM ('end_user', 'developer', 'admin');
    END IF;

    IF NOT EXISTS (SELECT 1 FROM pg_type WHERE typname = 'user_status') THEN
        CREATE TYPE user_status AS ENUM ('pending', 'active', 'suspended', 'deleted', 'banned');
    END IF;
    
    IF NOT EXISTS (SELECT 1 FROM pg_type WHERE typname = 'current_action') THEN
        CREATE TYPE current_action AS ENUM ('waiting', 'writing');
    END IF;

    IF NOT EXISTS (SELECT 1 FROM pg_type WHERE typname = 'gender') THEN
        CREATE TYPE gender AS ENUM ('male', 'female', 'other');
    END IF;
END$$;

-- Drop della tabella se esiste già
DROP TABLE IF EXISTS "user";

-- Creazione tabella "user"
CREATE TABLE "user" (
    id SERIAL PRIMARY KEY,
    first_name VARCHAR(256) NOT NULL,
    last_name VARCHAR(256) NOT NULL,
    username VARCHAR(64) NOT NULL UNIQUE,
    email VARCHAR(256) NOT NULL UNIQUE,
    password VARCHAR(256) NOT NULL,
    user_type user_type NOT NULL DEFAULT 'end_user',
    user_status user_status NOT NULL DEFAULT 'active',
    birthday DATE NOT NULL,
    is_online BOOLEAN NOT NULL DEFAULT false,
    address VARCHAR(256) NOT NULL,
    current_action current_action NOT NULL DEFAULT 'waiting',
    gender gender NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP
);

INSERT INTO "user" (first_name, last_name, username, email, password, user_type, user_status, birthday, is_online, address, current_action, gender)
VALUES
('Test', 'User', 'testuser', 'test.user@example.com', '$2b$04$somethinghashed', 'developer', 'active', '1990-01-01', false, '123 Main St', 'waiting', 'male');

-- Crea la funzione per aggiornare il campo updated_at
CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
  NEW.updated_at = CURRENT_TIMESTAMP;
  RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- Crea il trigger sulla tabella user
CREATE TRIGGER set_user_updated_at
BEFORE UPDATE ON "user"
FOR EACH ROW
EXECUTE FUNCTION update_updated_at_column();