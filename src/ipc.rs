use crate::prelude::*;

use async_std::channel;
use lazy_static::lazy_static;
use std::sync::{Arc, Mutex};

#[derive(Clone, Debug, PartialEq)]
pub enum ToPhysics {
    Resize(f32, f32),
    Gravity(f32),
    Pause,
    Step,
}

#[derive(Clone, Debug, PartialEq)]
pub enum ToRenderer {
    Kill,
}

#[derive(Clone, Debug, PartialEq)]
pub enum ToConfig {
    Show,
    Hide,
}

struct UniversalIpc {
    config_send: Sender<ToConfig>,
    config_recv: Option<Receiver<ToConfig>>,

    render_send: Sender<ToRenderer>,
    render_recv: Option<Receiver<ToRenderer>>,

    physics_send: Sender<ToPhysics>,
    physics_recv: Option<Receiver<ToPhysics>>,
}

impl UniversalIpc {
    fn new() -> Self {
        let (config_send, config_recv) = channel::unbounded();
        let (render_send, render_recv) = channel::unbounded();
        let (physics_send, physics_recv) = channel::unbounded();

        Self {
            config_send,
            config_recv: Some(config_recv),
            render_send,
            render_recv: Some(render_recv),
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
            task::spawn_blocking(|| IPC.lock().unwrap().$sender.send_blocking(msg).unwrap());
        }
    };

    ($($sender:ident: $ty:ty),+) => {
        $(cfg_sender!($sender: $ty);)+
    };
}

macro_rules! cfg_reciever {
    ($reciever:ident: $ty:ty) => {
        pub fn $reciever() -> Receiver<$ty> {
            IPC.lock().unwrap().$reciever.take().unwrap()
        }
    };

    ($($sender:ident: $ty:ty),+) => {
        $(cfg_reciever!($sender: $ty);)+
    };
}

cfg_sender!(config_send: ToConfig, render_send: ToRenderer, physics_send: ToPhysics);
cfg_reciever!(config_recv: ToConfig, render_recv: ToRenderer, physics_recv: ToPhysics);
