use anyhow::Result;
use std::fs;

fn main() -> Result<()> {
    fs::create_dir_all("src/pb")?;
    /* let builder = tonic_build::configure();
    builder.out_dir("src/pb").compile(
        &[
            "../protos/user-stats/messages.proto",
            "../protos/user-stats/rpc.proto",
        ],
        &["../protos"],
    )?; */
    let build = tonic_build::configure();
    build.out_dir("src/pb").compile(
        &[
            "../protos/notification/messages.proto",
            "../protos/notification/rpc.proto",
        ],
        &["../protos"],
    )?;
    Ok(())
}
