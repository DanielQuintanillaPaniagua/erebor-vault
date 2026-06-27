mod models;
mod config;
mod vault;
mod banner;

use clap::{Parser, Subcommand};
use config::Config;
use models::Entry;
use rpassword::read_password;
use std::fs;
use std::io::{self, Write};

#[derive(Parser)]
#[command(
    name = "erebor",
    about = "⛰️  Erebor Vault — No cloud. No telemetry. Just you and the mountain.",
    version = "0.1.0"
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Inicializa el vault por primera vez
    Init,
    /// Agrega una nueva entrada
    Add { name: String },
    /// Obtiene una entrada
    Get { name: String },
    /// Lista todas las entradas
    List,
    /// Elimina una entrada
    Delete { name: String },
}

fn prompt(label: &str) -> String {
    print!("{}: ", label);
    io::stdout().flush().unwrap();
    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();
    input.trim().to_string()
}

fn unlock(config: &Config) -> [u8; 32] {
    print!("🔑 Master password: ");
    io::stdout().flush().unwrap();
    let password = read_password().unwrap();

    let keyfile = fs::read(&config.key_path)
        .expect("❌ Keyfile no encontrado. ¿Corriste `erebor init`?");

    vault::derive_key(&password, &keyfile)
}

fn main() {
    banner::print_banner();
    let cli = Cli::parse();
    let config = Config::default();

    match cli.command {
        Commands::Init => {
            if config.vault_path.exists() {
                println!("⚠️  El vault ya existe.");
                return;
            }

            println!("⛰️  Inicializando Erebor Vault...");

            fs::create_dir_all(config.vault_path.parent().unwrap())
                .expect("Error creando directorio");

            let mut keyfile_bytes = [0u8; 64];
            rand::RngCore::fill_bytes(&mut rand::thread_rng(), &mut keyfile_bytes);
            fs::write(&config.key_path, keyfile_bytes)
                .expect("Error escribiendo keyfile");

            println!("✅ Keyfile generado en: {:?}", config.key_path);

            print!("🔑 Elige tu master password: ");
            io::stdout().flush().unwrap();
            let password = read_password().unwrap();

            let key = vault::derive_key(&password, &keyfile_bytes);
            let empty_vault = models::Vault::new();
            vault::save_vault(&empty_vault, &config, &key);

            println!("✅ Vault creado. La montaña está sellada.");
            println!("⚠️  Guarda una copia de {:?} en lugar seguro.", config.key_path);
        }

        Commands::Add { name } => {
            let key = unlock(&config);
            let mut v = vault::load_vault(&config, &key);

            let username = prompt("👤 Usuario");
            print!("🔒 Contraseña: ");
            io::stdout().flush().unwrap();
            let password = read_password().unwrap();
            let notes = prompt("📝 Notas (opcional)");

            let entry = Entry {
                username,
                password,
                notes,
                created_at: chrono::Local::now()
                    .format("%Y-%m-%d")
                    .to_string(),
            };

            v.entries.insert(name.clone(), entry);
            vault::save_vault(&v, &config, &key);
            println!("✅ '{}' guardado en la montaña.", name);
        }

        Commands::Get { name } => {
            let key = unlock(&config);
            let v = vault::load_vault(&config, &key);

            match v.entries.get(&name) {
                Some(entry) => {
                    println!("\n⛰️  {}", name);
                    println!("👤 Usuario  : {}", entry.username);
                    println!("🔒 Password : {}", entry.password);
                    if !entry.notes.is_empty() {
                        println!("📝 Notas    : {}", entry.notes);
                    }
                    println!("📅 Creado   : {}", entry.created_at);
                }
                None => println!("❌ '{}' no encontrado.", name),
            }
        }

        Commands::List => {
            let key = unlock(&config);
            let v = vault::load_vault(&config, &key);

            if v.entries.is_empty() {
                println!("📭 El vault está vacío.");
                return;
            }

            println!("\n⛰️  Erebor Vault — {} entrada(s)\n", v.entries.len());
            for (name, entry) in &v.entries {
                println!("  🔑 {} ({})", name, entry.username);
            }
        }

        Commands::Delete { name } => {
            let key = unlock(&config);
            let mut v = vault::load_vault(&config, &key);

            if v.entries.remove(&name).is_some() {
                vault::save_vault(&v, &config, &key);
                println!("🗑️  '{}' eliminado del vault.", name);
            } else {
                println!("❌ '{}' no encontrado.", name);
            }
        }
    }
}
