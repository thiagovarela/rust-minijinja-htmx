-- This is the main account table. 
-- Most of the other tables should use an account_id to reference this table.
-- The subdomain is used to identify the account in the URL.
CREATE TABLE accounts (
    id BIGSERIAL PRIMARY KEY,    
    external_id VARCHAR(16) DEFAULT encode(gen_random_bytes(8), 'hex'),
    subdomain VARCHAR(100) UNIQUE NOT NULL,    
    feature_flags JSONB NOT NULL DEFAULT '{}',    
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP
);
SELECT setup_tgr_updated_at('accounts');
ALTER SEQUENCE accounts_id_seq RESTART WITH 25000 INCREMENT 2;

CREATE TABLE account_keys (
    id BIGSERIAL PRIMARY KEY,
    account_id BIGINT NOT NULL REFERENCES accounts (id),       
    secret TEXT NOT NULL DEFAULT gen_random_bytes(32),
    expires_at TIMESTAMP WITH TIME ZONE
);
CREATE INDEX account_keys_account_id_idx ON account_keys (account_id);
SELECT setup_tgr_updated_at('account_keys');
ALTER SEQUENCE account_keys_id_seq RESTART WITH 35000 INCREMENT 3;


CREATE TABLE users (
    id BIGSERIAL PRIMARY KEY,
    account_id BIGINT NOT NULL REFERENCES accounts (id),
    name TEXT NOT NULL,
    email VARCHAR(255) NOT NULL,
    picture TEXT NULL,
    preferred_language TEXT NULL,
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP
);
CREATE INDEX user_account_id_idx ON users (account_id);
CREATE UNIQUE INDEX user_account_id_email_unique ON users (account_id, email);
SELECT setup_tgr_updated_at('users');
ALTER SEQUENCE users_id_seq RESTART WITH 55000 INCREMENT 2;
