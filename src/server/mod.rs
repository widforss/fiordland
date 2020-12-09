use crate::command::Command;
use crate::{DEFAULT_CONN, DEFAULT_SECRET};
use futures::{stream::SplitSink, FutureExt, SinkExt, StreamExt};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::env;
use std::sync::Arc;
use tokio::sync::mpsc;
use tokio::sync::Mutex;
use tokio_postgres::{row::Row, NoTls};
use uuid::Uuid;
use warp::{
    ws::{Message, WebSocket},
    Filter,
};

pub async fn serve() {
    let conn_string = match env::args().nth(1) {
        Some(conn_string) => conn_string,
        None => DEFAULT_CONN.to_string(),
    };

    let set_secret = match env::args().nth(2) {
        Some(secret) => secret,
        None => DEFAULT_SECRET.to_string(),
    };

    let db = match tokio_postgres::connect(&conn_string[..], NoTls).await {
        Ok((db, connection)) => {
            tokio::spawn(async move {
                if let Err(e) = connection.await {
                    eprintln!("Database connection error: {}", e);
                }
            });
            Arc::new(db)
        }
        Err(_) => panic!("Could not connect to database."),
    };

    let (ws_tx, mut ws_rx) = mpsc::channel::<(String, SplitSink<WebSocket, Message>)>(32);
    let (db_tx, mut db_rx) = mpsc::channel::<(String, String)>(32);
    let sockets: HashMap<String, Vec<SplitSink<WebSocket, Message>>> = HashMap::new();
    let sockets_in = Arc::new(Mutex::new(sockets));
    let sockets_out = Arc::clone(&sockets_in);
    let ws_db = Arc::clone(&db);
    tokio::spawn(async move {
        while let Some((phone, mut ws)) = ws_rx.recv().await {
            match ws_db
                .query("SELECT * FROM public.hike($1)", &[&phone])
                .await
            {
                Ok(rows) if rows.len() > 0 => {
                    let json = convert_rows(rows);
                    let _ = ws.send(Message::text(json)).await;

                    let mut sockets = sockets_in.lock().await;
                    match (*sockets).get_mut(&phone) {
                        Some(wss) => wss.push(ws),
                        None => {
                            (*sockets).insert(phone, vec![ws]);
                        }
                    }
                }
                _ => {
                    let _ = ws.send(Message::text("null".to_string())).await;
                }
            }
        }
    });
    tokio::spawn(async move {
        while let Some((phone, json)) = db_rx.recv().await {
            let mut sockets = sockets_out.lock().await;
            if let Some(wss) = (*sockets).get_mut(&phone) {
                let mut remove = vec![];
                for i in 0..wss.len() {
                    let ws = &mut wss[i];
                    if let Err(_) = ws.send(Message::text(json.clone())).await {
                        remove.push(i);
                    }
                }
                while let Some(i) = remove.pop() {
                    let _ = wss.remove(i);
                }
            }
        }
    });

    let root = warp::get()
        .and(warp::path::end())
        .and(warp::fs::file("www/static/html/index.html"));

    let static_content = warp::path("static").and(warp::fs::dir("www/static"));

    // We always need to return 200, due to 46Elks error handling.
    let sms = warp::post()
        .and(warp::path!("sms" / String))
        .and(warp::body::content_length_limit(1024 * 16))
        .and(warp::body::form())
        .map(move |given_secret: String, sms: HashMap<String, String>| {
            if set_secret != given_secret {
                return "".to_string();
            }

            let message = match sms.get("message") {
                Some(message) => message.clone(),
                None => return "".to_string(),
            };
            let from = match sms.get("from") {
                Some(from) => from.clone(),
                None => return "".to_string(),
            };
            let command = match Command::parse(&message[..]) {
                Ok(command) => command,
                Err(_) => return "".to_string(),
            };
            let json = serde_json::to_value(&command).unwrap();

            use Command::*;
            let db = db.clone();
            let mut db_tx = db_tx.clone();
            let query = match command {
                Create(_) => "SELECT * FROM public.create_hike($1, $2)",
                Edit(_) => "SELECT * FROM public.edit_route($1, $2)",
                Checkin(_) => "SELECT * FROM public.edit_route($1, $2)",
                Complete => "SELECT * FROM public.complete_hike($1)",
            };
            tokio::spawn(async move {
                let rows = if let Complete = command {
                    db.query(query, &[&from]).await.unwrap();
                    None
                } else {
                    match db.query(query, &[&from, &json]).await {
                        Ok(rows) if rows.len() > 0 => Some(rows),
                        _ => None,
                    }
                };

                if let Some(rows) = rows {
                    let json = convert_rows(rows);
                    db_tx.send((from, json)).await.unwrap();
                } else {
                    db_tx.send((from, "null".to_string())).await.unwrap();
                }
            });
            "".to_string()
        });

    let ws = warp::path("listen")
        .and(warp::ws())
        .map(move |ws: warp::ws::Ws| {
            let ws_tx = ws_tx.clone();
            ws.on_upgrade(move |ws| ws_connect(ws, ws_tx.clone()))
        });

    let map = warp::get()
        .and(warp::path("map"))
        .and(warp::path::end())
        .and(warp::fs::file("www/static/html/map.html"));

    let routes = root.or(static_content).or(sms).or(ws).or(map);

    warp::serve(routes).run(([127, 0, 0, 1], 3030)).await;
}

async fn ws_connect(
    ws: WebSocket,
    mut ws_tx: mpsc::Sender<(String, SplitSink<WebSocket, Message>)>,
) {
    let (tx, mut rx) = ws.split();
    let phone = rx.next().map(move |phone| phone).await;
    if let Some(Ok(phone)) = phone {
        if let Ok(phone) = phone.to_str() {
            ws_tx.send((phone.to_string(), tx)).await.unwrap();
        }
    }
}

fn convert_rows(rows: Vec<Row>) -> String {
    let _id: Uuid = rows[0].get(0);
    let routes: Option<Value> = rows[0].get(2);
    let traces: Option<Value> = rows[0].get(3);
    json!({
        "_id": _id,
        "routes": routes,
        "traces": traces,
    })
    .to_string()
}
