-- Drop della tabella se esiste già
DROP TABLE IF EXISTS "group_membership";
DROP TABLE IF EXISTS "invitation";
DROP TABLE IF EXISTS "group_chat";
DROP TABLE IF EXISTS "user";

-- Crea la funzione per aggiornare il campo updated_at
CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
  NEW.updated_at = CURRENT_TIMESTAMP;
  RETURN NEW;
END;
$$ LANGUAGE plpgsql;

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
('Test1', 'User1', 'testuser1', 'test.user1@example.com', '$2b$04$somethinghashed', 'developer', 'active', '1990-01-01', false, '123 Main St', 'waiting', 'male');

INSERT INTO "user" (first_name, last_name, username, email, password, user_type, user_status, birthday, is_online, address, current_action, gender)
VALUES
('Test2', 'User2', 'testuser2', 'test.user2@example.com', '$2b$04$somethinghashed', 'end_user', 'active', '2000-01-01', false, '123 Main St', 'waiting', 'female');



-- Crea il trigger sulla tabella user
CREATE TRIGGER set_user_updated_at
BEFORE UPDATE ON "user"
FOR EACH ROW
EXECUTE FUNCTION update_updated_at_column();

-- Creazione tabella "group_chat_service"
CREATE TABLE group_chat (
    id SERIAL PRIMARY KEY,
    name VARCHAR(256) NOT NULL,
    description VARCHAR(1024) NOT NULL,
    created_by INT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    CONSTRAINT fk_created_by FOREIGN KEY (created_by) REFERENCES "user"(id)
);

-- Inserimento di un gruppo di chat di esempio
INSERT INTO "group_chat" (name, description, created_by)
VALUES ('Developers', 'Chat group for developers', 1);

-- Crea il trigger sulla tabella group_chat_service
CREATE TRIGGER set_group_chat_updated_at
BEFORE UPDATE ON "group_chat"
FOR EACH ROW
EXECUTE FUNCTION update_updated_at_column();

-- Assicurati che esista il tipo ENUM invitation_status
DO $$
BEGIN
    IF NOT EXISTS (SELECT 1 FROM pg_type WHERE typname = 'invitation_status') THEN
        CREATE TYPE invitation_status AS ENUM ('pending', 'accepted', 'rejected');
    END IF;
END$$;

-- Creazione tabella INVITATION
CREATE TABLE invitation (
    id SERIAL PRIMARY KEY,
    from_user_id INTEGER NOT NULL REFERENCES "user"(id),
    to_user_id INTEGER NOT NULL REFERENCES "user"(id),
    group_chat_id INTEGER NOT NULL REFERENCES group_chat(id),
    status invitation_status DEFAULT 'pending',
    sent_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    responded_at TIMESTAMPTZ
);

-- Crea vincolo su inviti
CREATE UNIQUE INDEX unique_pending_invitation
ON invitation (from_user_id, to_user_id, group_chat_id)
WHERE status = 'pending';

-- Inserimento record di esempio
INSERT INTO invitation (
    from_user_id,
    to_user_id,
    group_chat_id,
    status,
    sent_at,
    responded_at
) VALUES (
    1,            -- from_user_id
    2,            -- to_user_id
    1,            -- group_chat_id
    'pending',    -- status
    DEFAULT,      -- sent_at
    NULL          -- responded_at
);

-- Crea ENUM member_role solo se non esiste
DO $$
BEGIN
    IF NOT EXISTS (SELECT 1 FROM pg_type WHERE typname = 'member_role') THEN
        CREATE TYPE member_role AS ENUM ('member', 'admin');
    END IF;
END$$;

-- Crea ENUM per membership_status solo se non esiste
DO $$
BEGIN
    IF NOT EXISTS (SELECT 1 FROM pg_type WHERE typname = 'membership_status') THEN
        CREATE TYPE membership_status AS ENUM ('active', 'left', 'banned');
    END IF;
END$$;

-- Creazione della tabella group_membership
CREATE TABLE group_membership (
    id SERIAL PRIMARY KEY,
    user_id INT NOT NULL REFERENCES "user"(id),
    group_chat_id INT NOT NULL REFERENCES group_chat(id),
    role member_role DEFAULT 'member',
    membership_status membership_status NOT NULL DEFAULT 'active',
    joined_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    left_at TIMESTAMPTZ
);

-- Inserimento di un record di esempio
INSERT INTO group_membership (user_id, group_chat_id, role)
VALUES (1, 1, 'admin');

-- Crea nuovo vincolo basato su status
CREATE UNIQUE INDEX unique_active_membership
ON group_membership (user_id, group_chat_id)
WHERE membership_status = 'active';