use super::super::ignore::is_ignored;
use anyhow::Result;
use clap::Parser;
use std::env;
use std::io;
use std::path;

/// Debug rgitignore / exclude files
#[derive(Parser, Debug)]
pub struct CheckIgnoreArgs {
    /// Instead of printing the paths are excluded, for each path that matches an exclude pattern, print the exclude pattern together with the path.
    #[clap(short, long)]
    pub verbose: bool,

    /// Path to the directory to check
    pub pathname: String,
}

fn check_ignore(
    dir: &path::Path,
    pathname: &path::Path,
    verbose: bool,
    writer: &mut dyn io::Write,
) -> Result<u8> {
    let relative_path = pathname.strip_prefix(dir)?;

    let result = is_ignored(pathname)?;
    if result.is_ignored {
        if !verbose {
            writeln!(writer, "{}", relative_path.display())?;
        } else {
            let rule = result.matched_rule.unwrap();
            let relative_rgitignore_path = rule.rgitignore_path.strip_prefix(dir)?;
            writeln!(
                writer,
                "{}:{}:{}\t{}",
                relative_rgitignore_path.display(),
                rule.line_number,
                rule.rule,
                relative_path.display(),
            )?;
        }
    }

    Ok(!result.is_ignored as u8)
}

pub fn rgit_check_ignore(args: &CheckIgnoreArgs) -> Result<u8> {
    let root = env::current_dir()?;
    let pathname = path::Path::new(&args.pathname).canonicalize()?;
    let mut writer = io::stdout();
    check_ignore(
        root.as_path(),
        pathname.as_path(),
        args.verbose,
        &mut writer,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::init_rgit_dir;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn test_check_ignore() {
        let dir = tempdir().unwrap();
        init_rgit_dir(dir.path()).unwrap();
        let rgitignore_path = dir.path().join(".rgitignore");

        fs::write(&rgitignore_path, "\n\n*.txt\n").unwrap();
        fs::write(dir.path().join("file.txt"), "file content").unwrap();
        fs::write(dir.path().join("file.md"), "file content").unwrap();

        let mut buffer = Vec::new();
        let result = check_ignore(
            dir.path(),
            dir.path().join("file.txt").as_path(),
            false,
            &mut buffer,
        )
        .unwrap();
        assert_eq!(result, 0);
        assert_eq!(buffer, b"file.txt\n");
        buffer.clear();

        let result = check_ignore(
            dir.path(),
            dir.path().join("file.txt").as_path(),
            true,
            &mut buffer,
        )
        .unwrap();
        assert_eq!(result, 0);
        assert_eq!(buffer, b".rgitignore:3:*.txt\tfile.txt\n");
        buffer.clear();

        let result = check_ignore(
            dir.path(),
            dir.path().join("file.md").as_path(),
            false,
            &mut buffer,
        )
        .unwrap();
        assert_eq!(result, 1);
        assert_eq!(buffer, b"");
        buffer.clear();
    }
}
