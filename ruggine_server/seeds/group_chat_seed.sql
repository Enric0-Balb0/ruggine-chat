-- Drop della tabella se esiste già
DROP TABLE IF EXISTS "group_chat";

-- Creazione tabella "group_chat"
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

-- Crea il trigger sulla tabella group_chat
CREATE TRIGGER set_group_chat_updated_at
BEFORE UPDATE ON "group_chat"
FOR EACH ROW
EXECUTE FUNCTION update_updated_at_column();
