#![allow(unused_variables)]
extern crate byteorder;
extern crate bytes;
extern crate env_logger;
extern crate futures;
extern crate rand;
extern crate serde;
extern crate serde_json;
extern crate tokio_core;
extern crate tokio_io;

#[macro_use]
extern crate serde_derive;

extern crate actix;
extern crate actix_web;

use std::time::{Duration, Instant};

use actix::*;
use actix_web::server::HttpServer;
use actix_web::{fs, http, ws, App, Error, HttpRequest, HttpResponse};

mod reversi;
mod server;

use reversi::board::{Color, Move as ReversiMove};
use std::str::FromStr;

/// How often heartbeat pings are sent
const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(5);
/// How long before lack of client response causes a timeout
const CLIENT_TIMEOUT: Duration = Duration::from_secs(10);

/// This is our websocket route state, this state is shared with all route
/// instances via `HttpContext::state()`
struct WsChatSessionState {
    addr: Addr<server::GameServer>,
}

/// Entry point for our route
fn chat_route(req: &HttpRequest<WsChatSessionState>) -> Result<HttpResponse, Error> {
    ws::start(
        req,
        WsChatSession {
            id: 0,
            hb: Instant::now(),
            room: "Main".to_owned(),
            name: None,
            color: None,
        },
    )
}

struct WsChatSession {
    /// unique session id
    id: usize,
    /// Client must send ping at least once per 10 seconds (CLIENT_TIMEOUT),
    /// otherwise we drop connection.
    hb: Instant,
    /// joined room
    room: String,
    /// color
    color: Option<Color>,
    /// peer name
    name: Option<String>,
}

impl Actor for WsChatSession {
    type Context = ws::WebsocketContext<Self, WsChatSessionState>;

    /// Method is called on actor start.
    /// We register ws session with GameServer
    fn started(&mut self, ctx: &mut Self::Context) {
        // we'll start heartbeat process on session start.
        self.hb(ctx);

        // register self in chat server. `AsyncContext::wait` register
        // future within context, but context waits until this future resolves
        // before processing any other events.
        // HttpContext::state() is instance of WsChatSessionState, state is shared
        // across all routes within application
        let addr = ctx.address();
        ctx.state()
            .addr
            .send(server::Connect {
                addr: addr.recipient(),
            })
            .into_actor(self)
            .then(|res, act, ctx| {
                match res {
                    Ok(res) => act.id = res,
                    // something is wrong with chat server
                    _ => ctx.stop(),
                }
                fut::ok(())
            })
            .wait(ctx);
    }

    fn stopping(&mut self, ctx: &mut Self::Context) -> Running {
        // notify chat server
        ctx.state().addr.do_send(server::Disconnect { id: self.id });
        Running::Stop
    }
}

/// Handle messages from chat server, we simply send it to peer websocket
impl Handler<server::Message> for WsChatSession {
    type Result = ();

    fn handle(&mut self, msg: server::Message, ctx: &mut Self::Context) {
        ctx.text(msg.0);
    }
}

impl Handler<server::ReversiMessage> for WsChatSession {
    type Result = ();

    fn handle(&mut self, msg: server::ReversiMessage, ctx: &mut Self::Context) {
        println!("{:?}", serde_json::to_string(&msg).unwrap());
        ctx.text(serde_json::to_string(&msg).unwrap());
    }
}

