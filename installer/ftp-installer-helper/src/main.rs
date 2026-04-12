/// ftp-installer-helper — Downloads and installs components from the configured FTP server.
///
/// Actions:
///   install-postgres  Download PostgreSQL 18 tarball and extract to /usr/lib/ftp-simulator/pg18,
///                     then initdb and configure the cluster.
///   install-datasets  Download sample datasets and load them into PostgreSQL.
///   check-update      Fetch latest.json from the FTP server and print the latest version.
use std::path::PathBuf;
use std::process::{Command, ExitCode};

use clap::{Parser, ValueEnum};

#[derive(Parser)]
#[command(name = "ftp-installer-helper", about = "FTP Simulator install helper")]
struct Cli {
    /// Base URL of the FTP server (e.g. ftp://files.example.com/ftp-simulator)
    #[arg(long)]
    ftp_url: String,

    /// Action to perform
    #[arg(long, value_enum)]
    action: Action,

    /// Target installation prefix (default: /usr/lib/ftp-simulator)
    #[arg(long, default_value = "/usr/lib/ftp-simulator")]
    prefix: PathBuf,

    /// PostgreSQL data directory (default: /var/lib/ftp-simulator/pgdata)
    #[arg(long, default_value = "/var/lib/ftp-simulator/pgdata")]
    pgdata: PathBuf,
}

#[derive(Clone, ValueEnum)]
enum Action {
    InstallPostgres,
    InstallDatasets,
    CheckUpdate,
}

fn main() -> ExitCode {
    let cli = Cli::parse();
    match cli.action {
        Action::InstallPostgres => install_postgres(&cli.ftp_url, &cli.prefix, &cli.pgdata),
        Action::InstallDatasets => install_datasets(&cli.ftp_url, &cli.prefix),
        Action::CheckUpdate     => check_update(&cli.ftp_url),
    }
}

fn curl_download(url: &str, dest: &PathBuf) -> bool {
    eprintln!("Downloading {} …", url);
    Command::new("curl")
        .args(["--fail", "--silent", "--show-error", "--location", "-o"])
        .arg(dest)
        .arg(url)
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

fn install_postgres(ftp_url: &str, prefix: &PathBuf, pgdata: &PathBuf) -> ExitCode {
    let tarball_url = format!("{}/postgresql/pg18-linux-amd64.tar.gz", ftp_url.trim_end_matches('/'));
    let dest = PathBuf::from("/tmp/pg18.tar.gz");
    let pg_prefix = prefix.join("pg18");

    if !curl_download(&tarball_url, &dest) {
        eprintln!("Error: failed to download PostgreSQL 18 from {}", tarball_url);
        return ExitCode::FAILURE;
    }

    std::fs::create_dir_all(&pg_prefix).ok();

    let ok = Command::new("tar")
        .args(["-xzf"])
        .arg(&dest)
        .args(["--strip-components=1", "-C"])
        .arg(&pg_prefix)
        .status()
        .map(|s| s.success())
        .unwrap_or(false);

    if !ok {
        eprintln!("Error: failed to extract PostgreSQL tarball");
        return ExitCode::FAILURE;
    }

    // initdb if pgdata does not exist yet
    if !pgdata.join("PG_VERSION").exists() {
        eprintln!("Initialising PostgreSQL cluster at {} …", pgdata.display());
        std::fs::create_dir_all(pgdata).ok();
        let ok = Command::new(pg_prefix.join("bin/initdb"))
            .args(["-D"])
            .arg(pgdata)
            .args(["--encoding=UTF8", "--locale=C", "-U", "ftp_simulator"])
            .status()
            .map(|s| s.success())
            .unwrap_or(false);
        if !ok {
            eprintln!("Error: initdb failed");
            return ExitCode::FAILURE;
        }

        // Append pg_hba and postgresql.conf tweaks
        let hba = pgdata.join("pg_hba.conf");
        if let Ok(mut content) = std::fs::read_to_string(&hba) {
            content.push_str("\n# FTP Simulator\nlocal all ftp_simulator trust\nhost all ftp_simulator 127.0.0.1/32 trust\n");
            std::fs::write(&hba, content).ok();
        }
    }

    eprintln!("PostgreSQL 18 installed at {}", pg_prefix.display());
    ExitCode::SUCCESS
}

fn install_datasets(ftp_url: &str, prefix: &PathBuf) -> ExitCode {
    let datasets_url = format!("{}/datasets/sample-curves.sql", ftp_url.trim_end_matches('/'));
    let dest = prefix.join("sample-curves.sql");

    if !curl_download(&datasets_url, &dest) {
        eprintln!("Error: failed to download sample datasets");
        return ExitCode::FAILURE;
    }

    eprintln!("Sample datasets downloaded to {}", dest.display());
    eprintln!("Load with: psql -U ftp_simulator ftp_simulator < {}", dest.display());
    ExitCode::SUCCESS
}

fn check_update(ftp_url: &str) -> ExitCode {
    let url = format!("{}/releases/latest.json", ftp_url.trim_end_matches('/'));
    let dest = PathBuf::from("/tmp/ftp-simulator-latest.json");
    if !curl_download(&url, &dest) {
        eprintln!("Error: could not fetch latest.json");
        return ExitCode::FAILURE;
    }
    let content = std::fs::read_to_string(&dest).unwrap_or_default();
    println!("{}", content);
    ExitCode::SUCCESS
}
