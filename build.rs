fn main() {
    tonic_prost_build::configure()
        .type_attribute(
            ".sila2.org.silastandard.CreateBinaryRequest",
            "#[derive(serde::Serialize, serde::Deserialize)]",
        )
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
