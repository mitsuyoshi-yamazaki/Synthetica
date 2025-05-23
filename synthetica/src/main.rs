extern crate chrono;
extern crate futures;
extern crate tokio;
extern crate websocket;

pub mod types;

use tokio::runtime;
use tokio::runtime::TaskExecutor;

use std::cmp::{max, min};
use std::collections::HashMap;
use std::fmt::Debug;
use std::time::{Duration, Instant};

use websocket::message::OwnedMessage;
use websocket::server::r#async::Server;
use websocket::server::InvalidConnection;

use futures::future::{self, Loop};
use futures::{Future, Sink, Stream};

use std::sync::{Arc, RwLock};

use chrono::prelude::*;

fn main() {
    let runtime = runtime::Builder::new().build().unwrap();
    let executor = runtime.executor();
    let server = Server::bind("0.0.0.0:8080", &runtime.reactor()).expect("Failed to create server"); // TODO: .envからPORTを呼び出す

    let game_config = types::GameConfig {
        width: 200,
        height: 100,
        pad_height: 20,
    };

    let game_state = types::ServerGameState {
        left_paddle: game_config.height / 2,
        right_paddle: game_config.height / 2,
        players: HashMap::new(),
    };

    // Hashmap to store a sink value with an id key
    // A sink is used to send data to an open client connection
    let connections = Arc::new(RwLock::new(HashMap::new()));
    // Hashmap of id:entity pairs. This is basically the game state
    let entities = Arc::new(RwLock::new(game_state));
    // Used to assign a unique id to each new player
    let counter = Arc::new(RwLock::new(0));

    // Clone references to these states in order to move into the connection_handler closure
    let connections_inner = connections.clone();
    let entities_inner = entities.clone();
    let executor_inner = executor.clone();

    // This stream spawns a future on each new connection request from a client
    // The interesting part is the closure in "for_each" (line 46)
    let connection_handler = server
        .incoming()
        .map_err(|InvalidConnection { error, .. }| error)
        .for_each(move |(upgrade, _addr)| {
            // Clone again to move into closure "f"
            let connections_inner = connections_inner.clone();
            let entities = entities_inner.clone();
            let counter_inner = counter.clone();
            let executor_2inner = executor_inner.clone();

            // This future completes the connection and then proceses the sink and stream
            let accept = upgrade
                .accept()
                .and_then(move |(framed, _)| {
                    let (sink, stream) = framed.split();

                    {
                        // Increment the counter by first locking the RwLock
                        let mut c = counter_inner.write().unwrap();
                        *c += 1;
                    }

                    // Assign an id to the new connection and associate with a new entity and the sink
                    let id = *counter_inner.read().unwrap();
                    connections_inner.write().unwrap().insert(id, sink);

                    println!("New connection {}", id);

                    entities.write().unwrap().players.insert(
                        id,
                        match id {
                            1 => types::PlayerRole::Player(types::Side::Left),
                            2 => types::PlayerRole::Player(types::Side::Right),
                            _ => types::PlayerRole::Observer,
                        },
                    );

                    let c = *counter_inner.read().unwrap();

                    // Spawn a stream to process future messages from this client
                    let f = stream
                        .for_each(move |msg| {
                            process_message(c, &msg, entities.clone());
                            Ok(())
                        })
                        .map_err(move |c| {
                            println!("Error while running core loop {}", c);
                            ();
                        });

                    executor_2inner.spawn(f);

                    Ok(())
                })
                .map_err(|_| ());

            executor_inner.spawn(accept);
            Ok(())
        })
        .map_err(|_| ());

    // This stream is the game loop
    let send_handler = future::loop_fn((), move |_| {
        // This time we clone because "and_then" (line 94) takes a FnMut closure which
        let connections_inner = connections.clone();
        let executor = executor.clone();
        let entities_inner = entities.clone();

        // Delay makes the loop run just 10 times a second
        tokio::timer::Delay::new(Instant::now() + Duration::from_millis(100))
            .map_err(|_| ())
            .and_then(move |_| {
                let now = Utc::now();
                let timestamp = now.timestamp(); // 秒単位のUNIXタイムスタンプを取得
                println!("Current timestamp: {}", timestamp);

                let mut conn = connections_inner.write().unwrap();
                let ids = conn.iter().map(|(k, _v)| k.clone()).collect::<Vec<_>>();

                for id in ids.iter() {
                    // Must take ownership of the sink to send on it
                    // The only way to take ownership of a hashmap value is to remove it
                    // And later put it back (line 124)
                    let sink = conn.remove(id).unwrap();

                    /* Meticulously serialize entity vector into json */
                    let entities = entities_inner.read().unwrap();

                    let player_role = entities.players.get(id);
                    if let Option::Some(player_role) = player_role {
                        let client_game_state = format!(
                            "{{\"left\":{},\"right\":\"{}\",\"role\":\"{}\"}}",
                            entities.left_paddle, entities.right_paddle, player_role,
                        );

                        // Clone for future "f"
                        let connections = connections_inner.clone();
                        let id = id.clone();

                        // This is where the game state is actually sent to the client
                        let f = sink
                            .send(OwnedMessage::Text(client_game_state))
                            .and_then(move |sink| {
                                // Re-insert the entry to the connections map
                                connections.write().unwrap().insert(id.clone(), sink);
                                Ok(())
                            })
                            .map_err(|_| ());

                        executor.spawn(f);
                    }
                }

                // // Damn type inference...
                // // This would just return "Continue" if it could for an infinite loop
                // match true {
                //     true => Ok(Loop::Continue(())),
                //     false => Ok(Loop::Break(())),
                // }
                return Ok(Loop::Continue(()));
            })
    });

    // Finally, block the main thread to wait for the connection_handler and send_handler streams
    // to finish. Which they never should unless there is an error
    let _ = runtime
        .block_on_all(connection_handler.select(send_handler))
        .map_err(|_| println!("Error while running core loop"))
        .unwrap();
}

// Update a player's entity state depending on the command they sent
fn process_message(id: u32, msg: &OwnedMessage, entities: Arc<RwLock<types::ServerGameState>>) {
    if let OwnedMessage::Text(ref txt) = *msg {
        let mut state = entities.write().unwrap();
        if let Some(player_role) = state.players.get(&id) {
            println!("[{}] {} {}", id, player_role, txt);

            if let types::PlayerRole::Player(side) = player_role {
                match side {
                    &types::Side::Left => {
                        if txt == "down" {
                            state.left_paddle = 10;
                        } else if txt == "up" {
                            state.left_paddle -= 10;
                        }
                    }
                    &types::Side::Right => {
                        if txt == "down" {
                            state.right_paddle += 10;
                        } else if txt == "up" {
                            state.right_paddle -= 10;
                        }
                    }
                }
            }
        }
    }
    ();
}
