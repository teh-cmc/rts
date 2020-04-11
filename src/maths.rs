use cgmath::{
    Matrix4 as CGMat4, Point3 as CGPoint3, Quaternion as CGQuat, Vector2 as CGVec2,
    Vector3 as CGVec3, Vector4 as CGVec4,
};
use collision::Ray3 as CGRay3;
use raylib::{
    core::math::{
        Matrix as RMat4, Quaternion as RQuat, Ray as RRay3, Vector2 as RVec2, Vector3 as RVec3,
        Vector4 as RVec4,
    },
    ffi::{
        Matrix as RawMat4, Quaternion as RawQuat, Ray as RawRay3, Vector2 as RawVec2,
        Vector3 as RawVec3, Vector4 as RawVec4,
    },
};
use std::ops::{Deref, DerefMut};

pub use cgmath::prelude::*;
pub mod prelude {
    pub use super::*;
    pub use cgmath::{
        Matrix4 as CGMat4, Point3 as CGPoint3, Quaternion as CGQuat, Vector2 as CGVec2,
        Vector3 as CGVec3, Vector4 as CGVec4,
    };
}

// TODO(cmc): macros everywhere.
// TODO(cmc): provide Add, AddAssign and everything else.

pub type Quat = CGQuat<f32>;

// -----------------------------------------------------------------------------

#[derive(Debug, Clone, Copy)]
pub struct Mat4(CGMat4<f32>);

impl Default for Mat4 {
    fn default() -> Self {
        Self(CGMat4::identity())
    }
}

/* Deref to cgmath */

