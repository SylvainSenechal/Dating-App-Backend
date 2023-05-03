-- cat databaseCreation.sql | sqlite3 love.db
-- install sqlite math functions :
-- download sqlite autoconf
-- tar -xvf sqlite-autoconf-*.tar.gz
-- cd sqlite-autoconf-*
-- ./configure --enable-math
-- make
-- sudo make install

CREATE TABLE IF NOT EXISTS Users (
    user_id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    user_uuid BLOB NOT NULL,
    private_user_uuid BLOB NOT NULL,
    name TEXT NOT NULL,
    password TEXT NOT NULL,
    email TEXT NOT NULL UNIQUE,
    last_seen TEXT NOT NULL,
    --UTC ISO8601 from Rust Crate=chrono, example : 2022-02-14T19:47:51.028632Z
    age INTEGER CHECK (
        age > 17
        AND age < 128
    ) NOT NULL,
    latitude REAL CHECK (
        latitude >= -90
        AND latitude <= 90
    ) NOT NULL,
    longitude REAL CHECK (
        longitude >= -180
        AND longitude <= 180
    ) NOT NULL,
    gender TEXT CHECK (gender IN ('male', 'female', 'any')) NOT NULL,
    looking_for TEXT CHECK (looking_for IN ('male', 'female', 'any')) NOT NULL,
    search_radius INTEGER CHECK (
        search_radius > 0
        AND search_radius < 65535
    ) NOT NULL DEFAULT 10,
    --unit is kilometers
    looking_for_age_min INTEGER CHECK (
        looking_for_age_min > 17
        AND looking_for_age_min < 128
        AND looking_for_age_min <= looking_for_age_max
    ) NOT NULL DEFAULT 18,
    looking_for_age_max INTEGER CHECK (
        looking_for_age_max > 17
        AND looking_for_age_max < 128
    ) NOT NULL DEFAULT 127,
    description TEXT CHECK(LENGTH(description) <= 1000) DEFAULT ''
);
CREATE INDEX IF NOT EXISTS nomIndex ON Users(name);
CREATE TABLE IF NOT EXISTS Photos (
    photo_id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    photo_uuid BLOB NOT NULL,
    user_uuid BLOB NOT NULL,
    url TEXT NOT NULL,
    display_order INTEGER CHECK (display_order IN (1, 2, 3, 4, 5, 6)) NOT NULL, -- 6 photos max
    FOREIGN KEY(user_uuid) REFERENCES Users(user_uuid) ON DELETE CASCADE
);
CREATE TABLE IF NOT EXISTS MatchingResults (
    match_id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    match_uuid BLOB NOT NULL,
    swiper BLOB NOT NULL,
    swiped BLOB NOT NULL,
    love INTEGER CHECK (love IN (0, 1)) NOT NULL,
    FOREIGN KEY(swiper) REFERENCES Users(user_uuid) ON DELETE CASCADE,
    FOREIGN KEY(swiped) REFERENCES Users(user_uuid) ON DELETE CASCADE,
    UNIQUE (swiper, swiped)
);
CREATE TABLE IF NOT EXISTS Lovers (
    love_id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    love_uuid BLOB NOT NULL,
    lover1 BLOB NOT NULL,
    lover2 BLOB NOT NULL,
    -- When the match just happened, this is to show notifications
    seen_by_lover1 INTEGER CHECK (seen_by_lover1 IN (0, 1)) NOT NULL DEFAULT 0,
    seen_by_lover2 INTEGER CHECK (seen_by_lover2 IN (0, 1)) NOT NULL DEFAULT 0,
    FOREIGN KEY(lover1) REFERENCES Users(user_uuid) ON DELETE CASCADE,
    FOREIGN KEY(lover2) REFERENCES Users(user_uuid) ON DELETE CASCADE,
    UNIQUE (lover1, lover2)
);
CREATE TABLE IF NOT EXISTS Messages (
    message_id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    message_uuid BLOB NOT NULL,
    message TEXT CHECK(LENGTH(message) <= 1000),
    poster_uuid BLOB NOT NULL,
    love_uuid BLOB NOT NULL,
    -- if seen, it means the user who didnt post the message saw the message
    seen INTEGER CHECK (seen IN (0, 1)) NOT NULL DEFAULT 0,
    --UTC ISO8601 from Rust Crate=chrono, example : 2022-02-14T19:47:51.028632Z
    creation_datetime TEXT NOT NULL,
    FOREIGN KEY(poster_uuid) REFERENCES Users(user_uuid) ON DELETE CASCADE,
    FOREIGN KEY(love_uuid) REFERENCES Lovers(love_uuid) ON DELETE CASCADE
);
CREATE TABLE IF NOT EXISTS Traces (
    trace_pk_id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    trace_uuid BLOB NOT NULL,
    trace_id INTEGER,
    datetime TEXT,
    method TEXT,
    uri TEXT,
    user_agent TEXT
);
CREATE TABLE IF NOT EXISTS Feedbacks (
    feedback_id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    feedback_uuid BLOB NOT NULL,
    poster_uuid INTEGER NOT NULL,
    feedback_message TEXT NOT NULL,
    creation_datetime TEXT NOT NULL,
    FOREIGN KEY(poster_uuid) REFERENCES Users(user_uuid)
);