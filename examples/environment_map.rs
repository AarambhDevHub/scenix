use scenix::{CubeCamera, CubeFace, Vec3};

fn main() {
    let cube_camera = CubeCamera::new(Vec3::new(0.0, 1.5, 0.0), 0.1, 50.0);
    for face in CubeFace::all() {
        let matrix = cube_camera.view_projection(face);
        println!("{face:?}: first column = {:?}", matrix.cols[0]);
    }
}
