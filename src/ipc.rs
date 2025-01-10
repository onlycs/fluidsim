use crate::prelude::*;

use async_std::channel;
use async_std::sync::{Arc, Mutex};
use lazy_static::lazy_static;
use physics::settings::{MouseState, SimSettings};

#[derive(Clone, Debug, PartialEq)]
pub enum ToPhysics {
    Settings(SimSettings),
    UpdateMouse(Option<MouseState>),
    Reset,
    Pause,
    Step,
}

struct UniversalIpc {
    physics_send: Sender<ToPhysics>,
    physics_recv: Option<Receiver<ToPhysics>>,
}

impl UniversalIpc {
    fn new() -> Self {
        let (physics_send, physics_recv) = channel::unbounded();

        Self {
            physics_send,
            physics_recv: Some(physics_recv),
        }
    }
}

lazy_static! {
    static ref IPC: Arc<Mutex<UniversalIpc>> = Arc::new(Mutex::new(UniversalIpc::new()));
}

macro_rules! cfg_sender {
    ($sender:ident: $ty:ty) => {
        pub fn $sender(msg: $ty) {
            trace!("Sending message via {}: {:?}", stringify!($sender), msg);

            task::spawn(async move {
                IPC.lock().await.$sender.send(msg).await.unwrap();
            });
        }
    };

    ($($sender:ident: $ty:ty),+) => {
        $(cfg_sender!($sender: $ty);)+
    };
}

macro_rules! cfg_reciever {
    ($reciever:ident: $ty:ty) => {
        pub fn $reciever() -> Receiver<$ty> {
            task::block_on(async move {
                IPC.lock().await.$reciever.take().unwrap()
            })
        }
    };

    ($($sender:ident: $ty:ty),+) => {
        $(cfg_reciever!($sender: $ty);)+
    };
}

cfg_sender!(physics_send: ToPhysics);
cfg_reciever!(physics_recv: ToPhysics);
