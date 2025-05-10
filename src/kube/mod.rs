use colored::Colorize;
use kube::{config::Kubeconfig, Client, Config};
use kube::api::Api;
use k8s_openapi::api::core::v1::Node;
use std::io::{self, stdout, Write};

use crate::aws::{extract_aws_profile, extract_aws_region, get_ec2_instance_id, start_ssm_session};

//load kube config from local home dir
fn load_kube_config() -> Kubeconfig {
    let kubeconfig = Kubeconfig::read().unwrap();
    println!("---------------------------------------------");
    println!("{}", "Available contexts:".green());
    for context in &kubeconfig.contexts {
        println!("{}", context.name);
    }

    return kubeconfig;
}

//allow user to choose a k8s context
fn choose_context() -> Result<Kubeconfig, String> {
    let mut kubeconfig = load_kube_config();

    print!("{}", "Select a context: ".yellow());
    io::stdout().flush().unwrap();

    let mut selected_context = String::new();
    io::stdin().read_line(&mut selected_context).unwrap();
    let selected_context = selected_context.trim();
    println!("---------------------------------------------");

    if kubeconfig.contexts.iter().any(|ctx| ctx.name == selected_context) {
        kubeconfig.current_context = Some(selected_context.to_string());
        Ok(kubeconfig)
    } else {
        let error_message = format!("Context '{}' not found in kubeconfig", selected_context);
        Err(error_message.red().to_string())
    }
}

//create the client config
pub async fn get_client_config(kubeconfig: Kubeconfig) -> Result<Config, String> {
    Config::from_custom_kubeconfig(kubeconfig, &Default::default())
    .await
    .map_err(|err| format!("Failed to build Kubernetes client config: {}", err))
}

//list k8s nodes and extract the metadata.name, this is the private dns name of the ec2 node
pub async fn list_node_ips(config: Config) -> Result<Vec<String>, String> {
    let client = Client::try_from(config).map_err(|err| format!("Error creating Kubernetes client: {}", err))?;

    let nodes_api: Api<Node> = Api::all(client);

    let nodes = nodes_api
        .list(&Default::default())
        .await
        .map_err(|err| format!("Error listing nodes: {}", err))?;

    let nodes_list: Vec<String> = nodes
        .items
        .into_iter()
        .filter_map(|node| node.metadata.name)
        .collect();

    Ok(nodes_list)
}

//exec into the node
pub async fn exec_into_node() {
    let kubeconfig = match choose_context() {
        Ok(kubeconfig) => kubeconfig,
        Err(error) => {
            eprintln!("{}", error);
            return;
        }
    };

    let client_config = match get_client_config(kubeconfig).await {
        Ok(client_config) => client_config,
        Err(error) => {
            eprintln!("{}", error);
            return;
        }
    };

    let eks_node_ips = match list_node_ips(client_config.clone()).await {
        Ok(eks_node_ips) => eks_node_ips,
        Err(error) => {
            eprintln!("{}", error);
            return;
        }
    };

    let aws_profile = extract_aws_profile(client_config.clone());
    let aws_region = extract_aws_region(client_config).unwrap();

    for eks_node_ip in eks_node_ips {
        println!("{}", eks_node_ip);
    }

    print!("{}", "Select a node: ".yellow());
    stdout().flush().unwrap();
    let mut node_private_ip = String::new();
    io::stdin().read_line(&mut node_private_ip).unwrap();
    let node_private_ip = node_private_ip.trim();
    println!("---------------------------------------------");

    let instance_id = match get_ec2_instance_id(node_private_ip.to_string(), aws_profile.clone()).await {
        Ok(instance_id) => instance_id,
        Err(error) => {
            eprintln!("{}", error);
            return;
        }
    };

    println!("{}", "Connecting to the EC2 instance using SSM...".yellow());
    println!("{}{}", "Instance id: ".green(), instanc_id.green());

    if let Err(err) = start_ssm_session(instanc_id, aws_region, aws_profile) {
        eprintln!("Failed to start SSM session: {}", err);
    }
}