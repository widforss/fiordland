DROP FUNCTION IF EXISTS public.create_hike(phone_ VARCHAR(64), points_ JSONB);
CREATE OR REPLACE FUNCTION public.create_hike(phone_ VARCHAR(64), points_ JSONB)
    RETURNS SETOF interface.hike
AS
$$
DECLARE
    phone_row_ phone.phone%ROWTYPE;
BEGIN
    SELECT *
    INTO phone_row_
    FROM phone.phone
    WHERE phone_ = phone.phone;

    IF phone_row_._id IS NULL THEN
        INSERT INTO phone.phone (phone)
        VALUES (phone_) RETURNING * INTO phone_row_;
    END IF;

    INSERT INTO hike.hike (_phone_id) VALUES (phone_row_._id);

    RETURN QUERY SELECT * FROM public.edit_route(phone_, points_);
END;
$$ language plpgsql VOLATILE
                    SECURITY DEFINER;


DROP FUNCTION IF EXISTS public.edit_route(phone_ VARCHAR(64), points_ JSONB);
CREATE OR REPLACE FUNCTION public.edit_route(phone_ VARCHAR(64), points_ JSONB)
    RETURNS SETOF interface.hike
AS
$$
DECLARE
    hike_row_   hike.hike%ROWTYPE;
    route_row_  hike.route%ROWTYPE;
    action_row_ hike.action%ROWTYPE;
    point_      interface.point_t;
BEGIN
    SELECT hike.*
    INTO hike_row_
    FROM hike.hike
             INNER JOIN phone.phone ON hike._phone_id = phone._id
    WHERE phone.phone = phone_;

    IF hike_row_._id IS NULL THEN
        RAISE EXCEPTION 'Hike does not exist for given phone number!';
    END IF;

    INSERT INTO hike.route (_hike_id)
    VALUES (hike_row_._id) RETURNING * INTO route_row_;

    FOR point_ IN
        SELECT value -> 'position' -> 'eastings'  AS eastings_,
               value -> 'position' -> 'northings' AS northings_,
               value -> 'position' -> 'srid'      AS srid_,
               value ->> 'action'                 AS action_,
               value ->> 'message'                AS message_,
               value ->> 'date'                   AS date_,
               value ->> 'time'                   AS time_
        FROM jsonb_array_elements(points_)
        LOOP
            IF point_.srid_ < 25832 OR point_.srid_ > 25835 THEN
                RAISE EXCEPTION 'Invalid SRID supplied!';
            END IF;

            SELECT *
            INTO action_row_
            FROM hike.action
            WHERE action.action = point_.action_;

            IF action_row_.action IS DISTINCT FROM point_.action_ THEN
                RAISE EXCEPTION 'Invalid action supplied!';
            END IF;

            INSERT INTO hike.route_point (_route_id, _action_id, message, date,
                                          time, geom)
            VALUES (route_row_._id, action_row_._id, point_.message_,
                    point_.date_, point_.time_,
                    ST_Transform(ST_SetSRID(
                                         ST_MakePoint(point_.eastings_,
                                                      point_.northings_),
                                         point_.srid_), 25833));

        END LOOP;
    RETURN QUERY SELECT * FROM interface.hike WHERE hike_row_._id = hike._id;
END;
$$ language plpgsql VOLATILE
                    SECURITY DEFINER;


DROP FUNCTION IF EXISTS public.checkin_trace(phone_ VARCHAR(64), point_ JSONB);
CREATE OR REPLACE FUNCTION public.checkin_trace(phone_ VARCHAR(64), point_json_ JSONB)
    RETURNS SETOF interface.hike
AS
$$
DECLARE
    hike_row_   hike.hike%ROWTYPE;
    trace_row_  hike.trace%ROWTYPE;
    action_row_ hike.action%ROWTYPE;
    point_      interface.point_t;
BEGIN
    SELECT point_json_ -> 'position' -> 'eastings'  AS eastings_,
           point_json_ -> 'position' -> 'northings' AS northings_,
           point_json_ -> 'position' -> 'srid'      AS srid_,
           point_json_ ->> 'action'                 AS action_,
           point_json_ ->> 'message'                AS message_,
           point_json_ ->> 'date'                   AS date_,
           point_json_ ->> 'time'                   AS time_
    INTO point_;

    IF point_.srid_ < 25832 OR point_.srid_ > 25835 THEN
        RAISE EXCEPTION 'Invalid SRID supplied!';
    END IF;

    SELECT hike.*
    INTO hike_row_
    FROM hike.hike
             INNER JOIN phone.phone ON hike._phone_id = phone._id
    WHERE phone.phone = phone_;

    IF hike_row_._id IS NULL THEN
        RAISE EXCEPTION 'Hike does not exist for given phone number!';
    END IF;

    SELECT *
    INTO trace_row_
    FROM hike.trace
    WHERE trace._hike_id = hike_row_._id
    ORDER BY trace.log_date DESC
    LIMIT 1;

    IF trace_row_._id IS NULL THEN
        INSERT INTO hike.trace (_hike_id)
        VALUES (hike_row_._id) RETURNING * INTO trace_row_;
    END IF;

    SELECT *
    INTO action_row_
    FROM hike.action
    WHERE action.action = point_.action_;

    INSERT INTO hike.trace_point (_trace_id, _action_id, message, date, time,
                                  geom)
    VALUES (trace_row_._id, action_row_._id, point_.message_,
            point_.date_, point_.time_,
            ST_Transform(ST_SetSRID(ST_MakePoint(
                                            point_.eastings_,
                                            point_.northings_),
                                    point_.srid_),
                         25833));

    RETURN QUERY SELECT * FROM interface.hike WHERE hike_row_._id = hike._id;
END ;
$$ language plpgsql VOLATILE
                    SECURITY DEFINER;


DROP FUNCTION IF EXISTS public.complete_hike(phone_ VARCHAR(64));
CREATE OR REPLACE FUNCTION public.complete_hike(phone_ VARCHAR(64))
    RETURNS BOOL
AS
$$
DECLARE
    phone_row_ phone.phone%ROWTYPE;
BEGIN
    DELETE
    FROM phone.phone
    WHERE phone_ = phone.phone RETURNING * INTO phone_row_;

    IF phone_row_._id IS NULL THEN
        RETURN FALSE;
    END IF;
    RETURN TRUE;
END;
$$ language plpgsql VOLATILE
                    SECURITY DEFINER;


DROP FUNCTION IF EXISTS public.hike(phone_ VARCHAR(64));
CREATE OR REPLACE FUNCTION public.hike(phone_ VARCHAR(64))
    RETURNS SETOF interface.hike
AS
$$
BEGIN
    RETURN QUERY SELECT *
                 FROM interface.hike
                 WHERE phone = phone_;
END;
$$ language plpgsql SECURITY DEFINER;
