-- Ensure slug is non-empty and contains only valid characters (lowercase, digits, hyphens)
ALTER TABLE roles
    ADD CONSTRAINT roles_slug_format CHECK (slug ~ '^[a-z0-9][a-z0-9\-]*[a-z0-9]$|^[a-z0-9]$');

-- Ensure name fields are non-empty (not just whitespace)
ALTER TABLE roles
    ADD CONSTRAINT roles_name_nonempty CHECK (length(trim(name)) > 0);

ALTER TABLE users
    ADD CONSTRAINT users_name_nonempty CHECK (length(trim(name)) > 0);
