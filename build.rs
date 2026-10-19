fn main() {
    // TODO(low): include_optional should be able to do this itself when the `track_path`
    //  feature is stabilized.
    println!("cargo::rerun-if-changed=src/migrations/test_data");
}
