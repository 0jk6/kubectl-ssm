# kubectl-ssm
This is a simple kubectl plugin to connect to a Kubernetes node using AWS SSM.

#### Requirements
- kubectl
- AWS CLI
- AWS Session Manager Plugin

#### Build
Ensure that you have Rust installed on your machine.

```bash
cargo build --release
```

Copy the binary into a folder that is configured in the `$PATH` variable.

```bash
cp ./target/release/kubectl-ssm /path/to/bin
```

Now you can run the following command to connect to a Kubernetes node using AWS SSM.

```bash
kubectl ssm
```