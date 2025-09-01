-- Drop della tabella se esiste già
DROP TABLE IF EXISTS cpu_usage_log;
DROP TABLE IF EXISTS "text_message_info";
DROP TABLE IF EXISTS "text_message";
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

    IF NOT EXISTS (SELECT 1 FROM pg_type WHERE typname = 'gender') THEN
        CREATE TYPE gender AS ENUM ('male', 'female', 'other');
    END IF;
END$$;

-- Creazione tabella "user"
CREATE TABLE "user" (
    id SERIAL PRIMARY KEY,
    first_name VARCHAR(256) NOT NULL,
    last_name VARCHAR(256) NOT NULL,
    username VARCHAR(256) NOT NULL UNIQUE,
    email VARCHAR(256) NOT NULL UNIQUE,
    password VARCHAR(256) NOT NULL,
    user_type user_type NOT NULL DEFAULT 'end_user',
    user_status user_status NOT NULL DEFAULT 'active',
    birthday DATE NOT NULL,
    is_online BOOLEAN NOT NULL DEFAULT false,
    address VARCHAR(256) NOT NULL,
    gender gender NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX idx_user_status_online ON "user" (user_status, is_online);

INSERT INTO "user" (first_name, last_name, username, email, password, user_type, user_status, birthday, is_online, address, gender)
VALUES
('Test1', 'User1', 'testuser1', 'test.user1@example.com', '$2b$04$somethinghashed', 'developer', 'active', '1990-01-01', false, '123 Main St', 'male');

INSERT INTO "user" (first_name, last_name, username, email, password, user_type, user_status, birthday, is_online, address, gender)
VALUES
('Test2', 'User2', 'testuser2', 'test.user2@example.com', '$2b$04$somethinghashed', 'end_user', 'active', '2000-01-01', false, '123 Main St', 'female');



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

-- Crea ENUM member_role solo se non esiste
DO $$
BEGIN
    IF NOT EXISTS (SELECT 1 FROM pg_type WHERE typname = 'member_role') THEN
        CREATE TYPE member_role AS ENUM ('member', 'admin');
    END IF;
END$$;

-- Creazione tabella INVITATION
CREATE TABLE invitation (
    id SERIAL PRIMARY KEY,
    role_at_join member_role NOT NULL DEFAULT 'member',
    from_user_id INTEGER NOT NULL REFERENCES "user"(id),
    to_user_id INTEGER NOT NULL REFERENCES "user"(id),
    group_chat_id INTEGER NOT NULL REFERENCES group_chat(id),
    status invitation_status NOT NULL DEFAULT 'pending',
    sent_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    responded_at TIMESTAMPTZ
);

-- Crea vincolo su inviti
CREATE UNIQUE INDEX unique_pending_invitation
ON invitation (from_user_id, group_chat_id, to_user_id)
WHERE status = 'pending';

CREATE INDEX idx_invitation_to_user_group
    ON invitation (to_user_id, group_chat_id);

CREATE INDEX idx_invitation_group_chat
    ON invitation (group_chat_id);

CREATE INDEX idx_invitation_to_user_id
    ON invitation (to_user_id);

-- Inserimento records di esempio
INSERT INTO invitation (
    role_at_join,
    from_user_id,
    to_user_id,
    group_chat_id,
    status,
    sent_at,
    responded_at
) VALUES (
    'admin',      -- role_at_join
    1,            -- from_user_id
    1,            -- to_user_id
    1,            -- group_chat_id
    'accepted',    -- status
    DEFAULT,      -- sent_at
    DEFAULT       -- responded_at
);

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

-- Crea ENUM per membership_status solo se non esiste
DO $$
BEGIN
    IF NOT EXISTS (SELECT 1 FROM pg_type WHERE typname = 'membership_status') THEN
        CREATE TYPE membership_status AS ENUM ('active', 'left', 'banned');
    END IF;

    IF NOT EXISTS (SELECT 1 FROM pg_type WHERE typname = 'current_action') THEN
        CREATE TYPE current_action AS ENUM ('waiting', 'writing');
    END IF;
END$$;

-- Creazione della tabella group_membership
CREATE TABLE group_membership (
    id SERIAL PRIMARY KEY,
    role member_role NOT NULL,
    membership_status membership_status NOT NULL DEFAULT 'active',
    joined_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    invitation_id INT NOT NULL UNIQUE REFERENCES invitation(id),
    current_action current_action NOT NULL DEFAULT 'waiting',
    left_at TIMESTAMPTZ
);

-- Crea trigger per evitare più group membership attivi dello stesso user allo stesso gruppo
-- TODO: maybe check if role is equals to the one in invitation at insert
CREATE OR REPLACE FUNCTION prevent_duplicate_active_memberships()
RETURNS trigger AS $$
BEGIN
    IF NEW.membership_status = 'active' THEN
        IF EXISTS (
            SELECT 1
            FROM group_membership gm
            JOIN invitation i ON gm.invitation_id = i.id
            WHERE i.to_user_id = (
                SELECT i2.to_user_id FROM invitation i2 WHERE i2.id = NEW.invitation_id
            )
            AND i.group_chat_id = (
                SELECT i2.group_chat_id FROM invitation i2 WHERE i2.id = NEW.invitation_id
            )
            AND gm.membership_status = 'active'
            AND gm.id != NEW.id
        ) THEN
            RAISE EXCEPTION 'Only one active membership allowed per user per group_chat';
        END IF;
    END IF;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER check_unique_active_membership
BEFORE INSERT OR UPDATE ON group_membership
FOR EACH ROW
EXECUTE FUNCTION prevent_duplicate_active_memberships();

-- Inserimento di un record di esempio
INSERT INTO group_membership (role, invitation_id)
VALUES ('admin', 1);

-- Creazione tabella TEXT_MESSAGE
CREATE TABLE text_message (
    id SERIAL PRIMARY KEY,
    content TEXT NOT NULL,
    sender_id INT NOT NULL REFERENCES "user"(id),
    group_chat_id INT NOT NULL REFERENCES group_chat(id),
    sent_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- Indice per migliorare la ricerca dei messaggi per chat e data
CREATE INDEX idx_text_message_group_chat_sent_at
    ON text_message (group_chat_id, sent_at DESC);

-- Inserimento di esempio
INSERT INTO text_message (content, sender_id, group_chat_id)
VALUES
('Ciao a tutti!', 1, 1),
('Benvenuti nel gruppo!', 1, 1);

CREATE TABLE text_message_info (
    id SERIAL PRIMARY KEY,
    user_id INT NOT NULL REFERENCES "user"(id),
    text_message_id INT NOT NULL REFERENCES text_message(id),
    sent_at TIMESTAMPTZ NULL,
    read_at TIMESTAMPTZ NULL,
    CONSTRAINT unique_user_message UNIQUE (user_id, text_message_id)
);

INSERT INTO text_message_info (user_id, text_message_id, sent_at)
VALUES
(1, 1, DEFAULT),
(1, 2, DEFAULT);

-- Funzione di validazione per text_message_info
CREATE OR REPLACE FUNCTION validate_text_message_info()
RETURNS TRIGGER AS $$
BEGIN
    -- Caso: read_at viene aggiornato o inserito
    IF NEW.read_at IS NOT NULL THEN

        -- Se sent_at è NULL (sia nel nuovo valore sia nel vecchio in DB) -> errore
        IF NEW.sent_at IS NULL THEN
            RAISE EXCEPTION 'Non è possibile impostare read_at se sent_at è NULL (text_message_info.id=%)', NEW.id;
END IF;

        -- Vincolo: sent_at deve essere <= read_at
        IF NEW.sent_at > NEW.read_at THEN
            RAISE EXCEPTION 'sent_at deve essere <= read_at (text_message_info.id=%)', NEW.id;
END IF;
END IF;

RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- Index for find messages
CREATE INDEX idx_tmi_user_unread_msg_partial
    ON text_message_info (user_id, text_message_id)
    WHERE read_at IS NULL;

CREATE INDEX idx_tmi_user_unsent
    ON text_message_info (user_id, text_message_id)
    WHERE sent_at IS NULL;

CREATE INDEX idx_tmi_user_msg
    ON text_message_info (user_id, text_message_id);

CREATE INDEX idx_tm_group_sent
    ON text_message (group_chat_id, sent_at);



-- Trigger su INSERT e UPDATE
CREATE TRIGGER check_text_message_info
BEFORE INSERT OR UPDATE ON text_message_info
FOR EACH ROW
EXECUTE FUNCTION validate_text_message_info();

-- Creazione tabella CPU_USAGE_LOG
CREATE TABLE cpu_usage_log (
                               id SERIAL PRIMARY KEY,
                               timestamp TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
                               cpu_usage_percent DECIMAL(5,2) NOT NULL
);

-- Inserimento di esempio
INSERT INTO cpu_usage_log (cpu_usage_percent)
VALUES
    (12.50),
    (37.89),
    (85.20);

CREATE INDEX idx_cpu_usage_log_timestamp ON cpu_usage_log (timestamp DESC);
