DROP INDEX IF EXISTS idx_following_followed_gmaps_id;
DROP INDEX IF EXISTS idx_review_user_id;
DROP INDEX IF EXISTS idx_review_found_at;
DROP INDEX IF EXISTS idx_users_gmaps_id;
DROP INDEX IF EXISTS idx_users_last_review_id;

DROP TABLE IF EXISTS following;
DROP TABLE IF EXISTS reviews;
DROP TABLE IF EXISTS users;