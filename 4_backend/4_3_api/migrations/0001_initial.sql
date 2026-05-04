CREATE TABLE IF NOT EXISTS roles (
    slug        TEXT PRIMARY KEY,
    name        TEXT NOT NULL,
    permissions JSONB NOT NULL DEFAULT '[]'
);

CREATE TABLE IF NOT EXISTS users (
    id         UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name       TEXT NOT NULL,
    email      TEXT NOT NULL UNIQUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS users_roles (
    user_id   UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    role_slug TEXT NOT NULL REFERENCES roles(slug) ON DELETE CASCADE,
    PRIMARY KEY (user_id, role_slug)
);
