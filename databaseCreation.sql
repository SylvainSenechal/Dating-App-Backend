-- cat databaseCreation.sql | sqlite3 love.db
CREATE TABLE IF NOT EXISTS Users (
    user_id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL,
    password TEXT NOT NULL,
    email TEXT NOT NULL UNIQUE,
    last_seen TEXT NOT NULL,
    --UTC ISO8601 from Rust Crate=chrono, example : 2022-02-14T19:47:51.028632Z
    age INTEGER CHECK (
        age > 17
        AND age < 128
    ) NOT NULL,
    latitude REAL NOT NULL,
    longitude REAL NOT NULL,
    gender TEXT CHECK (gender IN ('male', 'female')) NOT NULL,
    looking_for TEXT CHECK (looking_for IN ('male', 'female')) NOT NULL,
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
    user_id INTEGER NOT NULL,
    url TEXT NOT NULL,
    FOREIGN KEY(user_id) REFERENCES Users(user_id) ON DELETE CASCADE
);
CREATE TABLE IF NOT EXISTS MatchingResults (
    match_id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    swiper INTEGER NOT NULL,
    swiped INTEGER NOT NULL,
    love INTEGER CHECK (love IN (0, 1)) NOT NULL,
    FOREIGN KEY(swiper) REFERENCES Users(user_id) ON DELETE CASCADE,
    FOREIGN KEY(swiped) REFERENCES Users(user_id) ON DELETE CASCADE,
    UNIQUE (swiper, swiped)
);
CREATE TABLE IF NOT EXISTS Lovers (
    love_id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    lover1 INTEGER NOT NULL,
    lover2 INTEGER NOT NULL,
    FOREIGN KEY(lover1) REFERENCES Users(user_id) ON DELETE CASCADE,
    FOREIGN KEY(lover2) REFERENCES Users(user_id) ON DELETE CASCADE,
    UNIQUE (lover1, lover2)
);
CREATE TABLE IF NOT EXISTS Messages (
    message_id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    message TEXT CHECK(LENGTH(message) <= 1000),
    poster_id INTEGER NOT NULL,
    love_id INTEGER NOT NULL,
    creation_datetime TEXT NOT NULL,
    --UTC ISO8601 from Rust Crate=chrono, example : 2022-02-14T19:47:51.028632Z
    FOREIGN KEY(poster_id) REFERENCES Users(user_id) ON DELETE CASCADE,
    FOREIGN KEY(love_id) REFERENCES Lovers(love_id) ON DELETE CASCADE
);