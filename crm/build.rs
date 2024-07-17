use anyhow::Result;
use proto_builder_trait::tonic::BuilderAttributes;
use std::fs;

fn main() -> Result<()> {
    fs::create_dir_all("src/pb")?;
    let build = tonic_build::configure();
    build
        .out_dir("src/pb")
        .with_derive_builder(&["WelcomeRequest", "RecallRequest", "RemindRequest"], None)
        .with_field_attributes(
            &["WelcomeRequest.contents_ids"],
            &[r#"#[builder(setter(each(name="contents_id", into)))]"#],
        )
        .compile(
            &["../protos/crm/messages.proto", "../protos/crm/rpc.proto"],
            &["../protos"],
        )?;
    Ok(())
}
