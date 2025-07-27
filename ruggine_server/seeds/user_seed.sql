-- Vai su pgAdmin, connettiti al tuo server
-- Poi esegui una alla volta:

CREATE DATABASE ruggine;
CREATE DATABASE ruggine_test;

-- Connettiti al database ruggine

-- Crea ENUM user_type solo se non esiste già
DO $$
BEGIN
    IF NOT EXISTS (SELECT 1 FROM pg_type WHERE typname = 'user_type') THEN
        CREATE TYPE user_type AS ENUM ('end_user', 'developer', 'admin');
    END IF;

    IF NOT EXISTS (SELECT 1 FROM pg_type WHERE typname = 'user_status') THEN
        CREATE TYPE user_status AS ENUM ('pending', 'active', 'suspended', 'deleted', 'banned');
    END IF;
END$$;

-- Drop della tabella se esiste già
DROP TABLE IF EXISTS "user";

-- Creazione tabella "user"
CREATE TABLE "user" (
    id SERIAL PRIMARY KEY,
    first_name VARCHAR(255) NOT NULL,
    last_name VARCHAR(255) NOT NULL,
    username VARCHAR(255) NOT NULL UNIQUE,
    email VARCHAR(255) NOT NULL UNIQUE,
    password VARCHAR(255) NOT NULL,
    user_type user_type NOT NULL DEFAULT 'end_user',
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    user_status user_status NOT NULL DEFAULT 'active'
);

INSERT INTO "user" (first_name, last_name, username, email, password, user_type, user_status)
VALUES
('Test', 'User', 'testuser', 'test.user@example.com', '$2b$04$somethinghashed', 'developer', 'active');


-- Connettiti al database ruggine_test

-- Crea ENUM user_type solo se non esiste già
DO $$
BEGIN
    IF NOT EXISTS (SELECT 1 FROM pg_type WHERE typname = 'user_type') THEN
        CREATE TYPE user_type AS ENUM ('end_user', 'developer', 'admin');
    END IF;

    IF NOT EXISTS (SELECT 1 FROM pg_type WHERE typname = 'user_status') THEN
        CREATE TYPE user_status AS ENUM ('pending', 'active', 'suspended', 'deleted', 'banned');
    END IF;
END$$;

-- Drop della tabella se esiste già
DROP TABLE IF EXISTS "user";

-- Creazione tabella "user"
CREATE TABLE "user" (
    id SERIAL PRIMARY KEY,
    first_name VARCHAR(255) NOT NULL,
    last_name VARCHAR(255) NOT NULL,
    username VARCHAR(255) NOT NULL UNIQUE,
    email VARCHAR(255) NOT NULL UNIQUE,
    password VARCHAR(255) NOT NULL,
    user_type user_type NOT NULL DEFAULT 'end_user',
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    user_status user_status NOT NULL DEFAULT 'active'
);

INSERT INTO "user" (first_name, last_name, username, email, password, user_type, user_status)
VALUES
('Test', 'User', 'testuser', 'test.user@example.com', '$2b$04$somethinghashed', 'developer', 'active');
