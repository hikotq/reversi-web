use actix::prelude::*;
use rand::{self, rngs::ThreadRng, Rng};
use reversi::board::{Color, Move as ReversiMove};
use reversi::game::{Game, Winner};
use std::collections::{HashMap, HashSet};

#[derive(Message)]
pub struct Message(pub String);

#[derive(Message)]
#[rtype(usize)]
pub struct Connect {
    pub addr: Recipient<ReversiMessage>,
}

#[derive(Message)]
pub struct Disconnect {
    pub id: usize,
}

#[derive(Message)]
pub struct ClientReversiMoveMessage {
    pub id: usize,
    pub reversi_move: ReversiMove,
    pub room: String,
}

#[derive(Message)]
pub enum ReversiError {
    InvalidMove,
}

#[derive(Serialize, Deserialize, Message, Copy, Clone)]
pub struct ReversiMessage {
    kind: ReversiMessageKind,
    body: Option<ReversiMessageBody>,
}

#[derive(Serialize, Deserialize, Copy, Clone)]
enum ReversiMessageKind {
    GameStart,
    GameOver,
    Turn,
    Move,
    ReversiError,
}

#[derive(Serialize, Deserialize, Copy, Clone)]
pub enum ReversiMessageBody {
    GameOver(Winner),
    Turn(Color),
    Move(ReversiMove),
}

type Uid = usize;

type Uname = String;

struct Player {
    id: Uid,
    name: Uname,
    color: Option<Color>,
}

impl Player {
    fn is_black(&self) -> bool {
        if let Some(color) = self.color {
            color.is_black()
        } else {
            false
        }
    }

    fn is_white(&self) -> bool {
        if let Some(color) = self.color {
            color.is_white()
        } else {
            false
        }
    }
}

pub struct GameRoom {
    game: Game,
    player1: Option<Player>,
    player2: Option<Player>,
}

impl GameRoom {
    fn new() -> Self {
        GameRoom {
            game: Game::new(),
            player1: None,
            player2: None,
        }
    }

    fn black(&self) -> Option<Uid> {
        if let Some(ref p1) = self.player1 {
            if p1.is_black() {
                return Some(p1.id);
            }
        }
        if let Some(ref p2) = self.player1 {
            if p2.is_black() {
                return Some(p2.id);
            }
        }
        None
    }

    fn white(&self) -> Option<Uid> {
        if let Some(ref p1) = self.player1 {
            if p1.is_white() {
                return Some(p1.id);
            }
        }
        if let Some(ref p2) = self.player1 {
            if p2.is_white() {
                return Some(p2.id);
            }
        }
        None
    }
}

#[derive(Serialize, Deserialize)]
pub struct Room {
    name: String,
    stand_by_player: String,
    black: String,
    white: String,
}

pub struct ListRooms;

impl actix::Message for ListRooms {
    type Result = Vec<Room>;
}

#[derive(Message)]
pub struct Join {
    pub name: String,
    pub uid: Uid,
    pub uname: Uname,
}

#[derive(Message)]
pub struct MakeRoom {
    pub name: String,
    pub uid: Uid,
    pub uname: Uname,
    pub color: Option<Color>,
}

pub struct GameServer {
    sessions: HashMap<usize, Recipient<ReversiMessage>>,
    rooms: HashMap<String, HashSet<usize>>,
    games: HashMap<String, GameRoom>,
    rng: ThreadRng,
}

impl Default for GameServer {
    fn default() -> GameServer {
        // default room
        let mut rooms = HashMap::new();
        rooms.insert("Main".to_owned(), HashSet::new());

        GameServer {
            sessions: HashMap::new(),
            rooms: rooms,
            games: HashMap::new(),
            rng: rand::thread_rng(),
        }
    }
}

impl GameServer {
    fn send_reversi_message(&self, room: &str, message: ReversiMessage, id: Uid) {
        if let Some(addr) = self.sessions.get(&id) {
            let _ = addr.do_send(message.clone());
        }
    }
    fn send_reversi_message_room(
        &self,
        room: &str,
        message: ReversiMessage,
        skip_id: Option<usize>,
    ) {
        if let Some(sessions) = self.rooms.get(room) {
            for id in sessions {
                if let Some(skip_id) = skip_id {
                    if *id == skip_id {
                        continue;
                    }
                }
                self.send_reversi_message(room, message.clone(), *id);
            }
        }
    }
}

impl Actor for GameServer {
    type Context = Context<Self>;
}

impl Handler<Connect> for GameServer {
    type Result = usize;

