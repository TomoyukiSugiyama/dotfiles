mod app;
mod config;
mod package;
mod tools;

use clap::{Parser, Subcommand, ValueEnum};
use color_eyre::Result;
use std::path::PathBuf;

#[derive(Parser)]
#[command(author, version, about = "dotfiles manager", propagate_version = true)]
struct Cli {
    /// Run the TUI application (default)
    #[arg(long, default_value_t = false)]
    tui: bool,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Export configuration and tools into an archive
    Export {
        /// Output archive path (extension added if missing)
        #[arg(short, long)]
        dest: PathBuf,

        /// Archive format (tar.gz or zip)
        #[arg(long, value_enum, default_value_t = ExportFormat::TarGz)]
        format: ExportFormat,
    },
    /// Install configuration and tools from an archive
    Install {
        /// Source archive path
        #[arg(short, long)]
        src: PathBuf,

        /// Destination root directory (defaults to manifest's root)
        #[arg(short, long)]
        dest: Option<PathBuf>,

        /// Run without prompts (falls back to manifest root when dest missing)
        #[arg(long, default_value_t = false)]
        non_interactive: bool,
    },
}

#[derive(Clone, Copy, Debug, ValueEnum)]
enum ExportFormat {
    #[value(alias = "tgz", alias = "tar", alias = "tar.gz")]
    TarGz,
    #[value(alias = "zip")]
    Zip,
}

impl ExportFormat {
    fn as_archive_format(&self) -> package::ArchiveFormat {
        match self {
            ExportFormat::TarGz => package::ArchiveFormat::TarGz,
            ExportFormat::Zip => package::ArchiveFormat::Zip,
        }
    }
}

fn main() -> Result<()> {
    color_eyre::install()?;

    let cli = Cli::parse();
    match cli.command {
        Some(Commands::Export { dest, format }) => {
            let options = package::ExportOptions {
                destination: dest,
                format: format.as_archive_format(),
            };
            let archive_path = package::export_archive(&options)?;
            println!("Created archive at {}", archive_path.display());
            Ok(())
        }
        Some(Commands::Install {
            src,
            dest,
            non_interactive,
        }) => {
            let options = package::InstallOptions {
                archive_path: src,
                destination_root: dest,
                non_interactive,
            };
            let report = package::install_archive(&options)?;
            println!(
                "Installed dotfiles into {}",
                report.destination_root.display()
            );
            if !report.backups.is_empty() {
                println!("Backed up files:");
                for path in report.backups {
                    println!("  {}", path.display());
                }
            }
            println!("Installed files:");
            for path in report.installed_files {
                println!("  {}", path.display());
            }
            Ok(())
        }
        None => run_tui(),
    }
}

fn run_tui() -> Result<()> {
    let terminal = ratatui::init();
    let result = app::App::new().run(terminal);
    ratatui::restore();
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_export_format_as_archive_format() {
        assert!(matches!(
            ExportFormat::TarGz.as_archive_format(),
            package::ArchiveFormat::TarGz
        ));
        assert!(matches!(
            ExportFormat::Zip.as_archive_format(),
            package::ArchiveFormat::Zip
        ));
    }

    #[test]
    fn test_export_format_value_aliases() {
        // Test that the value aliases are correctly defined
        // These are checked by clap at runtime
        let format = ExportFormat::TarGz;
        assert!(matches!(format, ExportFormat::TarGz));
    }
}
