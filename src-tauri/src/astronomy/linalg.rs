//! Vecteurs et matrices 3×3, juste ce qu'il faut pour les changements de repère.

pub type Vec3 = [f64; 3];

pub fn norm(v: Vec3) -> f64 {
    (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt()
}

pub fn normalize(v: Vec3) -> Vec3 {
    scale(v, 1.0 / norm(v))
}

pub fn add(a: Vec3, b: Vec3) -> Vec3 {
    [a[0] + b[0], a[1] + b[1], a[2] + b[2]]
}

pub fn sub(a: Vec3, b: Vec3) -> Vec3 {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

pub fn scale(v: Vec3, k: f64) -> Vec3 {
    [v[0] * k, v[1] * k, v[2] * k]
}

pub fn dot(a: Vec3, b: Vec3) -> f64 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

/// Matrice de rotation 3×3, rangée par lignes.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Mat3(pub [[f64; 3]; 3]);

impl Mat3 {
    /// Rotation active d'angle `angle_deg` autour de l'axe X.
    pub fn rotation_x(angle_deg: f64) -> Mat3 {
        let (s, c) = angle_deg.to_radians().sin_cos();
        Mat3([[1.0, 0.0, 0.0], [0.0, c, -s], [0.0, s, c]])
    }

    /// Rotation active d'angle `angle_deg` autour de l'axe Y.
    pub fn rotation_y(angle_deg: f64) -> Mat3 {
        let (s, c) = angle_deg.to_radians().sin_cos();
        Mat3([[c, 0.0, s], [0.0, 1.0, 0.0], [-s, 0.0, c]])
    }

    /// Rotation active d'angle `angle_deg` autour de l'axe Z.
    pub fn rotation_z(angle_deg: f64) -> Mat3 {
        let (s, c) = angle_deg.to_radians().sin_cos();
        Mat3([[c, -s, 0.0], [s, c, 0.0], [0.0, 0.0, 1.0]])
    }

    pub fn mul(&self, other: &Mat3) -> Mat3 {
        let a = &self.0;
        let b = &other.0;
        let mut out = [[0.0; 3]; 3];
        for (i, row) in out.iter_mut().enumerate() {
            for (j, cell) in row.iter_mut().enumerate() {
                *cell = a[i][0] * b[0][j] + a[i][1] * b[1][j] + a[i][2] * b[2][j];
            }
        }
        Mat3(out)
    }

    pub fn apply(&self, v: Vec3) -> Vec3 {
        let m = &self.0;
        [dot(m[0], v), dot(m[1], v), dot(m[2], v)]
    }

    /// Transposée, c'est-à-dire l'inverse pour une rotation.
    pub fn transpose(&self) -> Mat3 {
        let m = &self.0;
        Mat3([
            [m[0][0], m[1][0], m[2][0]],
            [m[0][1], m[1][1], m[2][1]],
            [m[0][2], m[1][2], m[2][2]],
        ])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn close(a: Vec3, b: Vec3) -> bool {
        a.iter().zip(b).all(|(x, y)| (x - y).abs() < 1e-12)
    }

    #[test]
    fn rotates_axes_counterclockwise() {
        assert!(close(Mat3::rotation_z(90.0).apply([1.0, 0.0, 0.0]), [0.0, 1.0, 0.0]));
        assert!(close(Mat3::rotation_x(90.0).apply([0.0, 1.0, 0.0]), [0.0, 0.0, 1.0]));
        assert!(close(Mat3::rotation_y(90.0).apply([0.0, 0.0, 1.0]), [1.0, 0.0, 0.0]));
    }

    #[test]
    fn transpose_inverts_rotation() {
        let m = Mat3::rotation_x(23.4).mul(&Mat3::rotation_z(-41.0));
        let v = [0.3, -0.5, 0.8];
        assert!(close(m.transpose().apply(m.apply(v)), v));
    }
}
