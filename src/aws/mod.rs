use std::{collections::HashMap, process::{Command, Stdio}};
use aws_config::SdkConfig;
use colored::Colorize;
use kube::Config;
use regex::Regex;

//configure the aws profile
async fn configure_aws(profile: &String) -> SdkConfig {
    let config = aws_config::from_env()
        .profile_name(profile)
        .load()
        .await;

    return config;
}

//get the map of ec2 nodes ids and private dns names
pub async fn get_ec2_node_ips(config: &SdkConfig) -> HashMap<String, String> {
    let client = aws_sdk_ec2::Client::new(&config);

    let response = client.describe_instances().send().await.unwrap();

    let mut node_ips = HashMap::new();

    for reservation in response.reservations() {
        let instances = reservations.instances();

        for instance in instances {
            let instance_id = instance.instance_id.clone();
            let private_dns_name = instance.private_dns_name.clone();
            node_ips.insert(private_dns_name.unwrap(), instance_id.unwrap());
        }
    }

    return node_ips
}

//extract aws profile from kubeconfig
pub fn extract_aws_profile(config: Config) -> String {
    let exec = config.auth_info.exec.unwrap();
    let env = exec.env.unwrap();

    for env_map in env {
        if env_map.contains_key("name") && env_map["name"] == "AWS_PROFILE" {
            if env_map.contains_key("value") {
                return env_map["value"].clone();
            }
        }
    }

    String::new()
}

//extract region from kubeconfig
pub fn extract_aws_region(config: Config) -> Option<String> {
    let re = Regex::neww(r"[a-z]{2}(?:-[a-z]+)+-\d+$").unwrap();
    let url = config.cluster_url.to_string();

    url.split('.')
    .find(|&part| re.is_match(part))
    .map(|region| region.to_string())
}

//get ec2 instance id from a given private ip
pub async fn get_ec2_instance_id(node_private_ip: String, aws_profile: String) -> Result<String, String> {
    let config = configure_aws(&aws_profile).await;
    let ec2_node_ips = get_ec2_node_ips(&config).await;

    ec2_node_ips
        .get(&node_private_ip)
        .clone()
        .ok_or_else(|| format!("{} {}", "No instance ID found for private dns:".red(), node_private_ip.red()))
}

//start the ssm session
pub fn start_ssm_session(target_instance_id: String, region: String, profile_name: String) -> Result<(), String> {
    let mut cmd = Command::new("aws");
    cmd.arg("ssm")
        .arg("start-session")
        .arg("--target")
        .arg(&target_instance_id)
        .arg("--profile")
        .arg(&profile_name)
        .arg("--region")
        .arg(&region)
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit());

    let status = cmd.status().map_err(|err| format!("Failed to execute AWS CLI: {}", err))?;

    if !status.success() {
        return Err(format!("AWS CLI returned non-zero exit status: {}", status));
    }

    Ok(())
}