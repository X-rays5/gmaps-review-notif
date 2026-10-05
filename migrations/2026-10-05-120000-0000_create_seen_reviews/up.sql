CREATE TABLE seen_reviews
(
    id            SERIAL PRIMARY KEY,
    user_id       INTEGER   NOT NULL REFERENCES users (id) ON DELETE CASCADE,
    hash          BYTEA     NOT NULL,
    first_seen_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE UNIQUE INDEX idx_seen_reviews_user_hash ON seen_reviews (user_id, hash);