impl Deref for Mat4 {
    type Target = CGMat4<f32>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl DerefMut for Mat4 {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

/* Instantiate from cgmath */

impl<T> From<T> for Mat4
where
    T: Into<CGMat4<f32>>,
{
    fn from(m: T) -> Self {
        Self(m.into())
    }
}

/* Instantiate from Raylib Rust */

// impl From<RMat4> for Mat4 {
//     fn from(m: RMat4) -> Self {
//         let m = m.to_array();
//         #[rustfmt::skip]
//         let m = CGMat4::new(
//             m[0],  m[1],  m[2],  m[3],
//             m[4],  m[5],  m[6],  m[7],
//             m[8],  m[9],  m[10], m[11],
//             m[12], m[13], m[14], m[15],
//         );
//         Self(m)
//     }
// }
// impl From<&RMat4> for Mat4 {
//     fn from(m: &RMat4) -> Self {
//         Self::from(m.clone())
//     }
// }

/* Instantiate from Raylib C */

// impl From<RawMat4> for Mat4 {
//     fn from(m: RawMat4) -> Self {
//         Self::from(RMat4::from(m))
//     }
// }
// impl From<&RawMat4> for Mat4 {
//     fn from(m: &RawMat4) -> Self {
//         Self::from(m.clone())
//     }
// }

/* Feed directly to Raylib C */

impl Into<RMat4> for Mat4 {
    #[rustfmt::skip]
    fn into(self) -> RMat4 {
        let c0 = self.0.x;
        let c1 = self.0.y;
        let c2 = self.0.z;
        let c3 = self.0.w;
        RMat4 {
            m0:  c0[0], m1:  c0[1], m2:  c0[2], m3: c0[3],
            m4:  c1[0], m5:  c1[1], m6:  c1[2], m7: c1[3],
            m8:  c2[0], m9:  c2[1], m10: c2[2], m11: c2[3],
            m12: c3[0], m13: c3[1], m14: c3[2], m15: c3[3],
        }
    }
}

// TODO(cmc): remove dupes
impl Into<RawMat4> for Mat4 {
    #[rustfmt::skip]
    fn into(self) -> RawMat4 {
        let c0 = self.0.x;
        let c1 = self.0.y;
        let c2 = self.0.z;
        let c3 = self.0.w;
        RawMat4 {
            m0:  c0[0], m1:  c0[1], m2:  c0[2], m3: c0[3],
            m4:  c1[0], m5:  c1[1], m6:  c1[2], m7: c1[3],
            m8:  c2[0], m9:  c2[1], m10: c2[2], m11: c2[3],
            m12: c3[0], m13: c3[1], m14: c3[2], m15: c3[3],
        }
    }
}

// -----------------------------------------------------------------------------

#[derive(Debug, Clone, Copy)]
pub struct Vec2(CGVec2<f32>);

/* Deref to cgmath */

impl Deref for Vec2 {
    type Target = CGVec2<f32>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl DerefMut for Vec2 {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

/* Instantiate from cgmath */

impl<T> From<T> for Vec2
where
    T: Into<CGVec2<f32>>,
{
    fn from(m: T) -> Self {
        Self(m.into())
    }
}

/* Instantiate from Raylib Rust */

// impl From<RVec2> for Vec2 {
//     fn from(v: RVec2) -> Self {
//         let v = CGVec2::new(v.x, v.y);
//         Self(v)
//     }
// }
// impl From<&RVec2> for Vec2 {
//     fn from(m: &RVec2) -> Self {
//         Self::from(m.clone())
//     }
// }

/* Instantiate from Raylib C */

// impl From<RawVec2> for Vec2 {
//     fn from(m: RawVec2) -> Self {
//         Self::from(RVec2::from(m))
//     }
// }
// impl From<&RawVec2> for Vec2 {
//     fn from(m: &RawVec2) -> Self {
//         Self::from(m.clone())
//     }
// }

/* Feed directly to Raylib C */

impl Into<RawVec2> for Vec2 {
    fn into(self) -> RawVec2 {
        RawVec2 {
            x: self.0.x,
            y: self.0.y,
        }
    }
}

// -----------------------------------------------------------------------------

#[derive(Debug, Clone, Copy)]
pub struct Vec2i(CGVec2<i32>);

/* Deref to cgmath */

impl Deref for Vec2i {
    type Target = CGVec2<i32>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl DerefMut for Vec2i {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

/* Instantiate from cgmath */

impl<T> From<T> for Vec2i
where
    T: Into<CGVec2<i32>>,
{
    fn from(m: T) -> Self {
        Self(m.into())
    }
}

/* Instantiate from Raylib Rust */

// impl From<RVec2> for Vec2i {
//     fn from(v: RVec2) -> Self {
//         let v = CGVec2::new(v.x as i32, v.y as i32);
//         Self(v)
//     }
// }
// impl From<&RVec2> for Vec2i {
//     fn from(m: &RVec2) -> Self {
//         Self::from(m.clone())
//     }
// }

/* Instantiate from Raylib C */

// impl From<RawVec2> for Vec2i {
//     fn from(m: RawVec2) -> Self {
//         Self::from(RVec2::from(m))
//     }
// }
// impl From<&RawVec2> for Vec2i {
//     fn from(m: &RawVec2) -> Self {
//         Self::from(m.clone())
//     }
// }

/* Feed directly to Raylib C */

impl Into<RawVec2> for Vec2i {
    fn into(self) -> RawVec2 {
        RawVec2 {
            x: self.0.x as f32,
            y: self.0.y as f32,
        }
    }
}

// -----------------------------------------------------------------------------

#[derive(Debug, Clone, Copy)]
pub struct Point3(CGPoint3<f32>);

/* Deref to cgmath */

impl Deref for Point3 {
    type Target = CGPoint3<f32>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl DerefMut for Point3 {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

/* Instantiate from cgmath */

impl<T> From<T> for Point3
where
    T: Into<CGPoint3<f32>>,
{
    fn from(m: T) -> Self {
        Self(m.into())
    }
}

impl Into<Vec3> for Point3 {
    fn into(self) -> Vec3 {
        CGVec3::new(self.0.x, self.0.y, self.0.z).into()
    }
}

/* Instantiate from Raylib Rust */

// impl From<RPoint3> for Point3 {
//     fn from(v: RPoint3) -> Self {
//         let v = CGPoint3::new(v.x, v.y, v.z);
//         Self(v)
//     }
// }
// impl From<&RPoint3> for Point3 {
//     fn from(m: &RPoint3) -> Self {
//         Self::from(m.clone())
//     }
// }

/* Instantiate from Raylib C */

// impl From<RawPoint3> for Point3 {
//     fn from(m: RawPoint3) -> Self {
//         Self::from(RPoint3::from(m))
//     }
// }
// impl From<&RawPoint3> for Point3 {
//     fn from(m: &RawPoint3) -> Self {
//         Self::from(m.clone())
//     }
// }

/* Feed directly to Raylib C */

impl Into<RawVec3> for Point3 {
    fn into(self) -> RawVec3 {
        RawVec3 {
            x: self.0.x,
            y: self.0.y,
            z: self.0.z,
        }
    }
}

// -----------------------------------------------------------------------------

#[derive(Debug, Clone, Copy)]
pub struct Vec3(CGVec3<f32>);

/* Deref to cgmath */

impl Deref for Vec3 {
    type Target = CGVec3<f32>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl DerefMut for Vec3 {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

/* Instantiate from cgmath */

impl<T> From<T> for Vec3
where
    T: Into<CGVec3<f32>>,
{
    fn from(m: T) -> Self {
        Self(m.into())
    }
}

/* Instantiate from Raylib Rust */

// impl From<RVec3> for Vec3 {
//     fn from(v: RVec3) -> Self {
//         let v = CGVec3::new(v.x, v.y, v.z);
//         Self(v)
//     }
// }
// impl From<&RVec3> for Vec3 {
//     fn from(m: &RVec3) -> Self {
//         Self::from(m.clone())
//     }
// }

/* Instantiate from Raylib C */

// impl From<RawVec3> for Vec3 {
//     fn from(m: RawVec3) -> Self {
//         Self::from(RVec3::from(m))
//     }
// }
// impl From<&RawVec3> for Vec3 {
//     fn from(m: &RawVec3) -> Self {
//         Self::from(m.clone())
//     }
// }

/* Feed directly to Raylib C */

impl Into<RawVec3> for Vec3 {
    fn into(self) -> RawVec3 {
        RawVec3 {
            x: self.0.x,
            y: self.0.y,
            z: self.0.z,
        }
    }
}

// -----------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Vec3i(CGVec3<i32>);

/* Deref to cgmath */

impl Deref for Vec3i {
    type Target = CGVec3<i32>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl DerefMut for Vec3i {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

/* Instantiate from cgmath */

impl<T> From<T> for Vec3i
where
    T: Into<CGVec3<i32>>,
{
    fn from(m: T) -> Self {
        Self(m.into())
    }
}

/* Instantiate from Raylib Rust */

// impl From<RVec3> for Vec3i {
//     fn from(v: RVec3) -> Self {
//         let v = CGVec3::new(v.x as i32, v.y as i32);
//         Self(v)
//     }
// }
// impl From<&RVec3> for Vec3i {
//     fn from(m: &RVec3) -> Self {
//         Self::from(m.clone())
//     }
// }

/* Instantiate from Raylib C */

// impl From<RawVec3> for Vec3i {
//     fn from(m: RawVec3) -> Self {
//         Self::from(RVec3::from(m))
//     }
// }
// impl From<&RawVec3> for Vec3i {
//     fn from(m: &RawVec3) -> Self {
//         Self::from(m.clone())
//     }
// }

/* Feed directly to Raylib C */

impl Into<RawVec3> for Vec3i {
    fn into(self) -> RawVec3 {
        RawVec3 {
            x: self.0.x as f32,
            y: self.0.y as f32,
            z: self.0.z as f32,
        }
    }
}

// -----------------------------------------------------------------------------

#[derive(Debug, Clone, Copy)]
pub struct Vec4(CGVec4<f32>);

/* Deref to cgmath */

impl Deref for Vec4 {
    type Target = CGVec4<f32>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl DerefMut for Vec4 {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

/* Instantiate from cgmath */

impl<T> From<T> for Vec4
where
    T: Into<CGVec4<f32>>,
{
    fn from(m: T) -> Self {
        Self(m.into())
    }
}

/* Instantiate from Raylib Rust */

// impl From<RVec4> for Vec4 {
//     fn from(v: RVec4) -> Self {
//         let v = CGVec4::new(v.x, v.y, v.z, v.w);
//         Self(v)
//     }
// }
// impl From<&RVec4> for Vec4 {
//     fn from(m: &RVec4) -> Self {
//         Self::from(m.clone())
//     }
// }

/* Instantiate from Raylib C */

// impl From<RawVec4> for Vec4 {
//     fn from(m: RawVec4) -> Self {
//         Self::from(RVec4::from(m))
//     }
// }
// impl From<&RawVec4> for Vec4 {
//     fn from(m: &RawVec4) -> Self {
//         Self::from(m.clone())
//     }
// }

/* Feed directly to Raylib C */

impl Into<RawVec4> for Vec4 {
    fn into(self) -> RawVec4 {
        RawVec4 {
            x: self.0.x,
            y: self.0.y,
            z: self.0.z,
            w: self.0.w,
        }
    }
}
