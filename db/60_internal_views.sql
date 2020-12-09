CREATE OR REPLACE VIEW interface.route AS
SELECT route._hike_id,
       route._id                 as _route_id,
       MAX(route_point.log_date) AS log_date,
       JSONB_BUILD_OBJECT(
               'type', 'FeatureCollection',
               'id', route._id,
               'features', JSONB_AGG(JSONB_BUILD_OBJECT(
               'type', 'Feature',
               'id', route_point._id,
               'geometry', ST_AsGeoJSON(route_point.geom),
               'properties', JSONB_BUILD_OBJECT(
                       'action', action.action,
                       'message', route_point.message,
                       'date', route_point.date,
                       'time', route_point.time
                   )
           ))
           )                     AS geojson
FROM hike.route
         JOIN (SELECT *
               FROM hike.route_point
               ORDER BY COALESCE(date, log_date::DATE) DESC, time DESC,
                        log_date DESC) route_point
              ON route_point._route_id = hike.route._id
         LEFT JOIN hike.action ON route_point._action_id = hike.action._id
GROUP BY route._id;

CREATE OR REPLACE VIEW interface.trace AS
SELECT trace._hike_id,
       trace._id                 as _trace_id,
       MAX(trace_point.log_date) AS log_date,
       JSONB_BUILD_OBJECT(
               'type', 'FeatureCollection',
               'id', trace._id,
               'features', JSONB_AGG(JSONB_BUILD_OBJECT(
               'type', 'Feature',
               'id', trace_point._id,
               'geometry', ST_AsGeoJSON(trace_point.geom),
               'properties', JSONB_BUILD_OBJECT(
                       'action', action.action,
                       'message', trace_point.message,
                       'date', trace_point.date,
                       'time', trace_point.time
                   )
           ))
           )                     AS geojson
FROM hike.trace
         JOIN (SELECT *
               FROM hike.trace_point
               ORDER BY COALESCE(date, log_date::DATE) DESC, time DESC,
                        log_date DESC) trace_point
              ON trace_point._trace_id = hike.trace._id
         LEFT JOIN hike.action ON trace_point._action_id = hike.action._id
GROUP BY trace._id;

CREATE OR REPLACE VIEW interface.hike AS
SELECT hike._id,
       phone.phone,
       CASE
           WHEN COUNT(route.*) > 0 THEN JSONB_AGG(route.geojson)
           ELSE NULL END AS routes,
       CASE
           WHEN COUNT(trace.*) > 0 THEN JSONB_AGG(trace.geojson)
           ELSE NULL END AS traces
FROM hike.hike
         JOIN phone.phone ON hike.hike._phone_id = phone.phone._id
         LEFT JOIN (SELECT *
                    FROM interface.route
                    ORDER BY route.log_date) route
                   ON route._hike_id = hike._id
         LEFT JOIN (SELECT *
                    FROM interface.trace
                    ORDER BY trace.log_date) trace
                   ON trace._hike_id = hike._id
GROUP BY hike._id,
         phone.phone;
