use crate::command::Command;
use crate::{DEFAULT_CONN, DEFAULT_SECRET};
use std::collections::HashMap;
use std::env;
use std::sync::Arc;
use tokio_postgres::NoTls;
use warp::Filter;

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
                    eprintln!("connection error: {}", e);
                }
            });
            Arc::new(db)
        }
        Err(_) => panic!("Could not connect to database."),
    };

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
            let query = match command {
                Create(_) => "SELECT * FROM public.create_hike($1, $2)",
                Edit(_) => "SELECT * FROM public.edit_route($1, $2)",
                Checkin(_) => "SELECT * FROM public.edit_route($1, $2)",
                Complete => "SELECT * FROM public.complete_hike($1)",
            };
            let db = db.clone();
            tokio::spawn(async move {
                let _ = if let Complete = command {
                    db.query(query, &[&from]).await
                } else {
                    db.query(query, &[&from, &json]).await
                };
            });
            "".to_string()
        });

    warp::serve(sms).run(([127, 0, 0, 1], 3030)).await;
}
