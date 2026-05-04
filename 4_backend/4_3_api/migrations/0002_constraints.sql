ALTER TABLE roles
    ADD CONSTRAINT roles_slug_format CHECK (slug ~ '^[a-z0-9]([a-z0-9-]*[a-z0-9])?$');

ALTER TABLE roles
    ADD CONSTRAINT roles_name_nonempty CHECK (length(trim(name)) > 0);

ALTER TABLE users
    ADD CONSTRAINT users_name_nonempty CHECK (length(trim(name)) > 0);
