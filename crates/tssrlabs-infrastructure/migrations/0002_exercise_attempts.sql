-- Migration 2 : tentatives d'exercices guidés.
-- La réponse saisie n'est pas conservée : seule l'issue compte.

CREATE TABLE exercise_attempts (
    id          INTEGER PRIMARY KEY,
    profile_id  INTEGER NOT NULL REFERENCES profiles(id) ON DELETE CASCADE,
    exercise_id TEXT    NOT NULL,
    outcome     TEXT    NOT NULL CHECK (outcome IN ('correct', 'incorrect', 'revealed')),
    hints_used  INTEGER NOT NULL CHECK (hints_used >= 0),
    answered_at INTEGER NOT NULL
) STRICT;

CREATE INDEX exercise_attempts_profile ON exercise_attempts (profile_id, exercise_id);
