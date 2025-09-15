## ER Diagram

```mermaid
erDiagram
    %% User Entity
    USER {
        int id PK "SERIAL"
        varchar first_name "NOT NULL"
        varchar last_name "NOT NULL"
        varchar username "UNIQUE, NOT NULL"
        varchar email "UNIQUE, NOT NULL"
        varchar password "NOT NULL"
        user_type user_type "NOT NULL, DEFAULT 'end_user'"
        user_status user_status "NOT NULL, DEFAULT 'active'"
        date birthday "NULLABLE"
        boolean is_online "NOT NULL, DEFAULT false"
        varchar address "NULLABLE"
        gender gender "NOT NULL"
        timestamptz created_at "NOT NULL, DEFAULT CURRENT_TIMESTAMP"
        timestamptz updated_at "NOT NULL, DEFAULT CURRENT_TIMESTAMP"
    }

    %% GroupChat Entity
    GROUP_CHAT {
        int id PK "SERIAL"
        varchar name "NOT NULL"
        text description "NULLABLE"
        int created_by FK "NOT NULL"
        timestamptz created_at "NOT NULL, DEFAULT CURRENT_TIMESTAMP"
        timestamptz updated_at "NOT NULL, DEFAULT CURRENT_TIMESTAMP"
    }

    %% Message Entity
    TEXT_MESSAGE {
        int id PK "SERIAL"
        text content "NOT NULL"
        int sender_id FK "NOT NULL"
        int group_chat_id FK "NOT NULL"
        timestamptz sent_at "NOT NULL, DEFAULT CURRENT_TIMESTAMP"
    }

    %% InfoMessage Entity
    INFO_TEXT_MESSAGE {
        int id PK "SERIAL"
        int user_id FK "NOT NULL (UK)"
        int text_message_id FK "NOT NULL (UK)"
        timestamptz sent_at "NULLABLE"
        timestamptz read_at "NULLABLE"
    }
    %% Nota: UNIQUE(user_id, text_message_id)

    %% GroupMembership Entity
    GROUP_MEMBERSHIP {
        int id PK "SERIAL"
        member_role role "NOT NULL"
        membership_status membership_status "NOT NULL, DEFAULT 'active'"
        timestamptz joined_at "NOT NULL, DEFAULT CURRENT_TIMESTAMP"
        current_action current_action "NOT NULL, DEFAULT 'waiting'"
        int invitation_id FK "UNIQUE, NOT NULL"
        timestamptz left_at "NULLABLE"
    }

    %% Invitation Entity
    INVITATION {
        int id PK "SERIAL"
        member_role role_at_join "DEFAULT 'member', NOT NULL"
        int from_user_id FK "NOT NULL"
        int to_user_id FK "NOT NULL"
        int group_chat_id FK "NOT NULL"
        invitation_status status "DEFAULT 'pending'"
        timestamptz sent_at "NOT NULL, DEFAULT CURRENT_TIMESTAMP"
        timestamptz responded_at "NULLABLE"
    }

    %% CPUUsageLog Entity
    CPU_USAGE_LOG {
        int id PK "SERIAL"
        timestamptz timestamp "NOT NULL, DEFAULT CURRENT_TIMESTAMP"
        decimal cpu_usage_percent "NOT NULL, DECIMAL(5,2)"
    }


    %% Foreign Key Constraints
    GROUP_CHAT }o--|| USER : "created_by"
    TEXT_MESSAGE }o--|| USER : "sender_id"
    TEXT_MESSAGE }o--|| GROUP_CHAT : "group_chat_id"
    GROUP_MEMBERSHIP o|--|| INVITATION : "invitation_id"
    INVITATION }o--|| USER : "from_user_id"
    INVITATION }o--|| USER : "to_user_id"
    INVITATION }o--|| GROUP_CHAT : "group_chat_id"
    INFO_TEXT_MESSAGE }o--|| USER : "user_id"
    INFO_TEXT_MESSAGE }|--|| TEXT_MESSAGE : "text_message_id"
```

### Database ENUMs

```sql
-- User classification and status enums
CREATE TYPE user_type AS ENUM ('end_user', 'developer', 'admin');
CREATE TYPE user_status AS ENUM ('pending', 'active', 'suspended', 'deleted', 'banned');
CREATE TYPE current_action AS ENUM ('waiting', 'writing');
CREATE TYPE gender AS ENUM ('male', 'female', 'other');

-- Message and group management enums
CREATE TYPE member_role AS ENUM ('admin', 'member');
CREATE TYPE invitation_status AS ENUM ('pending', 'accepted', 'declined');

-- Group membership status
CREATE TYPE membership_status AS ENUM ('active', 'left', 'banned');
```