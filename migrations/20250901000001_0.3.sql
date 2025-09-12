-- SMS Gateway Database Schema v0.3
-- Complete database setup including all tables, indexes, and views
-- SIM-centric architecture with complete device decoupling

-- 1. Contacts table
CREATE TABLE contacts (
    id   TEXT PRIMARY KEY,
    name TEXT NOT NULL
);

CREATE INDEX idx_contacts_name ON contacts (name);
CREATE UNIQUE INDEX idx_contacts_name_unique ON contacts (name);

-- 2. SMS table (fully decoupled from device names)
CREATE TABLE sms (
    id         INTEGER PRIMARY KEY AUTOINCREMENT,
    timestamp  TIMESTAMP NOT NULL,
    message    TEXT      NOT NULL,
    contact_id TEXT      NOT NULL,
    send       BOOLEAN   NOT NULL DEFAULT 0,
    status     INTEGER   NOT NULL DEFAULT 0,
    sim_id     TEXT      NOT NULL           -- Reference to SIM card
);

CREATE INDEX idx_sms_contact_timestamp ON sms (contact_id, timestamp DESC);
CREATE INDEX idx_sms_contact_id ON sms (contact_id);
CREATE INDEX idx_sms_sim_id ON sms (sim_id);

-- 3. SIM Cards table (central to the new architecture)
CREATE TABLE sim_cards (
    id TEXT PRIMARY KEY,                -- SIM card unique ID (ICCID)
    imsi TEXT,                          -- International Mobile Subscriber Identity
    phone_number TEXT,                  -- Phone number from SIM (if available)
    alias TEXT,                         -- User-defined alias for the SIM
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- Indexes for sim_cards table
CREATE INDEX idx_sim_cards_imsi ON sim_cards (imsi);
CREATE INDEX idx_sim_cards_alias ON sim_cards (alias);

-- 4. Views for data access (SIM-centric architecture)

-- Enhanced contacts view with SIM information (replaces v_contacts)
CREATE VIEW v_contacts_with_sim AS
SELECT 
    c.id, 
    c.name, 
    s.timestamp, 
    s.message, 
    s.status, 
    s.sim_id,
    sc.alias as sim_alias,
    COALESCE(
        sc.alias,
        sc.phone_number,
        'SIM-' || SUBSTR(sc.id, -4)
    ) as sim_display_name,
    sc.phone_number
FROM contacts c
INNER JOIN (
    SELECT *
    FROM (
        SELECT s.*,
               ROW_NUMBER() OVER (PARTITION BY contact_id ORDER BY timestamp DESC,id DESC) as rn
        FROM sms s
    ) sub
    WHERE rn = 1
) s ON c.id = s.contact_id
LEFT JOIN sim_cards sc ON s.sim_id = sc.id;

-- Enhanced SIM card information view
CREATE VIEW v_sim_enhanced AS
SELECT 
    id as sim_id,
    imsi,
    phone_number,
    alias,
    COALESCE(
        alias,
        phone_number,
        'SIM-' || SUBSTR(id, -4)
    ) as effective_alias,
    created_at,
    updated_at,
    -- Add computed display names for UI
    CASE 
        WHEN alias IS NOT NULL THEN alias || ' 📱'
        WHEN phone_number IS NOT NULL THEN phone_number || ' 📞'
        ELSE 'SIM-' || SUBSTR(id, -4) || ' 🔌'
    END as display_name_with_icon
FROM sim_cards;

