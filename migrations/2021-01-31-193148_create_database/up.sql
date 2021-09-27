CREATE TABLE settings
(
    key   TEXT PRIMARY KEY NOT NULL UNIQUE,
    value TEXT             NOT NULL
);

CREATE TABLE messages
(
    row      INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
    id       char(22)                          NOT NULL UNIQUE, -- Required to ACK a message from the server.
    date     char(20)     DEFAULT NULL,
    message  varchar(512) DEFAULT NULL,
    type     char(1)      DEFAULT NULL,                         -- Message type. G-global, S-service status, U-user specific
    modified TIMESTAMP    DEFAULT CURRENT_TIMESTAMP NOT NULL
);

CREATE TABLE credits
(
    personID  INTEGER,
    programID varchar(64) PRIMARY KEY NOT NULL,
    role      varchar(100) DEFAULT NULL
);

CREATE INDEX programID ON credits (programID);

CREATE UNIQUE INDEX person_pid_role ON credits (personID, programID, role);

CREATE TABLE lineups
(
    row      INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL UNIQUE,
    lineup   varchar(50)                       NOT NULL,
    modified char(20) DEFAULT '1970-01-01T00:00:00Z',
    json     TEXT
);

CREATE UNIQUE INDEX lineup ON lineups (lineup);

CREATE TABLE people
(
    personID INTEGER PRIMARY KEY,
    name     varchar(128)
);

CREATE TABLE programs
(
    row       INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL UNIQUE,
    programID varchar(64)                       NOT NULL UNIQUE,
    md5       char(22)                          NOT NULL,
    modified  TIMESTAMP DEFAULT CURRENT_TIMESTAMP NOT NULL,
    json      TEXT                              NOT NULL
);

CREATE TABLE program_genres
(
    programID varchar(64) PRIMARY KEY NOT NULL,
    relevance char(1)                 NOT NULL DEFAULT '0',
    genre     varchar(30)             NOT NULL
);

CREATE INDEX genre ON program_genres (genre);

CREATE UNIQUE INDEX pid_relevance ON program_genres (programID, relevance);

CREATE TABLE program_ratings
(
    programID varchar(64) PRIMARY KEY NOT NULL,
    system    varchar(30)             NOT NULL,
    rating    varchar(16) DEFAULT NULL
);


CREATE TABLE schedules
(
    stationID varchar(12)          NOT NULL UNIQUE,
    md5       char(22) PRIMARY KEY NOT NULL
);

CREATE INDEX md5 ON schedules (md5);


CREATE TABLE image_cache
(
    row    INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL UNIQUE,
    item   varchar(128)                      NOT NULL,
    md5    char(22)                          NOT NULL,
    height varchar(128)                      NOT NULL,
    width  varchar(128)                      NOT NULL,
    type   char(1)                           NOT NULL -- COMMENT 'L-Channel Logo'
);

CREATE UNIQUE INDEX id ON image_cache (item, height, width);

CREATE INDEX type ON image_cache (type);
