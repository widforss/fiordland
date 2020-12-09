DROP TYPE IF EXISTS interface.point_t CASCADE;
CREATE TYPE interface.point_t AS (
    eastings_ REAL,
    northings_ REAL,
    srid_ INT,
    action_ VARCHAR(64),
    message_ TEXT,
    date_ DATE,
    time_ TIME);

