fn main() {
    tonic_prost_build::configure()
        .compile_protos(
            &["proto/SimpleStructure.proto"],
            &["proto", "sila_base/protobuf"],
        )
        .unwrap();
}
