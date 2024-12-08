use std::env;
use std::path::PathBuf;

fn main() {
    println!("cargo:rustc-link-lib=sdl3");

    // Always
    {
        let bindings = bindgen::Builder::default()
            // The input header we would like to generate
            // bindings for.
            .header("wrapper.h")
            // Tell cargo to invalidate the built crate whenever any of the
            // included header files changed.
            .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
            // Finish the builder and generate the bindings.
            .generate()
            // Unwrap the Result and panic on failure.
            .expect("Unable to generate SDL bindings");

        // Write the bindings to the $OUT_DIR/bindings.rs file.
        let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
        bindings
            .write_to_file(out_path.join("sdl_bindings.rs"))
            .expect("Couldn't write SDL bindings!");
    }

    #[cfg(feature = "main")]
    {
        let bindings_main = bindgen::Builder::default()
            // The input header we would like to generate
            // bindings for.
            .header("wrapper_main.h")
            // Tell cargo to invalidate the built crate whenever any of the
            // included header files changed.
            .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
            // Finish the builder and generate the bindings.
            .generate()
            // Unwrap the Result and panic on failure.
            .expect("Unable to generate SDL_main bindings");

        // Write the bindings to the $OUT_DIR/bindings.rs file.
        let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
        bindings_main
            .write_to_file(out_path.join("sdl_main_bindings.rs"))
            .expect("Couldn't write SDL_main bindings!");
    }
}