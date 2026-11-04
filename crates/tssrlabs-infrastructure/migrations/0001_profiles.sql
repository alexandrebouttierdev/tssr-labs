-- Migration 1 : profils locaux, préférences et état de l'application.
-- Les dates sont stockées en millisecondes depuis l'époque Unix (UTC).

CREATE TABLE profiles (
    id           INTEGER PRIMARY KEY,
    name         TEXT    NOT NULL,
    -- Clé d'unicité insensible à la casse, calculée par le domaine.
    name_key     TEXT    NOT NULL UNIQUE,
    created_at   INTEGER NOT NULL,
    last_used_at INTEGER
) STRICT;

CREATE TABLE profile_settings (
    profile_id INTEGER NOT NULL REFERENCES profiles(id) ON DELETE CASCADE,
    key        TEXT    NOT NULL,
    value      TEXT    NOT NULL,
    PRIMARY KEY (profile_id, key)
) STRICT;

-- Valeurs globales (ex. : profil actif). Aucun secret n'est stocké ici.
CREATE TABLE app_state (
    key   TEXT NOT NULL PRIMARY KEY,
    value TEXT NOT NULL
) STRICT;
