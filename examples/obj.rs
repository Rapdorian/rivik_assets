use std::env;

use log::error;
use mint::{Point2, Point3};
use reerror::{throw, Context, StatusCode};
use rivik_assets::{
    formats::mesh::{obj::ObjScene, Scene},
    load, Result,
};

fn run() -> Result<()> {
    let mut args = env::args().skip(1);
    // grab asset uri
    if let Some(uri) = args.next() {
        let obj = throw!(load(&uri, ObjScene));
        // print contents of obj file
        for mesh in &obj.nodes {
            println!("{}", mesh.1);
            for v in &mesh.0.verts {
                println!("\t{},{},{}", v.x, v.y, v.z);
            }
        }
    } else {
        eprintln!("Missing argument: Filename URI");
    }

    Ok(())
}

fn main() {
    env_logger::init();

    if let Err(e) = run() {
        eprintln!("{e}");
    }

    if let Err(e) = run() {
        eprintln!("{e}");
    }
}
