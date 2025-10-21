use chrono::Utc;
use dialoguer::Input;
use flate2::Compression;
use flate2::read::GzDecoder;
use flate2::write::GzEncoder;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashSet;
use std::fs::{self, File};
use std::io::{self, Cursor, Read, Write};
use std::path::{Component, Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};
use tar::{Archive as TarArchive, Builder as TarBuilder, EntryType, Header as TarHeader};
use tempfile::TempDir;
use zip::CompressionMethod;
use zip::ZipWriter;
use zip::read::ZipArchive;
use zip::write::FileOptions as ZipFileOptions;

use crate::config;
use crate::tools::Tools;

#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;

pub const MANIFEST_VERSION: u32 = 1;
const MANIFEST_FILE_NAME: &str = "manifest.json";

const DEFAULT_CONFIG_NAME: &str = "config.yaml";

#[derive(Debug, thiserror::Error)]
pub enum PackageError {
    #[error("Failed to load tools: {0}")]
    Tools(#[from] crate::tools::ToolError),
    #[error("I/O error: {0}")]
    Io(#[from] io::Error),
    #[error("Serialization error: {0}")]
    SerdeJson(#[from] serde_json::Error),
    #[error("Archive error: {0}")]
    Zip(#[from] zip::result::ZipError),
    #[error("Unsupported archive format")]
    UnsupportedArchive,
    #[error("Missing manifest in archive")]
    MissingManifest,
    #[error("Manifest version {found} is not supported (expected {expected})")]
    ManifestVersionMismatch { found: u32, expected: u32 },
    #[error("Path '{path}' escapes the package root")]
    PathOutsideRoot { path: String },
    #[error("Hash mismatch for {path}")]
    HashMismatch { path: String },
    #[error("Required file '{path}' not found in archive")]
    MissingFile { path: String },
    #[error("Interactive prompt required but disabled (provide --dest)")]
    PromptUnavailable,
    #[error("Archive contains duplicate entry '{path}'")]
    DuplicatePath { path: String },
    #[error("Prompt error: {0}")]
    Prompt(#[from] dialoguer::Error),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Manifest {
    pub version: u32,
    pub generated_at: String,
    pub original_root: String,
    pub config: ManifestFile,
    pub tools: Vec<ManifestToolEntry>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ManifestFile {
    pub path: String,
    pub sha256: String,
    pub mode: u32,
    pub size: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ManifestToolEntry {
    pub id: String,
    pub name: String,
    pub root: String,
    pub file: String,
    pub dependencies: Vec<String>,
    pub artifact: ManifestFile,
}

#[derive(Debug)]
pub struct ExportOptions {
    pub destination: PathBuf,
    pub format: ArchiveFormat,
}

#[derive(Debug)]
pub struct InstallOptions {
    pub archive_path: PathBuf,
    pub destination_root: Option<PathBuf>,
    pub non_interactive: bool,
}

#[derive(Debug)]
pub struct InstallReport {
    pub destination_root: PathBuf,
    pub installed_files: Vec<PathBuf>,
    pub backups: Vec<PathBuf>,
}

#[derive(Clone, Copy, Debug)]
pub enum ArchiveFormat {
    TarGz,
    Zip,
}

impl ArchiveFormat {
    pub fn extension(&self) -> &'static str {
        match self {
            ArchiveFormat::TarGz => "tar.gz",
            ArchiveFormat::Zip => "zip",
        }
    }
}

pub fn export_archive(options: &ExportOptions) -> Result<PathBuf, PackageError> {
    ensure_destination_parent(&options.destination)?;

    let (tools, warnings) = Tools::new_relaxed()?;
    for warning in warnings {
        eprintln!("Warning: {warning}");
    }

    let expanded_root = config::expand_home_path(tools.root().trim());
    if !expanded_root.exists() {
        fs::create_dir_all(&expanded_root)?;
    }

    let config_path = config::expand_home_path(config::DEFAULT_CONFIG_PATH);
    if !config_path.exists() {
        return Err(PackageError::MissingFile {
            path: config_path.to_string_lossy().into_owned(),
        });
    }

    let config_metadata = fs::metadata(&config_path)?;
    let config_relative = relative_path(&config_path, &expanded_root)
        .unwrap_or_else(|_| PathBuf::from(DEFAULT_CONFIG_NAME));
    let config_manifest = ManifestFile {
        path: path_to_string(&config_relative),
        sha256: compute_sha256_path(&config_path)?,
        mode: file_mode(&config_metadata),
        size: config_metadata.len(),
    };

    let mut manifest_tools = Vec::new();
    for tool in tools.iter() {
        let script_path = tools.tool_path(tool);
        if !script_path.exists() {
            eprintln!(
                "Warning: script for tool '{}' not found at {}",
                tool.display_name(),
                script_path.display()
            );
            continue;
        }
        let script_metadata = fs::metadata(&script_path)?;
        let relative = relative_path(&script_path, &expanded_root)
            .unwrap_or_else(|_| PathBuf::from(format!("{}/{}", tool.root, tool.file)));
        manifest_tools.push(ManifestToolEntry {
            id: tool.id.clone(),
            name: tool.name.clone(),
            root: tool.root.clone(),
            file: tool.file.clone(),
            dependencies: tool.dependencies.clone(),
            artifact: ManifestFile {
                path: path_to_string(&relative),
                sha256: if script_metadata.is_dir() {
                    String::new()
                } else {
                    compute_sha256_path(&script_path)?
                },
                mode: file_mode(&script_metadata),
                size: if script_metadata.is_dir() {
                    0
                } else {
                    script_metadata.len()
                },
            },
        });
    }

    let manifest = Manifest {
        version: MANIFEST_VERSION,
        generated_at: Utc::now().to_rfc3339(),
        original_root: expanded_root.to_string_lossy().into_owned(),
        config: config_manifest,
        tools: manifest_tools,
    };

    let package_path = finalize_destination(&options.destination, options.format);

    match options.format {
        ArchiveFormat::TarGz => create_tar_gz(&package_path, &manifest, &expanded_root)?,
        ArchiveFormat::Zip => create_zip(&package_path, &manifest, &expanded_root)?,
    }

    Ok(package_path)
}

pub fn install_archive(options: &InstallOptions) -> Result<InstallReport, PackageError> {
    let format = detect_archive_format(&options.archive_path)?;
    let temp_dir = extract_archive(&options.archive_path, format)?;
    let manifest_path = temp_dir.path().join(MANIFEST_FILE_NAME);
    if !manifest_path.exists() {
        return Err(PackageError::MissingManifest);
    }

    let manifest_file = File::open(&manifest_path)?;
    let manifest: Manifest = serde_json::from_reader(manifest_file)?;

    if manifest.version != MANIFEST_VERSION {
        return Err(PackageError::ManifestVersionMismatch {
            found: manifest.version,
            expected: MANIFEST_VERSION,
        });
    }

    validate_manifest_paths(&manifest)?;

    verify_manifest_entry(temp_dir.path(), &manifest.config)?;
    for entry in &manifest.tools {
        verify_manifest_entry(temp_dir.path(), &entry.artifact)?;
    }

    let destination_root = resolve_destination_root(&manifest, options)?;
    fs::create_dir_all(&destination_root)?;

    let mut report = InstallReport {
        destination_root: destination_root.clone(),
        installed_files: Vec::new(),
        backups: Vec::new(),
    };

    let config_source = temp_dir.path().join(&manifest.config.path);
    let config_target = destination_root.join(&manifest.config.path);
    install_file(
        &config_source,
        &config_target,
        manifest.config.mode,
        &mut report,
    )?;

    for entry in &manifest.tools {
        let script_source = temp_dir.path().join(&entry.artifact.path);
        let script_target = destination_root.join(&entry.artifact.path);
        install_file(
            &script_source,
            &script_target,
            entry.artifact.mode,
            &mut report,
        )?;
    }

    Ok(report)
}

fn ensure_destination_parent(path: &Path) -> Result<(), PackageError> {
    if let Some(parent) = path.parent() {
        if !parent.as_os_str().is_empty() {
            fs::create_dir_all(parent)?;
        }
    }
    Ok(())
}

fn finalize_destination(candidate: &Path, format: ArchiveFormat) -> PathBuf {
    if let Some(ext) = candidate.extension().and_then(|ext| ext.to_str()) {
        if ext.eq_ignore_ascii_case(format.extension()) {
            return candidate.to_path_buf();
        }
    }

    let mut final_path = candidate.to_path_buf();
    if final_path.as_os_str().is_empty() {
        final_path = default_archive_name(format.extension());
    } else {
        let mut os_string = final_path.into_os_string();
        os_string.push(format!(".{ext}", ext = format.extension()));
        final_path = PathBuf::from(os_string);
    }
    final_path
}

fn default_archive_name(extension: &str) -> PathBuf {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    PathBuf::from(format!("dotfiles-export-{timestamp}.{extension}"))
}

fn create_tar_gz(destination: &Path, manifest: &Manifest, root: &Path) -> Result<(), PackageError> {
    let file = File::create(destination)?;
    let encoder = GzEncoder::new(file, Compression::default());
    let mut builder = TarBuilder::new(encoder);

    append_bytes_to_tar(
        &mut builder,
        MANIFEST_FILE_NAME,
        &serde_json::to_vec_pretty(manifest)?,
    )?;

    append_file_to_tar(&mut builder, root, &manifest.config)?;
    for entry in &manifest.tools {
        append_file_to_tar(&mut builder, root, &entry.artifact)?;
    }

    builder.finish()?;
    Ok(())
}

fn create_zip(destination: &Path, manifest: &Manifest, root: &Path) -> Result<(), PackageError> {
    let file = File::create(destination)?;
    let mut writer = ZipWriter::new(file);

    let manifest_bytes = serde_json::to_vec_pretty(manifest)?;
    writer.start_file(
        MANIFEST_FILE_NAME,
        ZipFileOptions::default()
            .compression_method(CompressionMethod::Deflated)
            .unix_permissions(0o644),
    )?;
    writer.write_all(&manifest_bytes)?;

    add_file_to_zip(&mut writer, root, &manifest.config)?;
    for entry in &manifest.tools {
        add_file_to_zip(&mut writer, root, &entry.artifact)?;
    }

    writer.finish()?;
    Ok(())
}

fn append_bytes_to_tar<W: Write>(
    builder: &mut TarBuilder<W>,
    path: &str,
    bytes: &[u8],
) -> Result<(), PackageError> {
    let mut header = TarHeader::new_gnu();
    header.set_size(bytes.len() as u64);
    header.set_mode(0o644);
    header.set_entry_type(EntryType::Regular);
    header.set_mtime(
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs(),
    );
    header.set_cksum();
    builder.append_data(&mut header, path, &mut Cursor::new(bytes))?;
    Ok(())
}

fn append_file_to_tar<W: Write>(
    builder: &mut TarBuilder<W>,
    root: &Path,
    entry: &ManifestFile,
) -> Result<(), PackageError> {
    let full_path = root.join(&entry.path);
    if full_path.is_dir() {
        let mut header = TarHeader::new_gnu();
        header.set_size(0);
        header.set_mode(entry.mode);
        header.set_entry_type(EntryType::Directory);
        header.set_mtime(
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
        );
        header.set_cksum();
        builder.append_data(&mut header, Path::new(&entry.path), &mut io::empty())?;
        return Ok(());
    }

    let mut file = File::open(&full_path)?;
    let mut header = TarHeader::new_gnu();
    header.set_size(entry.size);
    header.set_mode(entry.mode);
    header.set_entry_type(EntryType::Regular);
    header.set_mtime(
        fs::metadata(&full_path)?
            .modified()
            .ok()
            .and_then(|mtime| mtime.duration_since(UNIX_EPOCH).ok())
            .map(|duration| duration.as_secs())
            .unwrap_or_else(|| {
                SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs()
            }),
    );
    header.set_cksum();
    builder.append_data(&mut header, Path::new(&entry.path), &mut file)?;
    Ok(())
}

fn add_file_to_zip(
    writer: &mut ZipWriter<File>,
    root: &Path,
    entry: &ManifestFile,
) -> Result<(), PackageError> {
    let full_path = root.join(&entry.path);
    if full_path.is_dir() {
        let dir_path = format!("{}/", path_to_string(Path::new(&entry.path)));
        writer.add_directory(
            dir_path,
            ZipFileOptions::default().unix_permissions(entry.mode),
        )?;
        return Ok(());
    }

    let mut file = File::open(&full_path)?;
    writer.start_file(
        path_to_string(Path::new(&entry.path)),
        ZipFileOptions::default()
            .compression_method(CompressionMethod::Deflated)
            .unix_permissions(entry.mode),
    )?;
    io::copy(&mut file, writer)?;
    Ok(())
}

fn detect_archive_format(path: &Path) -> Result<ArchiveFormat, PackageError> {
    if let Some(name) = path.file_name().and_then(|name| name.to_str()) {
        if name.ends_with(".tar.gz") || name.ends_with(".tgz") {
            return Ok(ArchiveFormat::TarGz);
        }
    }

    let lowercase = path
        .extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| ext.to_ascii_lowercase())
        .unwrap_or_default();

    match lowercase.as_str() {
        "zip" => Ok(ArchiveFormat::Zip),
        "gz" => Ok(ArchiveFormat::TarGz),
        _ => Err(PackageError::UnsupportedArchive),
    }
}

fn extract_archive(path: &Path, format: ArchiveFormat) -> Result<TempDir, PackageError> {
    let temp_dir = TempDir::new()?;

    match format {
        ArchiveFormat::TarGz => {
            let file = File::open(path)?;
            let decoder = GzDecoder::new(file);
            let mut archive = TarArchive::new(decoder);
            archive.unpack(temp_dir.path())?;
        }
        ArchiveFormat::Zip => {
            let file = File::open(path)?;
            let mut archive = ZipArchive::new(file)?;
            archive.extract(temp_dir.path())?;
        }
    }

    Ok(temp_dir)
}

fn validate_manifest_paths(manifest: &Manifest) -> Result<(), PackageError> {
    let mut seen = HashSet::new();
    check_path(&mut seen, &manifest.config.path)?;
    for entry in &manifest.tools {
        check_path(&mut seen, &entry.artifact.path)?;
    }
    Ok(())
}

fn check_path(seen: &mut HashSet<String>, path: &str) -> Result<(), PackageError> {
    validate_relative_path(path)?;
    if !seen.insert(path.to_string()) {
        return Err(PackageError::DuplicatePath {
            path: path.to_string(),
        });
    }
    Ok(())
}

fn verify_manifest_entry(root: &Path, entry: &ManifestFile) -> Result<(), PackageError> {
    let source = root.join(&entry.path);
    if !source.exists() {
        return Err(PackageError::MissingFile {
            path: source.to_string_lossy().into_owned(),
        });
    }

    if source.is_dir() {
        return Ok(());
    }

    let actual_hash = compute_sha256_path(&source)?;
    if actual_hash != entry.sha256 {
        return Err(PackageError::HashMismatch {
            path: entry.path.clone(),
        });
    }

    Ok(())
}

fn resolve_destination_root(
    manifest: &Manifest,
    options: &InstallOptions,
) -> Result<PathBuf, PackageError> {
    if let Some(root) = &options.destination_root {
        return Ok(root.clone());
    }

    if options.non_interactive {
        return Ok(PathBuf::from(&manifest.original_root));
    }

    let default_value = manifest.original_root.clone();
    let input: String = Input::new()
        .with_prompt("Destination root")
        .default(default_value.clone())
        .interact_text()?;

    let trimmed = input.trim();
    if trimmed.is_empty() {
        return Err(PackageError::PromptUnavailable);
    }

    Ok(config::expand_home_path(trimmed))
}

fn install_file(
    source: &Path,
    target: &Path,
    mode: u32,
    report: &mut InstallReport,
) -> Result<(), PackageError> {
    if source.is_dir() {
        fs::create_dir_all(target)?;
        report.installed_files.push(target.to_path_buf());
        return Ok(());
    }

    if let Some(parent) = target.parent() {
        fs::create_dir_all(parent)?;
    }

    if target.exists() {
        let backup = backup_existing(target)?;
        report.backups.push(backup.clone());
    }

    fs::copy(source, target)?;

    #[cfg(unix)]
    {
        let permissions = fs::Permissions::from_mode(mode);
        fs::set_permissions(target, permissions)?;
    }

    report.installed_files.push(target.to_path_buf());
    Ok(())
}

fn backup_existing(path: &Path) -> Result<PathBuf, PackageError> {
    let timestamp = Utc::now().format("%Y%m%d%H%M%S");
    let file_name = path
        .file_name()
        .map(|name| format!("{}.bak.{timestamp}", name.to_string_lossy()))
        .unwrap_or_else(|| format!("backup-{timestamp}"));

    let backup_path = path
        .parent()
        .map(|parent| parent.join(file_name.clone()))
        .unwrap_or_else(|| PathBuf::from(file_name.clone()));

    if path.is_dir() {
        fs::create_dir_all(&backup_path)?;
    } else {
        fs::rename(path, &backup_path)?;
    }

    Ok(backup_path)
}

fn compute_sha256_path(path: &Path) -> Result<String, PackageError> {
    if path.is_dir() {
        return Ok(String::new());
    }
    let file = File::open(path)?;
    compute_sha256_reader(file)
}

fn compute_sha256_reader<R: Read>(mut reader: R) -> Result<String, PackageError> {
    let mut hasher = Sha256::new();
    let mut buffer = [0u8; 8192];
    loop {
        let bytes_read = reader.read(&mut buffer)?;
        if bytes_read == 0 {
            break;
        }
        hasher.update(&buffer[..bytes_read]);
    }
    Ok(hex::encode(hasher.finalize()))
}

fn relative_path(path: &Path, root: &Path) -> Result<PathBuf, PackageError> {
    let relative = path
        .strip_prefix(root)
        .map_err(|_| PackageError::PathOutsideRoot {
            path: path.to_string_lossy().into_owned(),
        })?;
    validate_relative_path(relative.to_str().unwrap_or_default())?;
    Ok(relative.to_path_buf())
}

fn validate_relative_path(path: &str) -> Result<(), PackageError> {
    let candidate = Path::new(path);
    if candidate.is_absolute() {
        return Err(PackageError::PathOutsideRoot {
            path: path.to_string(),
        });
    }

    if candidate
        .components()
        .any(|component| matches!(component, Component::ParentDir))
    {
        return Err(PackageError::PathOutsideRoot {
            path: path.to_string(),
        });
    }

    Ok(())
}

fn path_to_string(path: &Path) -> String {
    path.components()
        .fold(PathBuf::new(), |mut acc, component| {
            acc.push(component);
            acc
        })
        .to_string_lossy()
        .replace('\\', "/")
}

fn file_mode(metadata: &fs::Metadata) -> u32 {
    #[cfg(unix)]
    {
        metadata.permissions().mode()
    }

    #[cfg(not(unix))]
    {
        0o644
    }
}
