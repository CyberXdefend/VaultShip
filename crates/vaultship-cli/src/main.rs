mod commands;
mod config;

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "vaultship")]
#[command(
    about = "Protected container runtime - encrypt, bind, sign, and harden Docker containers"
)]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Init {
        #[arg(long, default_value = "baseline")]
        profile: String,
    },
    Build {
        #[arg(default_value = ".")]
        path: String,
    },
    Encrypt {
        input: String,
        #[arg(long)]
        output: Option<String>,
        #[arg(long, default_value = "vaultship.layer.key")]
        key_file: String,
    },
    Decrypt {
        input: String,
        #[arg(long)]
        output: Option<String>,
        #[arg(long, default_value = "vaultship.layer.key")]
        key_file: String,
    },
    Push {
        registry: String,
    },
    Pull {
        image: String,
    },
    Run {
        image: String,
        #[arg(long)]
        bind_file: Option<String>,
        #[arg(long, default_value = "vaultship.public.key")]
        public_key: String,
        #[arg(long, default_value = "docker")]
        engine: String,
        #[arg(long)]
        dry_run: bool,
        #[arg(last = true)]
        extra_args: Vec<String>,
        #[arg(long)]
        license: Option<String>,
    },
    Harden {
        #[arg(default_value = "docker-compose.yml")]
        compose_file: String,
    },
    Inspect {
        image: String,
        #[arg(long)]
        json: bool,
    },
    Keygen {
        #[arg(long, default_value = "vaultship")]
        name: String,
    },
    Bind {
        #[arg(long, default_value = "vaultship.layer.key")]
        key_file: String,
        #[arg(long, default_value = "vaultship.private.key")]
        private_key: String,
        #[arg(long)]
        fingerprint: Option<String>,
        #[arg(long, default_value = "vaultship.bind.json")]
        output: String,
    },
    Fingerprint,
    Verify {
        image_or_ref: String,
        #[arg(long)]
        json: bool,
        #[arg(long, default_value = "vaultship.public.key")]
        public_key: String,
    },
    RotateKey {
        input: String,
        #[arg(long)]
        old_key: String,
        #[arg(long)]
        new_key: String,
        #[arg(long)]
        output: Option<String>,
    },
    License {
        #[command(subcommand)]
        action: commands::license::LicenseCommands,
    },
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let code = match run_cli().await {
        Ok(_) => 0,
        Err(e) => {
            let msg = e.to_string();
            if std::env::var("VAULTSHIP_OUTPUT")
                .map(|v| v.eq_ignore_ascii_case("json"))
                .unwrap_or(false)
            {
                println!(
                    "{}",
                    serde_json::to_string(&serde_json::json!({
                        "ok": false,
                        "error": msg,
                        "exit_code": map_exit_code(&e)
                    }))
                    .unwrap_or_else(|_| "{\"ok\":false}".to_string())
                );
            } else {
                eprintln!("ERROR: {msg}");
            }
            map_exit_code(&e)
        }
    };
    std::process::exit(code);
}

async fn run_cli() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Init { profile } => commands::init::run(&profile).await,
        Commands::Build { path } => commands::build::run(&path).await,
        Commands::Encrypt {
            input,
            output,
            key_file,
        } => commands::encrypt::run(&input, output.as_deref(), &key_file).await,
        Commands::Decrypt {
            input,
            output,
            key_file,
        } => commands::decrypt::run(&input, output.as_deref(), &key_file).await,
        Commands::Push { registry } => commands::push::run(&registry).await,
        Commands::Pull { image } => commands::pull::run(&image).await,
        Commands::Run {
            image,
            bind_file,
            public_key,
            engine,
            dry_run,
            extra_args,
            license,
        } => {
            commands::run::run(
                &image,
                bind_file.as_deref(),
                &public_key,
                dry_run,
                &extra_args,
                license.as_deref(),
                &engine,
            )
            .await
        }
        Commands::Harden { compose_file } => commands::harden::run(&compose_file).await,
        Commands::Inspect { image, json } => commands::inspect::run(&image, json).await,
        Commands::Keygen { name } => commands::keygen::run(&name).await,
        Commands::Bind {
            key_file,
            private_key,
            fingerprint,
            output,
        } => commands::bind::create(&key_file, &private_key, fingerprint.as_deref(), &output).await,
        Commands::Fingerprint => commands::fingerprint::run().await,
        Commands::Verify {
            image_or_ref,
            json,
            public_key,
        } => commands::verify::run(&image_or_ref, json, &public_key).await,
        Commands::RotateKey {
            input,
            old_key,
            new_key,
            output,
        } => commands::rotate_key::run(&input, &old_key, &new_key, output.as_deref()).await,
        Commands::License { action } => commands::license::run(action).await,
    }
}

fn map_exit_code(err: &anyhow::Error) -> i32 {
    let msg = err.to_string().to_lowercase();
    if msg.contains("fingerprint mismatch") || msg.contains("bound key") {
        21
    } else if msg.contains("signature") || msg.contains("verify") {
        22
    } else if msg.contains("registry") || msg.contains("manifest") || msg.contains("blob") {
        23
    } else if msg.contains("compose") || msg.contains("harden") {
        24
    } else if msg.contains("docker run failed") || msg.contains("podman") || msg.contains("nerdctl")
    {
        25
    } else {
        1
    }
}
