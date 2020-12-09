CREATE OR REPLACE VIEW interface.route AS
SELECT route._hike_id,
       route._id                as _route_id,
       route.log_date,
       CASE
           WHEN COUNT(route_point.*) > 0 THEN JSONB_BUILD_OBJECT(
                   'type', 'FeatureCollection',
                   'id', route._id,
                   'features', JSONB_AGG(
                           JSONB_BUILD_OBJECT(
                                   'type', 'Feature',
                                   'id', route_point._id,
                                   'geometry',
                                   ST_AsGeoJSON(route_point.geom)::JSONB,
                                   'properties', JSONB_BUILD_OBJECT(
                                           'action', action.action,
                                           'message',
                                           route_point.message,
                                           'date', route_point.date,
                                           'time', route_point.time
                                       )
                               ))
               )
           ELSE NULL END AS geojson
FROM hike.route
         LEFT JOIN hike.route_point
                   ON route_point._route_id = hike.route._id
         LEFT JOIN hike.action ON route_point._action_id = hike.action._id
GROUP BY route._id;

CREATE OR REPLACE VIEW interface.trace AS
SELECT trace._hike_id,
       trace._id                as _trace_id,
       trace.log_date,
       CASE
           WHEN COUNT(trace_point.*) > 0 THEN JSONB_BUILD_OBJECT(
                   'type', 'FeatureCollection',
                   'id', trace._id,
                   'features', JSONB_AGG(
                           JSONB_BUILD_OBJECT(
                                   'type', 'Feature',
                                   'id', trace_point._id,
                                   'geometry',
                                   ST_AsGeoJSON(trace_point.geom)::JSONB,
                                   'properties', JSONB_BUILD_OBJECT(
                                           'action', action.action,
                                           'message',
                                           trace_point.message,
                                           'date', trace_point.date,
                                           'time', trace_point.time
                                       )
                               ))
               )
           ELSE NULL END AS geojson
FROM hike.trace
         LEFT JOIN hike.trace_point
                   ON trace_point._trace_id = hike.trace._id
         LEFT JOIN hike.action ON trace_point._action_id = hike.action._id
GROUP BY trace._id;

CREATE OR REPLACE VIEW interface.hike AS
SELECT hike._id,
       phone.phone,
       route.geojson as route,
       trace.geojson as trace
FROM hike.hike
         JOIN phone.phone ON hike.hike._phone_id = phone.phone._id
         LEFT JOIN (SELECT *
                    FROM interface.route
                    ORDER BY route.log_date DESC
                    LIMIT 1) route
                   ON route._hike_id = hike._id
         LEFT JOIN (SELECT *
                    FROM interface.trace
                    ORDER BY trace.log_date DESC
                    LIMIT 1) trace
                   ON trace._hike_id = hike._id;
