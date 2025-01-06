use crate::prelude::*;
use std::ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Sub, SubAssign};

macro_rules! vec2_impl {
    (
        $(
            @vec2 $name:ident, $little:ident,
            @def $unit:path, $quantity:ty
            $(,{ $($extra:tt)* })? ;
        )*
    ) => {
        pub trait Vec2<Q> {
            fn from_quantities(x: Q, y: Q) -> Self;
        }

        $(
            pub mod $little {
                use super::*;

                pub trait Unit = $unit;
                pub type Quantity = $quantity;

                #[derive(Clone, Copy, Debug, PartialEq)]
                pub struct $name {
                    pub x: Quantity,
                    pub y: Quantity,
                }

                impl Vec2<Quantity> for $name {
                    fn from_quantities(x: Quantity, y: Quantity) -> Self {
                        $name { x, y }
                    }
                }

                impl $name {
                    pub fn new<N>(x: f32, y: f32) -> Self
                    where
                        N: Unit,
                        N: uom::Conversion<f32, T = f32>,
                    {
                        $name {
                            x: Quantity::new::<N>(x),
                            y: Quantity::new::<N>(y),
                        }
                    }

                    pub fn zero() -> Self {
                        use std::marker::PhantomData;

                        $name {
                            x: Quantity {
                                dimension: PhantomData,
                                units: PhantomData,
                                value: 0.,
                            },
                            y: Quantity {
                                dimension: PhantomData,
                                units: PhantomData,
                                value: 0.,
                            },
                        }
                    }

                    pub fn of(x: Quantity, y: Quantity) -> Self {
                        $name { x, y }
                    }

                    pub fn dot<N>(&self, rhs: $name) -> f32
                    where
                        N: Unit,
                        N: uom::Conversion<f32, T = f32>,
                    {
                        let x = self.x.get::<N>() * rhs.x.get::<N>();
                        let y = self.y.get::<N>() * rhs.y.get::<N>();

                        x + y
                    }

                    pub fn cross<N>(&self, rhs: $name) -> f32
                    where
                        N: Unit,
                        N: uom::Conversion<f32, T = f32>,
                    {
                        let x = self.x.get::<N>() * rhs.y.get::<N>();
                        let y = self.y.get::<N>() * rhs.x.get::<N>();

                        x - y
                    }

                    pub fn len_euclid(self, other: $name) -> Quantity {
                        let dx = self.x - other.x;
                        let dy = self.y - other.y;

                        (dx * dx + dy * dy).sqrt()
                    }

                    pub fn len_euclid_squared<N>(self, other: $name) -> f32
                    where
                        N: Unit,
                        N: uom::Conversion<f32, T = f32>,
                    {
                        let dx = self.x.get::<N>() - other.x.get::<N>();
                        let dy = self.y.get::<N>() - other.y.get::<N>();

                        dx * dx + dy * dy
                    }

                    pub fn glam<N>(&self) -> GlamVec2
                    where
                        N: Unit,
                        N: uom::Conversion<f32, T = f32>,
                    {
                        GlamVec2::new(self.x.get::<N>(), self.y.get::<N>())
                    }
                }

                impl<T: Clone, K> Div<T> for $name
                where
                    Quantity: Div<T, Output = K>,
                    K: Vec2Of,
                {
                    type Output = K::Output;

                    fn div(self, rhs: T) -> Self::Output {
                        K::Output::from_quantities(
                            <Quantity as Div<T>>::div(self.x, rhs.clone()),
                            <Quantity as Div<T>>::div(self.y, rhs),
                        )
                    }
                }

                impl<T: Clone> DivAssign<T> for $name
                where
                    Quantity: DivAssign<T>,
                {
                    fn div_assign(&mut self, rhs: T) {
                        self.x /= rhs.clone();
                        self.y /= rhs;
                    }
                }

                impl Div<$name> for $name {
                    type Output = GlamVec2;

                    fn div(self, rhs: $name) -> Self::Output {
                        GlamVec2::new((self.x / rhs.x).into(), (self.y / rhs.y).into())
                    }
                }

                impl<T: Clone, K> Mul<T> for $name
                where
                    Quantity: Mul<T, Output = K>,
                    K: Vec2Of,
                {
                    type Output = K::Output;

                    fn mul(self, rhs: T) -> Self::Output {
                        K::Output::from_quantities(
                            <Quantity as Mul<T>>::mul(self.x, rhs.clone()),
                            <Quantity as Mul<T>>::mul(self.y, rhs),
                        )
                    }
                }

                impl<T: Clone> MulAssign<T> for $name
                where
                    Quantity: MulAssign<T>,
                {
                    fn mul_assign(&mut self, rhs: T) {
                        self.x *= rhs.clone();
                        self.y *= rhs;
                    }
                }

                impl Mul<$name> for f32 {
                    type Output = $name;

                    fn mul(self, rhs: $name) -> Self::Output {
                        rhs * self
                    }
                }

                impl Add<$name> for $name {
                    type Output = $name;

                    fn add(self, rhs: $name) -> Self::Output {
                        $name {
                            x: self.x + rhs.x,
                            y: self.y + rhs.y,
                        }
                    }
                }

                impl AddAssign<$name> for $name {
                    fn add_assign(&mut self, rhs: $name) {
                        self.x += rhs.x;
                        self.y += rhs.y;
                    }
                }

                impl Sub<$name> for $name {
                    type Output = $name;

                    fn sub(self, rhs: $name) -> Self::Output {
                        $name {
                            x: self.x - rhs.x,
                            y: self.y - rhs.y,
                        }
                    }
                }

                impl SubAssign<$name> for $name {
                    fn sub_assign(&mut self, rhs: $name) {
                        self.x -= rhs.x;
                        self.y -= rhs.y;
                    }
                }

                $($($extra)*)?
            }
        )*

        pub trait Vec2Of: Sized {
            type Output: Vec2<Self>;
        }

        $(
            impl Vec2Of for $little::Quantity {
                type Output = $little::$name;
            }
        )*

        $(
            #[allow(unused)]
            pub use $little::$name;
        )*
    };
}

vec2_impl! {
    @vec2 Length2, length,
    @def uom::si::length::Unit, Length;

    @vec2 Acceleration2, acceleration,
    @def uom::si::acceleration::Unit, Acceleration;

    @vec2 Time2, time,
    @def uom::si::time::Unit, Time;

    @vec2 Velocity2, velocity,
    @def uom::si::velocity::Unit, Velocity;

    @vec2 Force2, force,
    @def uom::si::force::Unit, Force;
}
