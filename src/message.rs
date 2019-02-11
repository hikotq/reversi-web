use actix::prelude::*;
use server::Room;

#[derive(Message)]
pub struct Message(pub String);

#[derive(Message)]
#[rtype(usize)]
pub struct Connect<T>
where
    T: actix::Message + std::marker::Send,
    <T as actix::Message>::Result: std::marker::Send,
{
    pub addr: Recipient<T>,
}

#[derive(Message)]
pub struct Disconnect {
    pub id: usize,
}

pub struct ListRooms {
    pub uid: usize,
}

impl actix::Message for ListRooms {
    type Result = Vec<(String, Room)>;
}

#[derive(Message)]
pub struct Join {
    pub name: String,
    pub uid: usize,
    pub uname: String,
}
