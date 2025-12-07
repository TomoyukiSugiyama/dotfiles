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
#[cfg(unix)]
use std::os::unix::fs::symlink;
use std::path::{Component, Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};
use tar::{Archive as TarArchive, Builder as TarBuilder, EntryType, Header as TarHeader};
use tempfile::TempDir;
use walkdir::WalkDir;
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
    #[error("YAML error: {0}")]
    SerdeYaml(#[from] serde_yaml::Error),
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
    #[serde(default)]
    pub related_files: Vec<ManifestFile>,
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
            related_files: collect_related_files(&expanded_root, &relative)?,
        });
    }

    let manifest = Manifest {
        version: MANIFEST_VERSION,
        generated_at: Utc::now().to_rfc3339(),
        original_root: tools.root().to_string(),
        config: config_manifest,
        tools: manifest_tools,
    };

    let package_path = finalize_destination(&options.destination, options.format);
    ensure_destination_parent(&package_path)?;

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
        for related in &entry.related_files {
            verify_manifest_entry(temp_dir.path(), related)?;
        }
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

    rewrite_config_root(&config_target, &destination_root)?;

    for entry in &manifest.tools {
        let script_source = temp_dir.path().join(&entry.artifact.path);
        let script_target = destination_root.join(&entry.artifact.path);
        install_file(
            &script_source,
            &script_target,
            entry.artifact.mode,
            &mut report,
        )?;

        for related in &entry.related_files {
            let related_source = temp_dir.path().join(&related.path);
            let related_target = destination_root.join(&related.path);
            install_file(&related_source, &related_target, related.mode, &mut report)?;
        }
    }

    create_root_symlink(&manifest.original_root, &destination_root)?;

    Ok(report)
}

fn ensure_destination_parent(path: &Path) -> Result<(), PackageError> {
    if let Some(parent) = path.parent()
        && !parent.as_os_str().is_empty()
    {
        fs::create_dir_all(parent)?;
    }
    Ok(())
}

fn finalize_destination(candidate: &Path, format: ArchiveFormat) -> PathBuf {
    if candidate.as_os_str().is_empty() {
        return default_archive_name(format.extension());
    }

    let mut final_path = candidate.to_path_buf();

    // Treat as directory if it already exists as a directory OR has no extension
    let is_directory = final_path.is_dir()
        || final_path.extension().is_none() && !final_path.to_string_lossy().ends_with('.');

    if is_directory {
        let file_name = default_archive_name(format.extension());
        final_path.push(file_name.file_name().unwrap());
        return final_path;
    }

    match final_path.extension().and_then(|ext| ext.to_str()) {
        Some(ext) if ext.eq_ignore_ascii_case(format.extension()) => final_path,
        Some(_) => {
            final_path.set_extension(format.extension());
            final_path
        }
        None => {
            // This branch is now unreachable since no-extension paths are treated as directories
            let mut os_string = final_path.into_os_string();
            os_string.push(format!(".{ext}", ext = format.extension()));
            PathBuf::from(os_string)
        }
    }
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
        for related in &entry.related_files {
            append_file_to_tar(&mut builder, root, related)?;
        }
    }

    builder.finish()?;
    Ok(())
}

