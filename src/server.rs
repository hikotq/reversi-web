use actix::prelude::*;
use message::*;
use rand::{self, rngs::ThreadRng, Rng};
use reversi::board::{Color, Move as ReversiMove};
use reversi::game::{Game as ReversiGame, Winner};
use std::collections::{HashMap, HashSet};

type Uid = usize;
type Uname = String;

#[derive(Message)]
pub struct MakeRoom {
    pub name: String,
    pub uid: Uid,
    pub uname: Uname,
    pub color: Option<Color>,
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

#[derive(Serialize, Deserialize, Message, Clone)]
pub struct Game {
    board: Vec<String>,
    turn: Color,
}

impl From<&ReversiGame> for Game {
    fn from(game: &ReversiGame) -> Self {
        let board = game
            .board
            .iter()
            .cloned()
            .map(|cell| cell.to_string())
            .collect();
        Self {
            board: board,
            turn: game.turn,
        }
    }
}

#[derive(Serialize, Deserialize, Message, Clone)]
pub struct ReversiMessage {
    kind: ReversiMessageKind,
    body: Option<ReversiMessageBody>,
}

#[derive(Serialize, Deserialize, Copy, Clone)]
enum ReversiMessageKind {
    Game,
    GameStart,
    GameOver,
    Turn,
    Move,
    ReversiError,
}

#[derive(Serialize, Deserialize, Clone)]
pub enum ReversiMessageBody {
    GameOver((Game, Winner)),
    Turn(Color),
    Move(ReversiMove),
    GameStart(Color),
    Game(Game),
}

#[derive(Serialize, Deserialize, Clone, Debug)]
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

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct Room {
    #[serde(skip)]
    sessions: HashSet<usize>,
    #[serde(skip)]
    game: ReversiGame,
    player1: Option<Player>,
    player2: Option<Player>,
}

impl Room {
    fn new() -> Self {
        Room {
            sessions: HashSet::new(),
            game: ReversiGame::new(),
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

type RoomMap = HashMap<String, Room>;

trait RoomMapImpl {
    fn make_room(&mut self, room_name: String, Uid, Uname, Option<Color>);
    fn join(&mut self, room_name: String, Uid, Uname) -> Result<(usize, usize), String>;
    fn leave(&mut self, Uid);
}

impl RoomMapImpl for RoomMap {
    fn make_room(&mut self, room_name: String, uid: Uid, uname: Uname, color: Option<Color>) {
        if self.get_mut(&room_name).is_some() {
            eprintln!("Room {} is already created", room_name);
            return;
        }

        let mut sessions = HashSet::new();
        sessions.insert(uid);
        let room = Room {
            sessions,
            player1: Some(Player {
                id: uid,
                name: uname,
                color: color,
            }),
            player2: None,
            game: ReversiGame::new(),
        };
        self.insert(room_name.clone(), room);
    }

    fn join(
        &mut self,
        room_name: String,
        uid: Uid,
        uname: Uname,
    ) -> Result<(usize, usize), String> {
        //ゲームルームが存在していないか、すでに満員の場合は終了
        if !self.contains_key(&room_name) || self.get(&room_name).unwrap().player2.is_some() {
            println!("Failed enter the room");
            return Err("Failed enter the room".to_string());
        }

        // すべてのゲームルームからセッションを削除
        for (_, Room { sessions, .. }) in self.iter_mut() {
            sessions.remove(&uid);
        }

        //uidが一致するplayerがgameに登録されていたら消す
        for room in self.values_mut() {
            if let Some(p1) = room.player1.take() {
                if p1.id != uid {
                    room.player1 = Some(p1);
                }
            }
            if let Some(p2) = room.player2.take() {
                if p2.id != uid {
                    room.player2 = Some(p2);
                }
            }
        }

        println!("{}: Someone connected", room_name);

        // プレイヤーの登録
        self.get_mut(&room_name).unwrap().sessions.insert(uid);
        self.get_mut(&room_name).unwrap().player2 = Some(Player {
            id: uid,
            name: uname,
            color: None,
        });

        //1Pの色とは逆の色を入れる.
        //TODO 1Pの色が指定されていない場合は1Pの色はランダムで決めて, 2Pはもう片方の色にする.
        //TODO Rust2018Editionを適用してNLLを使う
        let black_id;
        let white_id;
        {
            let Room {
                ref mut player1,
                ref mut player2,
                ref mut game,
                ..
            } = self.get_mut(&room_name).unwrap();
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
            if player1.color.unwrap().is_black() {
                black_id = player1.id;
                white_id = player2.id;
            } else {
                white_id = player1.id;
                black_id = player2.id;
            }
            game.is_start = true;
        }
        Ok((black_id, white_id))
    }

    // 各Roomのセッションからもユーザーを削除し
    // セッションが空になったRoomを取り除く
    fn leave(&mut self, uid: Uid) {
        self.retain(
            |name,
             Room {
                 sessions,
                 player1,
                 player2,
                 ..
             }| {
                // 退出するユーザーがプレイヤーである場合は
                // 対応するプレイヤー(player1、またはplayer2)をNoneにする
                *player1 = player1.take().filter(|p1| p1.id != uid);
                *player2 = player2.take().filter(|p2| p2.id != uid);

                sessions.remove(&uid);
                if sessions.is_empty() {
                    println!("Remove Room: {}", name);
                }
                !sessions.is_empty()
            },
        )
    }
}

pub struct GameServer {
    sessions: HashMap<usize, Recipient<ReversiMessage>>,
    rooms: RoomMap,
    rng: ThreadRng,
}

impl Default for GameServer {
    fn default() -> GameServer {
        GameServer {
            sessions: HashMap::new(),
            rooms: HashMap::new(),
            rng: rand::thread_rng(),
        }
    }
}

impl GameServer {
    fn send_reversi_message(&self, message: ReversiMessage, id: Uid) {
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
        if let Some(Room { sessions, .. }) = self.rooms.get(room) {
            for id in sessions {
                if let Some(skip_id) = skip_id {
                    if *id == skip_id {
                        continue;
                    }
                }
                self.send_reversi_message(message.clone(), *id);
            }
        }
    }
}

impl Actor for GameServer {
    type Context = Context<Self>;
}

impl Handler<Connect<ReversiMessage>> for GameServer {
    type Result = usize;

    fn handle(&mut self, msg: Connect<ReversiMessage>, _: &mut Context<Self>) -> Self::Result {
        println!("Someone joined");

        // register session with random id
        let id = self.rng.gen::<usize>();
        self.sessions.insert(id, msg.addr);

        // send id back
        id
    }
}

impl Handler<Disconnect> for GameServer {
    type Result = ();

    fn handle(&mut self, msg: Disconnect, _: &mut Context<Self>) {
        println!("Someone disconnected");

        // remove address
        if self.sessions.remove(&msg.id).is_some() {
            self.rooms.leave(msg.id);
        }
    }
}

impl Handler<ClientReversiMoveMessage> for GameServer {
    type Result = ();

    fn handle(&mut self, msg: ClientReversiMoveMessage, _: &mut Context<Self>) {
        use self::{ReversiMessage, ReversiMessageBody, ReversiMessageKind};
        let mut is_over = false;
        if self.rooms.contains_key(&msg.room) {
            let result = {
                let room = self.rooms.get_mut(&msg.room).unwrap();
                println!("{:?}", room.game.board);
                if room.game.is_start {
                    let result = room.game.put_piece(msg.reversi_move);
                    if result.is_ok() && !room.game.is_over {
                        room.game.change_turn();
                        is_over = room.game.is_over;
                    }
                    result
                } else {
                    Err("Game is not started".to_string())
                }
            };
            println!("{:?}", result);
            if result.is_ok() {
                if is_over {
                    let game = &self.rooms.get(&msg.room).unwrap().game;
                    let winner = game.winner();
                    let game = Game::from(game);
                    self.send_reversi_message_room(
                        &msg.room,
                        ReversiMessage {
                            kind: ReversiMessageKind::GameOver,
                            body: Some(ReversiMessageBody::GameOver((game, winner))),
                        },
                        None,
                    );
                } else {
                    self.send_reversi_message_room(
                        &msg.room,
                        ReversiMessage {
                            kind: ReversiMessageKind::Game,
                            body: Some(ReversiMessageBody::Game(Game::from(
                                &self.rooms.get(&msg.room).unwrap().game,
                            ))),
                        },
                        None,
                    );
                }
            }
        }
    }
}

impl Handler<ListRooms> for GameServer {
    type Result = MessageResult<ListRooms>;

    fn handle(&mut self, msg: ListRooms, _: &mut Context<Self>) -> Self::Result {
        let uid = msg.uid;

        let room_list: Vec<(String, Room)> = self.rooms.clone().into_iter().collect();
        println!("{:?}", room_list);
        MessageResult(room_list)
    }
}

impl Handler<Join> for GameServer {
    type Result = ();

    fn handle(&mut self, msg: Join, _: &mut Context<Self>) {
        let Join { name, uid, uname } = msg;
        if let Ok((black_id, white_id)) = self.rooms.join(name, uid, uname) {
            self.send_reversi_message(
                ReversiMessage {
                    kind: ReversiMessageKind::GameStart,
                    body: Some(ReversiMessageBody::GameStart(Color::Black)),
                },
                black_id,
            );
            self.send_reversi_message(
                ReversiMessage {
                    kind: ReversiMessageKind::GameStart,
                    body: Some(ReversiMessageBody::GameStart(Color::White)),
                },
                white_id,
            );
        }
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
        self.rooms.make_room(name, uid, uname, color);
    }
}
