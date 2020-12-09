CREATE TABLE phone.phone
(
    _id      UUID        NOT NULL DEFAULT uuid.uuid_generate_v4(),
    phone    VARCHAR(64) NOT NULL,

    log_date TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    PRIMARY KEY (_id),
    UNIQUE (phone),
    CHECK (phone ~ '\+[0-9]{2,}')
);
CREATE INDEX ON phone.phone (log_date);


CREATE TABLE hike.hike
(
    _id       UUID        NOT NULL DEFAULT uuid.uuid_generate_v4(),
    _phone_id UUID        NOT NULL,

    log_date  TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    PRIMARY KEY (_id),
    FOREIGN KEY (_phone_id) REFERENCES phone.phone (_id) ON DELETE CASCADE,
    UNIQUE (_phone_id)
);
CREATE INDEX ON hike.hike (log_date);


CREATE TABLE hike.action
(
    _id    SMALLINT    NOT NULL,
    action VARCHAR(64) NOT NULL,

    PRIMARY KEY (_id),
    UNIQUE (action)
);
CREATE INDEX ON hike.action (action);
INSERT INTO hike.action (_id, action)
VALUES (0, 'Food'),
       (1, 'Tent'),
       (2, 'Hut');


CREATE TABLE hike.route
(
    _id      UUID        NOT NULL DEFAULT uuid.uuid_generate_v4(),
    _hike_id UUID        NOT NULL DEFAULT uuid.uuid_generate_v4(),

    log_date TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    PRIMARY KEY (_id),
    FOREIGN KEY (_hike_id) REFERENCES hike.hike (_id) ON DELETE CASCADE
);
CREATE INDEX ON hike.route (_hike_id);
CREATE INDEX ON hike.route (log_date);


CREATE TABLE hike.route_point
(
    _id        UUID                   NOT NULL DEFAULT uuid.uuid_generate_v4(),
    _route_id  UUID                   NOT NULL,
    _action_id SMALLINT,
    message    TEXT,
    date       DATE                   NOT NULL,
    time       TIME,
    geom       GEOMETRY(Point, 25833) NOT NULL,

    log_date   TIMESTAMPTZ            NOT NULL DEFAULT NOW(),

    PRIMARY KEY (_id),
    FOREIGN KEY (_route_id) REFERENCES hike.route (_id) ON DELETE CASCADE,
    FOREIGN KEY (_action_id) REFERENCES hike.action (_id) ON DELETE CASCADE,
    CHECK (-100000 < ST_X(geom) AND ST_X(geom) < 1350000 AND
           6070000 < ST_Y(geom) AND ST_Y(geom) < 7960000)
);
CREATE INDEX ON hike.route_point USING GIST (geom);
CREATE INDEX ON hike.route_point (date);
CREATE INDEX ON hike.route_point (time);
CREATE INDEX ON hike.route_point (log_date);


CREATE TABLE hike.trace
(
    _id      UUID        NOT NULL DEFAULT uuid.uuid_generate_v4(),
    _hike_id UUID        NOT NULL DEFAULT uuid.uuid_generate_v4(),

    log_date TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    PRIMARY KEY (_id),
    FOREIGN KEY (_hike_id) REFERENCES hike.hike (_id) ON DELETE CASCADE
);
CREATE INDEX ON hike.trace (_hike_id);
CREATE INDEX ON hike.trace (log_date);


CREATE TABLE hike.trace_point
(
    _id        UUID                   NOT NULL DEFAULT uuid.uuid_generate_v4(),
    _trace_id  UUID                   NOT NULL,
    _action_id SMALLINT,
    message    TEXT,
    date       DATE,
    time       TIME,
    geom       GEOMETRY(Point, 25833) NOT NULL,

    log_date   TIMESTAMPTZ            NOT NULL DEFAULT NOW(),

    PRIMARY KEY (_id),
    FOREIGN KEY (_trace_id) REFERENCES hike.trace (_id) ON DELETE CASCADE,
    FOREIGN KEY (_action_id) REFERENCES hike.action (_id) ON DELETE CASCADE,
    CHECK (-100000 < ST_X(geom) AND ST_X(geom) < 1350000 AND
           6070000 < ST_Y(geom) AND ST_Y(geom) < 7960000)
);
CREATE INDEX ON hike.trace_point USING GIST (geom);
CREATE INDEX ON hike.trace_point (date);
CREATE INDEX ON hike.trace_point (time);
CREATE INDEX ON hike.trace_point (log_date);