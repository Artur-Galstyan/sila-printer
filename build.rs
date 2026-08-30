fn main() {
    tonic_prost_build::configure()
        .compile_protos(
            &[
                "sila_base/protobuf/SiLAFramework.proto",
                "sila_base/protobuf/SiLABinaryTransfer.proto",
                "proto/PrintingControl.proto",
            ],
            &["sila_base/protobuf", "proto"],
        )
        .unwrap();
}
