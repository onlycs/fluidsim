use std::f32;

use glam::{Mat4, Quat, Vec3};

use crate::prelude::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum TimeState {
    Paused,
    /// Running(time of last-ran frame)
    Running(Instant),
    StepRequested,
}

impl TimeState {
    pub(crate) fn toggle(&mut self) {
        *self = match self {
            TimeState::Paused | TimeState::StepRequested => TimeState::Running(Instant::now()),
            TimeState::Running(_) => TimeState::Paused,
        };
    }

    pub(crate) fn pause(&mut self) {
        *self = TimeState::Paused;
    }

    pub(crate) fn step(&mut self) {
        if let TimeState::Paused = self {
            *self = TimeState::StepRequested;
        } else {
            warn!("Cannot step while not paused");
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct PlayerTransform {
    pub translate: Vec3,
    pub q: Quat,
    pub fov: f32, // vertical field of view in radians
}

impl PlayerTransform {
    pub(crate) fn view_matrix(&self) -> Mat4 {
        (Mat4::from_translation(self.translate) * Mat4::from_quat(self.q)).inverse()
    }

    pub(crate) fn projection_matrix(&self, screen: UVec2) -> Mat4 {
        let screen = screen.as_vec2();
        Mat4::perspective_rh(self.fov, screen.x / screen.y, 0.01, 200.0)
    }

    pub(crate) fn q_yaw(&self) -> Quat {
        let fwd = self.q * Vec3::NEG_Z;
        let fwd_xz = Vec3::new(fwd.x, 0.0, fwd.z).normalize();
        Quat::from_rotation_arc(Vec3::NEG_Z, fwd_xz)
    }
}

pub(crate) struct SimulationState {
    pub(crate) time: TimeState,
    pub(crate) gfx: GraphicsSettings,
    pub(crate) init: InitialConditions,
    pub(crate) player: PlayerTransform,
}

impl SimulationState {
    pub(crate) fn new() -> Self {
        Self {
            gfx: GraphicsSettings::default(),
            init: InitialConditions::default(),
            time: TimeState::Paused,
            player: PlayerTransform {
                translate: Vec3::new(12.69, 5.29, 11.57),
                q: Quat::from_xyzw(-0.05, 0.30, 0.00, 0.95),
                fov: f32::consts::FRAC_PI_2,
            },
        }
    }

    /// Calculates the change in time for this simulation frame.
    pub(crate) fn dtime(&mut self) -> f32 {
        match &mut self.time {
            TimeState::Running(prv) => {
                let now = Instant::now();
                let dtime = now.duration_since(*prv).as_secs_f64();
                *prv = now;
                (dtime as f32).min(1.0 / 60.0) * self.gfx.speed
            }
            TimeState::Paused => 0.0,
            TimeState::StepRequested => {
                self.time = TimeState::Paused;
                self.gfx.step_time / 1000.0
            }
        }
    }
}
