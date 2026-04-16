use anyhow::Result;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

pub const MAX_IR_SIZE: u64 = 50 * 1024 * 1024;
pub const MAX_LLM_RESPONSE_SIZE: usize = 1024 * 1024;
pub const MAX_DESCRIPTION_LENGTH: usize = 500;

fn blocked_prefixes() -> Vec<PathBuf> {
    let root = Path::new(std::path::MAIN_SEPARATOR_STR);
    ["etc", "dev", "proc", "sys", "boot"]
        .iter()
        .map(|name| root.join(name))
        .collect()
}

fn is_symlink(path: &Path) -> bool {
    path.symlink_metadata()
        .map(|m| m.file_type().is_symlink())
        .unwrap_or(false)
}

pub fn load_file_safe(path: &Path) -> Result<String> {
    if is_symlink(path) {
        anyhow::bail!("Input file must not be a symbolic link");
    }
    let mut file =
        std::fs::File::open(path).map_err(|e| anyhow::anyhow!("Cannot open file: {e}"))?;
    let metadata = file.metadata()?;
    if metadata.len() > MAX_IR_SIZE {
        anyhow::bail!("File exceeds maximum size of {MAX_IR_SIZE} bytes");
    }
    let capacity = usize::try_from(metadata.len()).unwrap_or(0);
    let mut content = String::with_capacity(capacity);
    file.read_to_string(&mut content)?;
    Ok(content)
}

pub fn write_output_safe(path: &Path, content: &str) -> Result<()> {
    let path_str = path.to_string_lossy();
    if path_str.contains("..") {
        anyhow::bail!("Output path must not contain '..' traversal");
    }
    if is_symlink(path) {
        anyhow::bail!("Output path must not be a symbolic link");
    }

    let resolved = if let Some(parent) = path.parent() {
        if parent.as_os_str().is_empty() {
            let cwd = std::env::current_dir()?;
            cwd.join(path)
        } else {
            if is_symlink(parent) {
                anyhow::bail!("Output parent directory must not be a symbolic link");
            }
            let canonical_parent = parent
                .canonicalize()
                .map_err(|e| anyhow::anyhow!("Cannot resolve output directory: {e}"))?;
            canonical_parent.join(path.file_name().unwrap_or_default())
        }
    } else {
        path.to_path_buf()
    };

    let resolved_str = resolved.to_string_lossy();
    for prefix in blocked_prefixes() {
        let prefix_str = prefix.to_string_lossy();
        if resolved_str.starts_with(prefix_str.as_ref()) {
            anyhow::bail!("Writing to system directories is not allowed");
        }
    }

    let mut file = std::fs::OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(&resolved)?;
    file.write_all(content.as_bytes())?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        file.set_permissions(std::fs::Permissions::from_mode(0o600))?;
    }
    Ok(())
}

#[must_use]
pub fn escape_markdown(s: &str) -> String {
    s.replace('|', "\\|")
        .replace('\n', " ")
        .replace('\r', "")
        .replace('`', "'")
}

#[must_use]
pub fn safe_description(desc: &str, max_len: usize) -> String {
    let escaped = escape_markdown(desc);
    if escaped.len() <= max_len {
        return escaped;
    }
    let truncate_at = max_len.saturating_sub(3);
    let boundary = escaped
        .char_indices()
        .take_while(|(i, _)| *i <= truncate_at)
        .last()
        .map_or(0, |(i, _)| i);
    format!("{}...", &escaped[..boundary])
}

/// Sanitize LLM response content: strip HTML tags, escape markdown.
#[must_use]
pub fn sanitize_llm_response(s: &str) -> String {
    use std::sync::LazyLock;
    static RE_HTML: LazyLock<regex::Regex> =
        LazyLock::new(|| regex::Regex::new(r"<[^>]+>").unwrap());
    let no_html = RE_HTML.replace_all(s, "").into_owned();
    escape_markdown(&no_html)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_traversal() {
        let path = PathBuf::from("..").join("..").join("etc").join("x");
        assert!(write_output_safe(&path, "t").is_err());
    }

    #[test]
    fn writes_to_temp() {
        let dir = tempfile::tempdir().unwrap();
        let p = dir.path().join("out.md");
        write_output_safe(&p, "# Hi").unwrap();
        assert_eq!(std::fs::read_to_string(&p).unwrap(), "# Hi");
    }

    #[test]
    fn sanitizes_html_in_llm() {
        assert_eq!(
            sanitize_llm_response("hello <script>alert</script> world"),
            "hello alert world"
        );
        assert_eq!(sanitize_llm_response("<b>bold</b>"), "bold");
    }

    #[test]
    fn escapes_markdown() {
        assert_eq!(super::escape_markdown("a|b\nc"), "a\\|b c");
    }
}
