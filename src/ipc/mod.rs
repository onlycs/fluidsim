pub mod shared;

cfg_if! {
    if #[cfg(not(feature = "sync"))] {
        use crate::prelude::*;
        use lazy_static::lazy_static;

        #[derive(Clone, Debug, PartialEq)]
        pub enum ToPhysics {
            Settings(SimSettings),
            UpdateMouse(MouseState),
            Reset,
            Pause,
            Step,
            Kill,
        }

        struct IpcIsh {
            physics: &'static mut Vec<ToPhysics>,
        }

        impl IpcIsh {
            fn new() -> Self {
                Self {
                    physics: Box::leak(Box::new(vec![])),
                }
            }
        }

        lazy_static! {
            static ref IPC: IpcIsh = IpcIsh::new();
        }

        #[allow(invalid_reference_casting)] // dancing with the devil
        fn get_physics() -> &'static mut Vec<ToPhysics> {
            let physics = &IPC.physics;
            let physics = physics as *const &mut Vec<ToPhysics>;
            let physics = physics as *mut &mut Vec<ToPhysics>;
            unsafe { &mut *physics }
        }

        pub fn physics_send(msg: ToPhysics) {
            std::hint::black_box({
                get_physics().push(msg)
            });
        }

        pub fn physics_recv() -> Option<ToPhysics> {
            std::hint::black_box(
                get_physics().pop()
            )
        }
    }
}