/// WebSocket message handler
impl StreamHandler<ws::Message, ws::ProtocolError> for WsChatSession {
    fn handle(&mut self, msg: ws::Message, ctx: &mut Self::Context) {
        println!("WEBSOCKET MESSAGE: {:?}", msg);
        match msg {
            ws::Message::Ping(msg) => {
                self.hb = Instant::now();
                ctx.pong(&msg);
            }
            ws::Message::Pong(_) => {
                self.hb = Instant::now();
            }
            ws::Message::Text(text) => {
                let m = text.trim();
                // we check for /sss type of messages
                if m.starts_with('/') {
                    let v: Vec<&str> = m.splitn(4, ' ').collect();
                    match v[0] {
                        "/standByList" => {
                            // Send ListRooms message to chat server and wait for
                            // response
                            println!("List rooms");
                            ctx.state()
                                .addr
                                .send(server::ListRooms)
                                .into_actor(self)
                                .then(|res, _, ctx| {
                                    match res {
                                        Ok(rooms) => {
                                            for room in &rooms {
                                                ctx.text(serde_json::to_string(room).unwrap());
                                            }
                                        }
                                        _ => println!("Something is wrong"),
                                    }
                                    fut::ok(())
                                })
                                .wait(ctx)
                            // .wait(ctx) pauses all events in context,
                            // so actor wont receive any new messages until it get list
                            // of rooms back
                        }
                        "/join" => {
                            if v.len() == 3 {
                                self.room = v[1].to_owned();
                                let uname = v[2].to_owned();
                                ctx.state().addr.do_send(server::Join {
                                    name: self.room.clone(),
                                    uid: self.id,
                                    uname: uname,
                                });
                                ctx.text("joined");
                            } else {
                                ctx.text("!!! room name is required");
                            }
                        }
                        "/makeRoom" => {
                            println!("someone made room");
                            if v.len() == 3 {
                                self.room = v[1].to_owned();
                                let uname = v[2].to_owned();
                                ctx.state().addr.do_send(server::MakeRoom {
                                    name: self.room.clone(),
                                    uid: self.id,
                                    uname: uname,
                                    color: None,
                                });
                            } else if v.len() == 4 {
                                self.room = v[1].to_owned();
                                let uname = v[2].to_owned();
                                let color = <Color as FromStr>::from_str(v[3]).unwrap();
                                ctx.state().addr.do_send(server::MakeRoom {
                                    name: self.room.clone(),
                                    uid: self.id,
                                    uname: uname,
                                    color: Some(color),
                                });
                            } else {
                                ctx.text("!!! room name is required");
                            }
                        }
                        "/move" => {
                            if v.len() == 4 {
                                let color = if v[1].to_uppercase() == "BLACK" {
                                    Color::Black
                                } else {
                                    Color::White
                                };
                                let m = ReversiMove {
                                    x: v[2].parse().unwrap(),
                                    y: v[3].parse().unwrap(),
                                    color,
                                };
                                ctx.state().addr.do_send(server::ClientReversiMoveMessage {
                                    id: self.id,
                                    reversi_move: m,
                                    room: self.room.clone(),
                                })
                            } else {
                                ctx.text("!!! color is required");
                            }
                        }
                        _ => ctx.text(format!("!!! unknown command: {:?}", m)),
                    }
                } else {
                    //let msg = if let Some(ref name) = self.name {
                    //    format!("{}: {}", name, m)
                    //} else {
                    //    m.to_owned()
                    //};
                    //// send message to chat server
                    //ctx.state().addr.do_send(server::ClientReversiMoveMessage {
                    //    id: self.id,
                    //    msg: msg,
                    //    room: self.room.clone(),
                    //})
                }
            }
            ws::Message::Binary(bin) => println!("Unexpected binary"),
            ws::Message::Close(_) => {
                ctx.stop();
            }
        }
    }
}

impl WsChatSession {
    /// helper method that sends ping to client every second.
    ///
    /// also this method checks heartbeats from client
    fn hb(&self, ctx: &mut ws::WebsocketContext<Self, WsChatSessionState>) {
        ctx.run_interval(HEARTBEAT_INTERVAL, |act, ctx| {
            // check client heartbeats
            if Instant::now().duration_since(act.hb) > CLIENT_TIMEOUT {
                // heartbeat timed out
                println!("Websocket Client heartbeat failed, disconnecting!");

                // notify chat server
                ctx.state().addr.do_send(server::Disconnect { id: act.id });

                // stop actor
                ctx.stop();

                // don't try to send a ping
                return;
            }

            ctx.ping("");
        });
    }
}

fn main() {
    let _ = env_logger::init();
    let sys = actix::System::new("websocket-reversi-example");

    // Start chat server actor in separate thread
    let server = Arbiter::start(|_| server::GameServer::default());

    // Create Http server with websocket support
    HttpServer::new(move || {
        // Websocket sessions state
        let state = WsChatSessionState {
            addr: server.clone(),
        };

        App::with_state(state)
            // redirect to websocket.html
            .resource("/", |r| {
                r.method(http::Method::GET).f(|_| {
                    HttpResponse::Found()
                        .header("LOCATION", "/static/websocket.html")
                        .finish()
                })
            })
            // websocket
            .resource("/ws/", |r| r.route().f(chat_route))
            // static resources
            .handler("/static/", fs::StaticFiles::new("static/").unwrap())
    })
    .bind("127.0.0.1:8080")
    .unwrap()
    .start();

    println!("Started http server: 127.0.0.1:8080");
    let _ = sys.run();
}