    fn handle(&mut self, msg: Connect, _: &mut Context<Self>) -> Self::Result {
        println!("Someone joined");

        // register session with random id
        let id = self.rng.gen::<usize>();
        self.sessions.insert(id, msg.addr);

        // auto join session to Main room
        self.rooms.get_mut(&"Main".to_owned()).unwrap().insert(id);

        // send id back
        id
    }
}

impl Handler<Disconnect> for GameServer {
    type Result = ();

    fn handle(&mut self, msg: Disconnect, _: &mut Context<Self>) {
        println!("Someone disconnected");

        let mut rooms: Vec<String> = Vec::new();

        // remove address
        if self.sessions.remove(&msg.id).is_some() {
            // remove session from all rooms
            for (name, sessions) in &mut self.rooms {
                if sessions.remove(&msg.id) {
                    rooms.push(name.to_owned());
                }
            }
        }
    }
}

impl Handler<ClientReversiMoveMessage> for GameServer {
    type Result = ();

    fn handle(&mut self, msg: ClientReversiMoveMessage, _: &mut Context<Self>) {
        use self::{ReversiMessage, ReversiMessageBody, ReversiMessageKind};
        if let Some(game) = self.games.get(&msg.room) {
            if game.game.is_start {
                self.send_reversi_message(
                    &msg.room,
                    ReversiMessage {
                        kind: ReversiMessageKind::Move,
                        body: Some(ReversiMessageBody::Move(msg.reversi_move)),
                    },
                    Some(msg.id),
                );
            } else {
                println!("Game is not started");
            }
        }
    }
}


impl Handler<ListRooms> for GameServer {
    type Result = MessageResult<ListRooms>;

    fn handle(&mut self, _: ListRooms, _: &mut Context<Self>) -> Self::Result {
        let mut rooms = Vec::new();

        for (room_name, game) in self.games.iter() {
            if !game.game.is_start {}
        }

        MessageResult(rooms)
    }
}

impl Handler<Join> for GameServer {
    type Result = ();

    fn handle(&mut self, msg: Join, _: &mut Context<Self>) {
        let Join { name, uid, uname } = msg;

        //ゲームルームが存在していないか、すでに満員の場合は終了
        println!("{}", self.games.contains_key(&name));
        if !self.games.contains_key(&name) || self.games.get(&name).unwrap().player2.is_some() {
            println!("Failed enter the room");
            return;
        }

        // すべてのゲームルームからセッションを削除
        for (_, sessions) in &mut self.rooms {
            sessions.remove(&uid);
        }

        //uidが一致するplayerがgameに登録されていたら消す
        for game in self.games.values_mut() {
            if let Some(p1) = game.player1.take() {
                if p1.id != uid {
                    game.player1 = Some(p1);
                }
            }
            if let Some(p2) = game.player2.take() {
                if p2.id != uid {
                    game.player2 = Some(p2);
                }
            }
        }

        println!("{}: Someone connected", name);

        // プレイヤーの登録
        self.rooms.get_mut(&name).unwrap().insert(uid);
        self.games.get_mut(&name).unwrap().player2 = Some(Player {
            id: uid,
            name: uname,
            color: None,
        });

        //1Pの色とは逆の色を入れる.
        //TODO 1Pの色が指定されていない場合は1Pの色はランダムで決めて, 2Pはもう片方の色にする.
        //TODO Rust2018Editionを適用してNLLを使う
        {
            let GameRoom {
                ref mut player1,
                ref mut player2,
                ref mut game,
            } = self.games.get_mut(&name).unwrap();
            let player1 = player1.as_mut().unwrap();
            let player2 = player2.as_mut().unwrap();
            if let Some(color) = player1.color.clone() {
                let color = if color.is_black() {
                    Color::White
                } else {
                    Color::Black
                };
                player2.color = Some(color);
            } else {
                player1.color = Some(Color::Black);
                player2.color = Some(Color::White);
            }
            game.is_start = true;
        }
        self.send_reversi_message(
            &name,
            ReversiMessage {
                kind: ReversiMessageKind::GameStart,
                body: None,
            },
            Some(uid),
        );
    }
}

impl Handler<MakeRoom> for GameServer {
    type Result = ();
    fn handle(&mut self, msg: MakeRoom, _: &mut Context<Self>) {
        let MakeRoom {
            name,
            uid,
            uname,
            color,
        } = msg;

        println!("{} made GameRoom: {}", uname, name);
        if self.rooms.get_mut(&name).is_none() {
            self.rooms.insert(name.clone(), HashSet::new());
        }
        self.rooms.get_mut(&name).unwrap().insert(uid);
        self.games.insert(
            name.clone(),
            GameRoom {
                player1: Some(Player {
                    id: uid,
                    name: uname,
                    color: color,
                }),
                player2: None,
                game: Game::new(),
            },
        );
    }
}
