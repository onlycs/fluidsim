use crate::prelude::*;

use async_std::channel;
use lazy_static::lazy_static;
use physics::settings::{MouseState, SimSettings};

#[cfg(not(target_arch = "wasm32"))]
use async_std::{
    channel::{self, Receiver, Sender},
    sync::{Arc, Mutex},
};

#[cfg(target_arch = "wasm32")]
use std::sync::{
    mpsc::{self, Receiver, Sender},
    Arc, Mutex,
};

#[derive(Clone, Debug, PartialEq)]
pub enum ToPhysics {
    Settings(SimSettings),
    UpdateMouse(Option<MouseState>),
    Reset,
    Pause,
    Step,
    Kill,
}

struct UniversalIpc {
    physics_send: Sender<ToPhysics>,
    physics_recv: Option<Receiver<ToPhysics>>,
}

impl UniversalIpc {
    fn new() -> Self {
        #[cfg(not(target_arch = "wasm32"))]
        let (physics_send, physics_recv) = channel::unbounded();

        #[cfg(target_arch = "wasm32")]
        let (physics_send, physics_recv) = mpsc::channel();

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
        #[cfg(not(target_arch = "wasm32"))]
        pub fn $sender(msg: $ty) {
            trace!("Sending message via {}: {:?}", stringify!($sender), msg);

            task::spawn(async move {
                IPC.lock().await.$sender.send(msg).await.unwrap();
            });
        }

        #[cfg(target_arch = "wasm32")]
        pub fn $sender(msg: $ty) {
            trace!("Sending message via {}: {:?}", stringify!($sender), msg);

            IPC.lock().unwrap().$sender.send(msg).unwrap();
        }
    };

    ($($sender:ident: $ty:ty),+) => {
        $(cfg_sender!($sender: $ty);)+
    };
}

macro_rules! cfg_receiver {
    ($receiver:ident: $ty:ty) => {
        #[cfg(not(target_arch = "wasm32"))]
        pub fn $receiver() -> Receiver<$ty> {
            task::block_on(async move {
                IPC.lock().await.$receiver.take().unwrap()
            })
        }

        #[cfg(target_arch = "wasm32")]
        pub fn $receiver() -> Receiver<$ty> {
            IPC.lock().unwrap().$receiver.take().unwrap()
        }
    };

    ($($sender:ident: $ty:ty),+) => {
        $(cfg_receiver!($sender: $ty);)+
    };
}

cfg_sender!(physics_send: ToPhysics);
cfg_receiver!(physics_recv: ToPhysics);
