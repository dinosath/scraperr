fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_prost_build::configure()
        .build_server(true)
        .build_client(false)
        .compile_protos(
            &["../../proto/auth.proto", "../../proto/jobs.proto"],
            &["../../proto"],
        )?;
    Ok(())
}
