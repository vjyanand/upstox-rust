fn main() {
    prost_build::compile_protos(&["proto/upstox.proto"], &["proto/"]).unwrap();
}
