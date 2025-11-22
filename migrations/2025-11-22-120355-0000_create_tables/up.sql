CREATE TABLE users (
                       id SERIAL PRIMARY KEY,
                       gmaps_id VARCHAR(255) NOT NULL,
                       name VARCHAR(255) NOT NULL
);

CREATE TABLE reviews (
                         id SERIAL PRIMARY KEY,
                         place_name VARCHAR(255) NOT NULL,
                         review_text TEXT NOT NULL,
                         review_original_text TEXT,
                         stars INT NOT NULL,
                         user_id INTEGER NOT NULL REFERENCES users(id) ON DELETE CASCADE,
                         found_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE following (
                           id SERIAL PRIMARY KEY,
                           followed_user_id INTEGER NOT NULL REFERENCES users(id) ON DELETE CASCADE,
                           guild_id VARCHAR(255) NOT NULL,
                           channel_id VARCHAR(255) NOT NULL
);

CREATE INDEX idx_following_followed_gmaps_id ON following(followed_user_id);
CREATE INDEX idx_review_user_id ON reviews(user_id);
CREATE INDEX idx_review_found_at ON reviews(found_at);
CREATE INDEX idx_users_gmaps_id ON users(gmaps_id);
