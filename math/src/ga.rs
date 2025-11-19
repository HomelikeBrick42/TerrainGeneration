use bytemuck::NoUninit;

use crate::Vector3;

ga_generator::ga! {
    element_type = f32;
    scalar_name = s;
    elements = [e0 = zero, e1 = positive_one, e2 = positive_one, e3 = positive_one];

    group #[derive(NoUninit)] #[repr(C)] Scalar = s;

    group #[derive(NoUninit)] #[repr(C)] Plane = e1 + e2 + e3;
    group #[derive(NoUninit)] #[repr(C)] Line = Plane ^ Plane;
    group #[derive(NoUninit)] #[repr(C)] Point = Line ^ Plane;

    group #[derive(NoUninit)] #[repr(C)] PgaPlane = e0 + e1 + e2 + e3;
    group #[derive(NoUninit)] #[repr(C)] PgaLine = PgaPlane ^ PgaPlane;
    group #[derive(NoUninit)] #[repr(C)] PgaPoint = PgaLine ^ PgaPlane;
    group #[derive(NoUninit)] #[repr(C)] PgaPseudoscalar = PgaPoint ^ PgaPlane;

    group #[derive(NoUninit)] #[repr(C)] Rotor = Scalar + Line;
    group #[derive(NoUninit)] #[repr(C)] Transform = Scalar + PgaLine + PgaPseudoscalar;

    fn rotor_reverse(rotor: Rotor) -> Rotor {
        return ~rotor;
    }

    fn rotor_then(rotor: Rotor, then: Rotor) -> Rotor {
        return rotor * then;
    }

    fn rotor_point(rotor: Rotor, x: Scalar, y: Scalar, z: Scalar) -> [Scalar, Scalar, Scalar] {
        let x = e1 - x*e0;
        let y = e2 - y*e0;
        let z = e3 - z*e0;
        let point = (x ^ y) ^ z;

        let transformed = (~rotor * point) * rotor;
        let assume_normalised = point | (1 - (~rotor * rotor));
        let result = transformed + assume_normalised;

        return [
            e1 & result,
            e2 & result,
            e3 & result,
        ];
    }

    fn transform_reverse(transform: Transform) -> Transform {
        return ~transform;
    }

    fn transform_then(transform: Transform, then: Transform) -> Transform {
        return transform * then;
    }

    fn transform_normal(transform: Transform, x: Scalar, y: Scalar, z: Scalar) -> [Scalar, Scalar, Scalar] {
        let x = e1 - x*e0;
        let y = e2 - y*e0;
        let z = e3 - z*e0;
        let origin = (e1 ^ e2) ^ e3;
        let point = (((x ^ y) ^ z) & origin) ^ e0;

        let transformed = (~transform * point) * transform;
        let assume_normalised = point | (1 - (~transform * transform));
        let result = transformed + assume_normalised;

        return [
            e1 & result,
            e2 & result,
            e3 & result,
        ];
    }

    fn transform_point(transform: Transform, x: Scalar, y: Scalar, z: Scalar) -> [Scalar, Scalar, Scalar] {
        let x = e1 - x*e0;
        let y = e2 - y*e0;
        let z = e3 - z*e0;
        let point = (x ^ y) ^ z;

        let transformed = (~transform * point) * transform;
        let assume_normalised = point | (1 - (~transform * transform));
        let result = transformed + assume_normalised;

        return [
            e1 & result,
            e2 & result,
            e3 & result,
        ];
    }
}

impl Rotor {
    pub const IDENTITY: Self = Self {
        s: 1.0,
        e1e2: 0.0,
        e1e3: 0.0,
        e2e3: 0.0,
    };

    pub fn rotation_xy(angle: f32) -> Self {
        let (sin, cos) = (angle * 0.5).sin_cos();
        Self {
            s: cos,
            e1e2: sin,
            ..Self::zero()
        }
    }

    pub fn rotation_xz(angle: f32) -> Self {
        let (sin, cos) = (angle * 0.5).sin_cos();
        Self {
            s: cos,
            e1e3: sin,
            ..Self::zero()
        }
    }

    pub fn rotation_yz(angle: f32) -> Self {
        let (sin, cos) = (angle * 0.5).sin_cos();
        Self {
            s: cos,
            e2e3: sin,
            ..Self::zero()
        }
    }

    pub fn reverse(self) -> Self {
        rotor_reverse(self)
    }

    pub fn then(self, then: Self) -> Self {
        rotor_then(self, then)
    }

    pub fn rotate_vector(self, point: Vector3<f32>) -> Vector3<f32> {
        let (Scalar { s: x }, Scalar { s: y }, Scalar { s: z }) = rotor_point(
            self,
            Scalar { s: point.x },
            Scalar { s: point.y },
            Scalar { s: point.z },
        );
        Vector3 { x, y, z }
    }
}

impl Transform {
    pub const IDENTITY: Self = Self {
        s: 1.0,
        e0e1: 0.0,
        e0e2: 0.0,
        e0e3: 0.0,
        e1e2: 0.0,
        e1e3: 0.0,
        e2e3: 0.0,
        e0e1e2e3: 0.0,
    };

    pub fn translation(offset: Vector3<f32>) -> Self {
        Self {
            e0e1: offset.x * 0.5,
            e0e2: offset.y * 0.5,
            e0e3: offset.z * 0.5,
            ..Self::IDENTITY
        }
    }

    pub fn rotation_xy(angle: f32) -> Self {
        Self::from_rotor(Rotor::rotation_xy(angle))
    }

    pub fn rotation_xz(angle: f32) -> Self {
        Self::from_rotor(Rotor::rotation_xz(angle))
    }

    pub fn rotation_yz(angle: f32) -> Self {
        Self::from_rotor(Rotor::rotation_yz(angle))
    }

    pub fn reverse(self) -> Self {
        transform_reverse(self)
    }

    pub fn then(self, then: Self) -> Self {
        transform_then(self, then)
    }

    pub fn rotate_vector(self, point: Vector3<f32>) -> Vector3<f32> {
        self.rotor_part().rotate_vector(point)
    }

    pub fn transform_vector(self, point: Vector3<f32>) -> Vector3<f32> {
        let (Scalar { s: x }, Scalar { s: y }, Scalar { s: z }) = transform_point(
            self,
            Scalar { s: point.x },
            Scalar { s: point.y },
            Scalar { s: point.z },
        );
        Vector3 { x, y, z }
    }

    pub fn from_rotor(rotor: Rotor) -> Self {
        let Rotor {
            s,
            e1e2,
            e1e3,
            e2e3,
        } = rotor;
        Self {
            s,
            e0e1: 0.0,
            e0e2: 0.0,
            e0e3: 0.0,
            e1e2,
            e1e3,
            e2e3,
            e0e1e2e3: 0.0,
        }
    }

    pub fn rotor_part(self) -> Rotor {
        let Self {
            s,
            e0e1: _,
            e0e2: _,
            e0e3: _,
            e1e2,
            e1e3,
            e2e3,
            e0e1e2e3: _,
        } = self;
        Rotor {
            s,
            e1e2,
            e1e3,
            e2e3,
        }
    }
}
