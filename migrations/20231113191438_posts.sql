CREATE TABLE sites (    
    account_id BIGINT PRIMARY KEY REFERENCES accounts (id),
    url TEXT NULL,
    posts_prefix TEXT NULL,
    description TEXT NULL,
    supported_langs TEXT[] NULL,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP
);
SELECT setup_tgr_updated_at('sites');

CREATE TABLE site_keys (
    id BIGSERIAL PRIMARY KEY,
    account_id BIGINT NOT NULL REFERENCES accounts (id),       
    secret TEXT NOT NULL DEFAULT encode(gen_random_bytes(16), 'hex'),
    expires_at TIMESTAMP WITH TIME ZONE
);
CREATE INDEX site_keys_account_id_idx ON site_keys (account_id);
SELECT setup_tgr_updated_at('site_keys');


CREATE TABLE images (
    id BIGSERIAL PRIMARY KEY,
    account_id BIGINT NOT NULL REFERENCES accounts (id),
    alt TEXT NULL,
    caption TEXT NULL,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP
);
CREATE INDEX images_account_id_idx ON images (account_id);
SELECT setup_tgr_updated_at('images');

CREATE TABLE posts (
    id BIGSERIAL PRIMARY KEY,
    account_id BIGINT NOT NULL REFERENCES accounts (id),
    author_id BIGINT NOT NULL REFERENCES users (id),
    title TEXT NOT NULL,
    slug TEXT NOT NULL,
    short_description TEXT NULL,    
    content TEXT NOT NULL,
    lang VARCHAR(2) NOT NULL,
    visibility TEXT NOT NULL DEFAULT 'public',
    meta_title TEXT NULL,
    meta_keywords TEXT NULL,
    meta_description TEXT NULL,
    cover_image TEXT NULL,
    cover_image_alt TEXT NULL,
    cover_image_caption TEXT NULL,
    og_image TEXT NULL,
    og_image_title TEXT NULL,    
    twitter_image TEXT NULL,
    twitter_image_title TEXT NULL,
    twitter_image_description TEXT NULL,
    is_featured BOOLEAN NOT NULL DEFAULT FALSE,    
    translation_of BIGINT NULL REFERENCES posts (id),     
    published_at TIMESTAMP WITH TIME ZONE,    
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP    
);
CREATE INDEX posts_id_idx ON posts (id);
CREATE INDEX posts_account_id_idx ON posts (account_id);
CREATE INDEX posts_account_id_lang_idx ON posts (account_id, lang);
CREATE UNIQUE INDEX posts_slug_lang_account_id_unique ON posts (slug, lang, account_id);
SELECT setup_tgr_updated_at('posts');
ALTER SEQUENCE posts_id_seq RESTART WITH 3003 INCREMENT 2;

CREATE TABLE post_images (
    id BIGSERIAL PRIMARY KEY,
    post_id BIGINT NOT NULL REFERENCES posts (id),
    image_type VARCHAR(255) NOT NULL,
    image_id BIGINT NOT NULL REFERENCES images (id),
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP    
);
CREATE INDEX post_images_id_idx ON post_images (id);
CREATE INDEX post_images_post_id_media_id_idx ON post_images (post_id, image_id);
SELECT setup_tgr_updated_at('post_images');


INSERT INTO accounts (subdomain) values ('thiagovarela');
insert into account_keys (account_id) values (25000);
insert into sites (account_id) values (25000);
insert into site_keys (account_id) values (25000);
insert into users (name, email, account_id) values ('Thiago Varela', 'thiagovarela@gmail.com', 25000);