fn create_zip(destination: &Path, manifest: &Manifest, root: &Path) -> Result<(), PackageError> {
    let file = File::create(destination)?;
    let mut writer = ZipWriter::new(file);

    let manifest_bytes = serde_json::to_vec_pretty(manifest)?;
    writer.start_file::<_, ()>(
        MANIFEST_FILE_NAME,
        ZipFileOptions::default()
            .compression_method(CompressionMethod::Deflated)
            .unix_permissions(0o644),
    )?;
    writer.write_all(&manifest_bytes)?;

    add_file_to_zip(&mut writer, root, &manifest.config)?;
    for entry in &manifest.tools {
        add_file_to_zip(&mut writer, root, &entry.artifact)?;
        for related in &entry.related_files {
            add_file_to_zip(&mut writer, root, related)?;
        }
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

    if entry.path.ends_with('/') {
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

    if full_path.is_dir() {
        let dir_path = if entry.path.ends_with('/') {
            entry.path.clone()
        } else {
            format!("{}/", entry.path)
        };
        let mut header = TarHeader::new_gnu();
        header.set_size(0);
        header.set_mode(entry.mode);
        header.set_entry_type(EntryType::Directory);
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
        builder.append_data(&mut header, Path::new(&dir_path), &mut io::empty())?;
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
    let trimmed = entry.path.trim_end_matches('/');
    let full_path = root.join(trimmed);

    if entry.path.ends_with('/') {
        let dir_path = format!("{}/", path_to_string(Path::new(trimmed)));
        writer.add_directory::<_, ()>(
            dir_path,
            ZipFileOptions::default().unix_permissions(entry.mode),
        )?;
        return Ok(());
    }

    let mut file = File::open(&full_path)?;
    writer.start_file::<_, ()>(
        path_to_string(Path::new(trimmed)),
        ZipFileOptions::default()
            .compression_method(CompressionMethod::Deflated)
            .unix_permissions(entry.mode),
    )?;
    io::copy(&mut file, writer)?;
    Ok(())
}

fn detect_archive_format(path: &Path) -> Result<ArchiveFormat, PackageError> {
    if let Some(name) = path.file_name().and_then(|name| name.to_str())
        && (name.ends_with(".tar.gz") || name.ends_with(".tgz"))
    {
        return Ok(ArchiveFormat::TarGz);
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
        for related in &entry.related_files {
            check_path(&mut seen, &related.path)?;
        }
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
    if source.is_dir() || target.as_os_str().to_string_lossy().ends_with('/') {
        fs::create_dir_all(target)?;
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
        .any(|component| matches!(component, Component::ParentDir | Component::RootDir))
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

fn collect_related_files(
    root: &Path,
    script_relative: &Path,
) -> Result<Vec<ManifestFile>, PackageError> {
    let script_dir = script_relative.parent().map(Path::to_path_buf);
    let mut files = Vec::new();
    let mut seen = HashSet::new();

    if let Some(dir) = script_dir {
        let absolute_dir = root.join(&dir);
        if absolute_dir.is_dir() {
            for entry in WalkDir::new(&absolute_dir)
                .into_iter()
                .filter_map(Result::ok)
            {
                let path = entry.path();
                if path == root.join(script_relative) {
                    continue;
                }

                let relative = relative_path(path, root)?;
                let relative_string = path_to_string(&relative);
                if !seen.insert(relative_string.clone()) {
                    continue;
                }

                let metadata = fs::metadata(path)?;

                if path.is_dir() {
                    files.push(ManifestFile {
                        path: format!("{relative_string}/"),
                        sha256: String::new(),
                        mode: file_mode(&metadata),
                        size: 0,
                    });
                } else if path.is_file() {
                    files.push(ManifestFile {
                        path: relative_string,
                        sha256: compute_sha256_path(path)?,
                        mode: file_mode(&metadata),
                        size: metadata.len(),
                    });
                }
            }
        }
    }

    files.sort_by(|a, b| a.path.cmp(&b.path));
    Ok(files)
}

fn rewrite_config_root(config_path: &Path, new_root: &Path) -> Result<(), PackageError> {
    use std::fmt::Write as FmtWrite;

    let contents = fs::read_to_string(config_path)?;
    let mut output = String::with_capacity(contents.len() + 64);
    let mut in_system_preferences = false;
    let new_root_str = new_root.to_string_lossy();

    for line in contents.lines() {
        let trimmed = line.trim_start();

        if trimmed.starts_with("SystemPreferences:") {
            in_system_preferences = true;
            writeln!(&mut output, "{}", line).unwrap();
            continue;
        }

        if in_system_preferences {
            if trimmed.starts_with("Root:") {
                let indentation = &line[..line.len() - trimmed.len()];
                writeln!(&mut output, "{indentation}Root: {new_root_str}").unwrap();
                in_system_preferences = false;
                continue;
            }

            if trimmed.is_empty() || trimmed.starts_with('#') {
                writeln!(&mut output, "{}", line).unwrap();
                continue;
            }

            in_system_preferences = false;
        }

        writeln!(&mut output, "{}", line).unwrap();
    }

    if !contents.ends_with('\n') {
        output.push('\n');
    }

    fs::write(config_path, output)?;
    Ok(())
}

fn create_root_symlink(original_root: &str, destination_root: &Path) -> Result<(), PackageError> {
    #[cfg(unix)]
    {
        let original_path = config::expand_home_path(original_root);

        if original_path == destination_root {
            return Ok(());
        }

        if let Some(parent) = original_path.parent() {
            fs::create_dir_all(parent)?;
        }

        if let Ok(metadata) = fs::symlink_metadata(&original_path) {
            if metadata.file_type().is_symlink() {
                if let Ok(existing_target) = fs::read_link(&original_path)
                    && existing_target == destination_root
                {
                    return Ok(());
                }
                fs::remove_file(&original_path)?;
            } else {
                let timestamp = Utc::now().format("%Y%m%d%H%M%S");
                let file_name = original_path
                    .file_name()
                    .and_then(|name| name.to_str())
                    .map(|name| name.to_string())
                    .unwrap_or_else(|| "dotfiles".to_string());
                let backup_name = format!("{file_name}.backup-{timestamp}");
                let backup_path = original_path
                    .parent()
                    .unwrap_or_else(|| Path::new(""))
                    .join(backup_name);
                fs::rename(&original_path, backup_path)?;
            }
        }

        symlink(destination_root, &original_path)?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_archive_format_extension() {
        assert_eq!(ArchiveFormat::TarGz.extension(), "tar.gz");
        assert_eq!(ArchiveFormat::Zip.extension(), "zip");
    }

    #[test]
    fn test_detect_archive_format() {
        assert!(matches!(
            detect_archive_format(Path::new("test.tar.gz")),
            Ok(ArchiveFormat::TarGz)
        ));
        assert!(matches!(
            detect_archive_format(Path::new("test.zip")),
            Ok(ArchiveFormat::Zip)
        ));
        assert!(detect_archive_format(Path::new("test.txt")).is_err());
    }

    #[test]
    fn test_validate_relative_path() {
        // Safe paths
        assert!(validate_relative_path("test/file.txt").is_ok());
        assert!(validate_relative_path("file.txt").is_ok());

        // Unsafe paths with parent directory
        assert!(validate_relative_path("../file.txt").is_err());
        assert!(validate_relative_path("test/../file.txt").is_err());

        // Absolute paths
        assert!(validate_relative_path("/etc/passwd").is_err());
    }

    #[test]
    fn test_compute_sha256_reader() {
        let data = b"Hello, World!";
        let reader = Cursor::new(data);
        let hash = compute_sha256_reader(reader).unwrap();

        // Expected SHA256 hash of "Hello, World!"
        let expected = "dffd6021bb2bd5b0af676290809ec3a53191dd81c7f70a4b28688a362182986f";
        assert_eq!(hash, expected);
    }

    #[test]
    fn test_finalize_destination() {
        // Path without extension is treated as a directory
        let path = Path::new("backup");
        let result = finalize_destination(path, ArchiveFormat::TarGz);
        assert!(result.to_string_lossy().starts_with("backup/"));
        assert!(result.to_string_lossy().ends_with(".tar.gz"));

        let result = finalize_destination(path, ArchiveFormat::Zip);
        assert!(result.to_string_lossy().starts_with("backup/"));
        assert!(result.to_string_lossy().ends_with(".zip"));

        // Path with matching extension
        let path = Path::new("myfile.zip");
        let result = finalize_destination(path, ArchiveFormat::Zip);
        assert_eq!(result.to_string_lossy(), "myfile.zip");

        // Path with different extension gets replaced
        let path = Path::new("myfile.txt");
        let result = finalize_destination(path, ArchiveFormat::Zip);
        assert_eq!(result.to_string_lossy(), "myfile.zip");
    }

    #[test]
    fn test_default_archive_name() {
        let result = default_archive_name("tar.gz");
        assert!(result.to_string_lossy().contains("dotfiles"));
        assert!(result.to_string_lossy().ends_with(".tar.gz"));
    }

    #[test]
    fn test_path_to_string() {
        let path = Path::new("test/file.txt");
        let result = path_to_string(path);
        assert_eq!(result, "test/file.txt");
    }

    #[test]
    fn test_check_path() {
        let mut seen = HashSet::new();

        // First insertion should succeed
        assert!(check_path(&mut seen, "file1.txt").is_ok());

        // Duplicate should fail
        assert!(check_path(&mut seen, "file1.txt").is_err());

        // Different path should succeed
        assert!(check_path(&mut seen, "file2.txt").is_ok());
    }

    #[test]
    fn test_rewrite_config_root_basic() {
        use std::path::PathBuf;
        use tempfile::NamedTempFile;

        let mut temp_file = NamedTempFile::new().unwrap();
        let content = r#"# Config file
SystemPreferences:
  Root: /old/path
Preferences:
  ToolsSettings:
    - Name: Test
"#;
        std::io::Write::write_all(&mut temp_file, content.as_bytes()).unwrap();
        let path = temp_file.path();

        let new_root = PathBuf::from("/new/path");
        rewrite_config_root(path, &new_root).unwrap();

        let result = fs::read_to_string(path).unwrap();
        assert!(result.contains("Root: /new/path"));
        assert!(!result.contains("Root: /old/path"));
    }

    #[test]
    fn test_rewrite_config_root_with_comments() {
        use std::path::PathBuf;
        use tempfile::NamedTempFile;

        let mut temp_file = NamedTempFile::new().unwrap();
        let content = r#"SystemPreferences:
  # This is a comment
  Root: /old/path
  # Another comment
Preferences:
  ToolsSettings: []
"#;
        std::io::Write::write_all(&mut temp_file, content.as_bytes()).unwrap();
        let path = temp_file.path();

        let new_root = PathBuf::from("/new/path");
        rewrite_config_root(path, &new_root).unwrap();

        let result = fs::read_to_string(path).unwrap();
        assert!(result.contains("Root: /new/path"));
        assert!(result.contains("# This is a comment"));
        assert!(result.contains("# Another comment"));
    }

    #[test]
    fn test_file_mode_unix() {
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            use tempfile::NamedTempFile;

            let temp_file = NamedTempFile::new().unwrap();
            let path = temp_file.path();

            // Set specific permissions
            let perms = fs::Permissions::from_mode(0o755);
            fs::set_permissions(path, perms).unwrap();

            let metadata = fs::metadata(path).unwrap();
            let mode = file_mode(&metadata);

            assert_eq!(mode & 0o777, 0o755);
        }

        #[cfg(not(unix))]
        {
            let metadata = fs::metadata(".").unwrap();
            let mode = file_mode(&metadata);
            assert_eq!(mode, 0o644);
        }
    }

    #[test]
    fn test_ensure_destination_parent() {
        use tempfile::TempDir;

        let temp_dir = TempDir::new().unwrap();
        let nested_path = temp_dir.path().join("nested").join("dir").join("file.txt");

        ensure_destination_parent(&nested_path).unwrap();

        // Parent directories should be created
        assert!(nested_path.parent().unwrap().exists());
    }

    #[test]
    fn test_relative_path_success() {
        let root = Path::new("/root/path");
        let path = Path::new("/root/path/sub/file.txt");

        let result = relative_path(path, root).unwrap();
        assert_eq!(result, PathBuf::from("sub/file.txt"));
    }

    #[test]
    fn test_relative_path_outside_root() {
        let root = Path::new("/root/path");
        let path = Path::new("/other/path/file.txt");

        let result = relative_path(path, root);
        assert!(result.is_err());
        assert!(matches!(result, Err(PackageError::PathOutsideRoot { .. })));
    }

    #[test]
    fn test_manifest_file_structure() {
        let manifest_file = ManifestFile {
            path: "test/file.txt".to_string(),
            sha256: "abc123".to_string(),
            mode: 0o644,
            size: 1024,
        };

        assert_eq!(manifest_file.path, "test/file.txt");
        assert_eq!(manifest_file.sha256, "abc123");
        assert_eq!(manifest_file.mode, 0o644);
        assert_eq!(manifest_file.size, 1024);
    }

    #[test]
    fn test_manifest_tool_entry_structure() {
        let entry = ManifestToolEntry {
            id: "tool1".to_string(),
            name: "Tool 1".to_string(),
            root: "tool1".to_string(),
            file: "tool1.sh".to_string(),
            dependencies: vec!["tool2".to_string()],
            artifact: ManifestFile {
                path: "tool1/tool1.sh".to_string(),
                sha256: "def456".to_string(),
                mode: 0o755,
                size: 2048,
            },
            related_files: vec![],
        };

        assert_eq!(entry.id, "tool1");
        assert_eq!(entry.dependencies.len(), 1);
        assert_eq!(entry.artifact.mode, 0o755);
    }

    #[test]
    fn test_validate_manifest_paths_duplicate() {
        let manifest = Manifest {
            version: MANIFEST_VERSION,
            generated_at: "2024-01-01T00:00:00Z".to_string(),
            original_root: "/test".to_string(),
            config: ManifestFile {
                path: "config.yaml".to_string(),
                sha256: "hash1".to_string(),
                mode: 0o644,
                size: 100,
            },
            tools: vec![ManifestToolEntry {
                id: "tool1".to_string(),
                name: "Tool 1".to_string(),
                root: "tool1".to_string(),
                file: "tool1.sh".to_string(),
                dependencies: vec![],
                artifact: ManifestFile {
                    path: "config.yaml".to_string(), // Duplicate!
                    sha256: "hash2".to_string(),
                    mode: 0o755,
                    size: 200,
                },
                related_files: vec![],
            }],
        };

        let result = validate_manifest_paths(&manifest);
        assert!(result.is_err());
        assert!(matches!(result, Err(PackageError::DuplicatePath { .. })));
    }

    #[test]
    fn test_path_to_string_with_backslashes() {
        // Test that backslashes are converted to forward slashes
        let path = Path::new("test").join("sub").join("file.txt");
        let result = path_to_string(&path);
        assert!(!result.contains('\\'));
        assert!(result.contains('/') || !result.contains(std::path::MAIN_SEPARATOR));
    }
}
