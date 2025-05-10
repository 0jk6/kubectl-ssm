mod aws;
mod kube;

use kube::exec_into_node;

#[tokio::main]
async fn main() -> Result<(), aws_sdk_ec2::Error> {
    let _ = rustls::crypto::aws_lc_rs::default_provider().install_default();

    exec_into_node().await;

    Ok(())
}
