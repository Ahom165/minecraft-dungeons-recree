//! Types de sommets + constructeur de cuboïdes (style Minecraft) et quads.

use glam::{Mat4, Vec3, Vec4};

/// Sommet du monde (statique et dynamique, même layout). 80 octets.
#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct DynVertex {
    pub pos: [f32; 3],
    pub nrm: [f32; 3],
    pub uv: [f32; 2],
    pub tint: [f32; 4],
    pub uv_rect: [f32; 4],
    pub param: [f32; 4],
}

/// Sommet d'interface (ortho). 32 octets.
#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct UiVertex {
    pub pos: [f32; 2],
    pub uv: [f32; 2],
    pub color: [f32; 4],
}

/// Instance de billboard (particules). 48 octets.
#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct FxInstance {
    pub center_size: [f32; 4],
    pub color: [f32; 4],
    pub uv_rect: [f32; 4],
}

/// Ordre des faces : +X, -X, +Y (dessus), -Y (dessous), +Z (avant), -Z (arrière).
pub type FaceSlots = [usize; 6];

/// Rect UV absolu d'une case de l'atlas 1024² (grille 8×8).
/// Format : [u0, v0, largeur, hauteur] — la face échantillonne SA case, sans wrap.
#[inline]
pub const fn slot_rect(slot: usize) -> [f32; 4] {
    let s = 1.0 / 8.0;
    let cx = (slot % 8) as f32;
    let cy = (slot / 8) as f32;
    [cx * s, cy * s, s, s]
}

/// Ajoute un cuboïde transformé par `mat` (translation * rotation), demi-taille `half`.
/// `param` = [émissif, alpha_suppl, défilement_uv_x, défilement_uv_y].
#[allow(clippy::too_many_arguments)]
pub fn push_box(
    out: &mut Vec<DynVertex>,
    mat: Mat4,
    half: Vec3,
    slots: FaceSlots,
    tint: Vec4,
    param: [f32; 4],
) {
    let hx = half.x;
    let hy = half.y;
    let hz = half.z;
    let faces: [([Vec3; 4], Vec3, usize); 6] = [
        (
            [
                Vec3::new(hx, -hy, hz),
                Vec3::new(hx, -hy, -hz),
                Vec3::new(hx, hy, -hz),
                Vec3::new(hx, hy, hz),
            ],
            Vec3::X,
            slots[0],
        ),
        (
            [
                Vec3::new(-hx, -hy, -hz),
                Vec3::new(-hx, -hy, hz),
                Vec3::new(-hx, hy, hz),
                Vec3::new(-hx, hy, -hz),
            ],
            Vec3::NEG_X,
            slots[1],
        ),
        (
            [
                Vec3::new(-hx, hy, hz),
                Vec3::new(hx, hy, hz),
                Vec3::new(hx, hy, -hz),
                Vec3::new(-hx, hy, -hz),
            ],
            Vec3::Y,
            slots[2],
        ),
        (
            [
                Vec3::new(-hx, -hy, -hz),
                Vec3::new(hx, -hy, -hz),
                Vec3::new(hx, -hy, hz),
                Vec3::new(-hx, -hy, hz),
            ],
            Vec3::NEG_Y,
            slots[3],
        ),
        (
            [
                Vec3::new(-hx, -hy, hz),
                Vec3::new(hx, -hy, hz),
                Vec3::new(hx, hy, hz),
                Vec3::new(-hx, hy, hz),
            ],
            Vec3::Z,
            slots[4],
        ),
        (
            [
                Vec3::new(hx, -hy, -hz),
                Vec3::new(-hx, -hy, -hz),
                Vec3::new(-hx, hy, -hz),
                Vec3::new(hx, hy, -hz),
            ],
            Vec3::NEG_Z,
            slots[5],
        ),
    ];
    // UV : v=0 en haut de la texture, v=1 en bas.
    // Topologie TriangleList : 6 sommets par face (2 triangles), sans index buffer.
    let uvs: [[f32; 2]; 4] = [[0.0, 1.0], [1.0, 1.0], [1.0, 0.0], [0.0, 0.0]];
    for (corners, nrm, slot) in faces {
        let rect = slot_rect(slot);
        let n = mat.transform_vector3(nrm).normalize_or_zero();
        let mut verts = [DynVertex {
            pos: [0.0; 3],
            nrm: [0.0; 3],
            uv: [0.0; 2],
            tint: tint.to_array(),
            uv_rect: rect,
            param,
        }; 4];
        for (v, (c, uv)) in verts.iter_mut().zip(corners.iter().zip(uvs)) {
            let p = mat.transform_point3(*c);
            *v = DynVertex {
                pos: [p.x, p.y, p.z],
                nrm: [n.x, n.y, n.z],
                uv,
                tint: tint.to_array(),
                uv_rect: rect,
                param,
            };
        }
        // Triangles (0,1,2) et (0,2,3) — sens anti-horaire vu de l'extérieur.
        out.extend_from_slice(&[verts[0], verts[1], verts[2], verts[0], verts[2], verts[3]]);
    }
}

/// Ajoute un quad explicite (sol, eau, ombres…). `corners` doit être CCW vu du dessus.
/// Topologie TriangleList : 6 sommets (2 triangles), sans index buffer.
#[allow(clippy::too_many_arguments)]
pub fn push_quad(
    out: &mut Vec<DynVertex>,
    corners: [Vec3; 4],
    nrm: Vec3,
    slot: usize,
    tint: Vec4,
    param: [f32; 4],
) {
    let rect = slot_rect(slot);
    let uvs: [[f32; 2]; 4] = [[0.0, 1.0], [1.0, 1.0], [1.0, 0.0], [0.0, 0.0]];
    let mut verts = [DynVertex {
        pos: [0.0; 3],
        nrm: [0.0; 3],
        uv: [0.0; 2],
        tint: tint.to_array(),
        uv_rect: rect,
        param,
    }; 4];
    for (v, (c, uv)) in verts.iter_mut().zip(corners.iter().zip(uvs)) {
        *v = DynVertex {
            pos: [c.x, c.y, c.z],
            nrm: [nrm.x, nrm.y, nrm.z],
            uv,
            tint: tint.to_array(),
            uv_rect: rect,
            param,
        };
    }
    out.extend_from_slice(&[verts[0], verts[1], verts[2], verts[0], verts[2], verts[3]]);
}
