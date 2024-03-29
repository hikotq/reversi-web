use std::time::{Duration, Instant};

use crate::message;
use crate::server;
use actix::*;
use actix_web::server::HttpServer;
use actix_web::{fs, http, ws, App as ActixApp, Error, HttpRequest, HttpResponse};
use reversi::board::{Color, Move as ReversiMove};
use std::str::FromStr;

/// How often heartbeat pings are sent
const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(5);
/// How long before lack of client response causes a timeout
const CLIENT_TIMEOUT: Duration = Duration::from_secs(10);

/// This is our websocket route state, this state is shared with all route instances via `HttpContext::state()`
struct WsGameSessionState {
    addr: Addr<server::GameServer>,
}

/// Entry point for our route
fn chat_route(req: &HttpRequest<WsGameSessionState>) -> Result<HttpResponse, Error> {
    ws::start(
        req,
        WsGameSession {
            id: 0,
            hb: Instant::now(),
            room: "Main".to_owned(),
            name: None,
            color: None,
        },
    )
}

struct WsGameSession {
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

impl Actor for WsGameSession {
    type Context = ws::WebsocketContext<Self, WsGameSessionState>;

    /// Method is called on actor start.
    /// We register ws session with GameServer
    fn started(&mut self, ctx: &mut Self::Context) {
        // we'll start heartbeat process on session start.
        self.hb(ctx);

        // register self in chat server. `AsyncContext::wait` register
        // future within context, but context waits until this future resolves
        // before processing any other events.
        // HttpContext::state() is instance of WsGameSessionState, state is shared
        // across all routes within application
        let addr = ctx.address();
        ctx.state()
            .addr
            .send(message::Connect {
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
        ctx.state()
            .addr
            .do_send(message::Disconnect { id: self.id });
        Running::Stop
    }
}

/// Handle messages from chat server, we simply send it to peer websocket
impl Handler<message::Message> for WsGameSession {
    type Result = ();

    fn handle(&mut self, msg: message::Message, ctx: &mut Self::Context) {
        ctx.text(msg.0);
    }
}

impl Handler<server::ReversiMessage> for WsGameSession {
    type Result = ();

    fn handle(&mut self, msg: server::ReversiMessage, ctx: &mut Self::Context) {
        println!("{:?}", serde_json::to_string(&msg).unwrap());
        ctx.text(serde_json::to_string(&msg).unwrap());
    }
}

/// WebSocket message handler
impl StreamHandler<ws::Message, ws::ProtocolError> for WsGameSession {
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
                        "/listRooms" => {
                            // Send ListRooms message to chat server and wait for
                            // response
                            println!("List rooms");
                            ctx.state()
                                .addr
                                .send(message::ListRooms { uid: self.id })
                                .into_actor(self)
                                .then(|res, _, ctx| {
                                    match res {
                                        Ok(rooms) => {
                                            ctx.text(serde_json::to_string(&rooms).unwrap());
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
                                ctx.state().addr.do_send(message::Join {
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

impl WsGameSession {
    /// helper method that sends ping to client every second.
    ///
    /// also this method checks heartbeats from client
    fn hb(&self, ctx: &mut ws::WebsocketContext<Self, WsGameSessionState>) {
        ctx.run_interval(HEARTBEAT_INTERVAL, |act, ctx| {
            // check client heartbeats
            if Instant::now().duration_since(act.hb) > CLIENT_TIMEOUT {
                // heartbeat timed out
                println!("Websocket Client heartbeat failed, disconnecting!");

                // notify chat server
                ctx.state().addr.do_send(message::Disconnect { id: act.id });

                // stop actor
                ctx.stop();

                // don't try to send a ping
                return;
            }

            ctx.ping("");
        });
    }
}

pub struct App;

impl App {
    pub fn start() {
        let _ = env_logger::init();
        let sys = actix::System::new("websocket-reversi-example");

        // Start chat server actor in separate thread
        let server = Arbiter::start(|_| server::GameServer::default());

        // Create Http server with websocket support
        HttpServer::new(move || {
            // Websocket sessions state
            let state = WsGameSessionState {
                addr: server.clone(),
            };

            ActixApp::with_state(state)
                // redirect to websocket.html
                .resource("/", |r| {
                    r.method(http::Method::GET).f(|_| {
                        HttpResponse::Found()
                            .header("LOCATION", "/index.html")
                            .finish()
                    })
                })
                // websocket
                .resource("/ws/", |r| r.route().f(chat_route))
                // static resources
                .handler("/", fs::StaticFiles::new("static/").unwrap())
        })
        .bind("127.0.0.1:8080")
        .unwrap()
        .start();

        println!("Started http server: 127.0.0.1:8080");
        let _ = sys.run();
    }
}

#[test]
fn test_make_room() {
    use crate::server;
    use actix_web::*;
    use futures::{Future, Stream};
    use std::{thread, time};

    macro_rules! read_ws_assert {
        ($server:expr, $reader:ident, $expect:expr) => {
            let (item, $reader) = $server.execute($reader.into_future()).unwrap();
            assert_eq!(item, Some($expect));
        };
    }

    let (tx, rx) = std::sync::mpsc::channel();

    thread::spawn(move || {
        let sys = actix::System::new("http-server");
        let addr = Arbiter::start(|_| server::GameServer::default());
        let _ = tx.send(addr);
        let _ = sys.run();
    });

    let server = rx.recv().unwrap();
    let mut srv = test::TestServer::build_with_state(move || WsGameSessionState {
        addr: server.clone(),
    })
    .start(|app| {
        app.handler(|req| {
            ws::start(
                req,
                WsGameSession {
                    id: 0,
                    hb: Instant::now(),
                    room: "Main".to_owned(),
                    name: None,
                    color: None,
                },
            )
        })
    });

    let (r1, mut w1) = srv.ws().unwrap();

    let (r2, mut w2) = srv.ws().unwrap();

    w1.text("/makeRoom Shiba pipopa black");
    w1.ping("");
    w2.text("/join Shiba Tatsuo");

    read_ws_assert!(srv, r1, ws::Message::Pong("".to_string()));
    read_ws_assert!(srv, r2, ws::Message::Text("joined".to_string()));
    read_ws_assert!(
        srv,
        r1,
        ws::Message::Text(
            "{\"kind\":\"GameStart\",\"body\":{\"GameStart\":\"Black\"}}".to_string()
        )
    );
    read_ws_assert!(
        srv,
        r2,
        ws::Message::Text(
            "{\"kind\":\"GameStart\",\"body\":{\"GameStart\":\"White\"}}".to_string()
        )
    );
}
