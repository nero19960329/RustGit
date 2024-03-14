// A blank line matches no files, so it can serve as a separator for readability.
// A line starting with # serves as a comment. Put a backslash ("\") in front of the first hash for patterns that begin with a hash.
// Trailing spaces are ignored unless they are quoted with backslash ("\").
// An optional prefix "!" which negates the pattern; any matching file excluded by a previous pattern will become included again. It is not possible to re-include a file if a parent directory of that file is excluded. Git doesn’t list excluded directories for performance reasons, so any patterns on contained files have no effect, no matter where they are defined. Put a backslash ("\") in front of the first "!" for patterns that begin with a literal "!", for example, "\!important!.txt".
// The slash "/" is used as the directory separator. Separators may occur at the beginning, middle or end of the .gitignore search pattern.
// If there is a separator at the beginning or middle (or both) of the pattern, then the pattern is relative to the directory level of the particular .gitignore file itself. Otherwise the pattern may also match at any level below the .gitignore level.
// If there is a separator at the end of the pattern then the pattern will only match directories, otherwise the pattern can match both files and directories.
// An asterisk "*" matches anything except a slash. The character "?" matches any one character except "/". The range notation, e.g. [a-zA-Z], can be used to match one of the characters in a range. See fnmatch(3) and the FNM_PATHNAME flag for a more detailed description.
use super::utils::get_rgit_dir;
use anyhow::Result;
use regex::Regex;
use std::fs;
use std::path;

#[derive(Debug)]
pub struct RGitIgnoreRule {
    pub rule: String,
    pub rgitignore_path: path::PathBuf,
    pub line_number: usize,
}

fn handle_spaces(rule: &str) -> String {
    // trailing "\ " -> " "
    // remove leading spaces
    // remove trailing spaces
    // don't care about spaces in the middle
    let mut result = String::new();
    let rule = rule.trim_start();
    if rule.len() <= 1 {
        return rule.to_string();
    }

    let rule_without_trailing_spaces = rule.trim_end();
    result.push_str(&rule_without_trailing_spaces);
    if rule_without_trailing_spaces.ends_with('\\')
        && rule.len() > rule_without_trailing_spaces.len()
    {
        result.push(' ');
    }

    result
}

fn load_ignore_rules(rgitignore_path: &path::Path) -> Result<Vec<RGitIgnoreRule>> {
    let content = fs::read_to_string(rgitignore_path)?;
    let mut rules = Vec::new();
    for (i, line) in content.lines().enumerate() {
        if line.starts_with('#') || line.is_empty() {
            continue;
        }

        let rule = handle_spaces(line);
        rules.push(RGitIgnoreRule {
            rule,
            rgitignore_path: rgitignore_path.to_path_buf(),
            line_number: i + 1,
        });
    }
    Ok(rules)
}

fn is_ignored_by_rule(file_path: &path::Path, rule: &RGitIgnoreRule) -> Result<Option<bool>> {
    // None: not matched
    // true: matched, should be ignored
    // false: matched, should not be ignored
    // make file path relative to the .rgitignore file
    if fs::metadata(&file_path).is_err() {
        return Ok(None);
    }
    let file_path = file_path.canonicalize()?;
    let is_dir = file_path.is_dir();
    let relative_file_path = file_path.strip_prefix(rule.rgitignore_path.parent().unwrap())?;
    // if the file path is a directory, add a trailing slash if it doesn't have one
    let relative_file_path = if is_dir {
        relative_file_path.join("")
    } else {
        relative_file_path.to_path_buf()
    };

    // handle negated
    let mut is_negated = false;
    let mut rule = rule.rule.as_str();
    if rule.starts_with('!') {
        is_negated = true;
        rule = &rule[1..];
    }

    // handle wildcards: *, ?
    let rule = rule.replace("*", "[^/]*").replace("?", "[^/]");
    // handle **
    let rule = rule.replace("**", ".*");

    let re = Regex::new(&format!("^(.*/)?{}(/.*)?$", rule))?;
    let is_matched = re.is_match(relative_file_path.to_str().unwrap());
    if is_matched {
        Ok(Some(!is_negated))
    } else {
        Ok(None)
    }
}

#[derive(Debug)]
pub struct RGitIgnoreResult {
    pub is_ignored: bool,
    pub matched_rule: Option<RGitIgnoreRule>,
}

fn path_equal(p: &path::Path, q: &path::Path) -> Result<bool> {
    Ok(p.canonicalize()? == q.canonicalize()?)
}

pub fn is_ignored(file_path: &path::Path) -> Result<RGitIgnoreResult> {
    let rgit_dir = get_rgit_dir(file_path)?;
    let mut cur_dir = file_path.parent().unwrap();
    let mut result = None;
    let mut matched_rule = None;

    loop {
        let rgitignore_path = cur_dir.join(".rgitignore");

        let mut rules = vec![RGitIgnoreRule {
            rule: ".rgit".to_string(),
            rgitignore_path: rgitignore_path.clone(),
            line_number: 0,
        }];
        if rgitignore_path.is_file() {
            rules.append(&mut load_ignore_rules(&rgitignore_path)?);
        }

        for rule in rules {
            let is_ignored = is_ignored_by_rule(file_path, &rule)?;

            if is_ignored.is_some()
                && (result.is_none()
                    || (result.is_some() && result.unwrap() && !is_ignored.unwrap()))
            {
                result = is_ignored;
                matched_rule = Some(rule);
            }
        }

        if path_equal(cur_dir, rgit_dir.parent().unwrap())? || result.is_some() {
            break;
        }
        cur_dir = cur_dir.parent().unwrap();
    }

    Ok(RGitIgnoreResult {
        is_ignored: result.unwrap_or(false),
        matched_rule,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::init_rgit_dir;
    use tempfile::tempdir;

    #[test]
    fn test_handle_spaces() {
        assert_eq!(handle_spaces("a\\ b"), "a\\ b");
        assert_eq!(handle_spaces("a\\ b "), "a\\ b");
        assert_eq!(handle_spaces("a\\ b\\ "), "a\\ b\\ ");
        assert_eq!(handle_spaces("a b\\ \\ "), "a b\\ \\ ");
        assert_eq!(handle_spaces("a"), "a");
    }

    #[test]
    fn test_load_ignore_rules() {
        let dir = tempdir().unwrap();
        let rgitignore_path = dir.path().join(".rgitignore");
        fs::write(&rgitignore_path, "a\nb\n\n\n# c\n").unwrap();
        let rules = load_ignore_rules(&rgitignore_path).unwrap();
        assert_eq!(rules.len(), 2);
        assert_eq!(rules[0].rule, "a");
        assert_eq!(rules[0].line_number, 1);
        assert_eq!(rules[1].rule, "b");
        assert_eq!(rules[1].line_number, 2);
    }

    #[test]
    fn test_is_ignored_by_rule() {
        let dir = tempdir().unwrap();
        let rgitignore_path = dir.path().join(".rgitignore");

        fs::create_dir_all(dir.path().join("a")).unwrap();
        fs::create_dir_all(dir.path().join("b/a")).unwrap();
        fs::create_dir_all(dir.path().join("c")).unwrap();
        fs::write(dir.path().join("b/a/c"), "").unwrap();
        fs::write(dir.path().join("b/acd"), "").unwrap();
        fs::write(dir.path().join("c/a"), "").unwrap();

        fs::write(dir.path().join("b/e.data"), "").unwrap();
        fs::write(dir.path().join("b/ecd.data"), "").unwrap();

        let rgitignore_rule = RGitIgnoreRule {
            rule: "a".to_string(),
            rgitignore_path: rgitignore_path.clone(),
            line_number: 1,
        };
        assert_eq!(
            is_ignored_by_rule(&dir.path().join("d/a"), &rgitignore_rule).unwrap(),
            None
        );
        assert_eq!(
            is_ignored_by_rule(&dir.path().join("a"), &rgitignore_rule).unwrap(),
            Some(true)
        );
        assert_eq!(
            is_ignored_by_rule(&dir.path().join("b/a"), &rgitignore_rule).unwrap(),
            Some(true)
        );
        assert_eq!(
            is_ignored_by_rule(&dir.path().join("b/a/c"), &rgitignore_rule).unwrap(),
            Some(true)
        );
        assert_eq!(
            is_ignored_by_rule(&dir.path().join("b/acd"), &rgitignore_rule).unwrap(),
            None
        );
        assert_eq!(
            is_ignored_by_rule(&dir.path().join("c/a"), &rgitignore_rule).unwrap(),
            Some(true)
        );

        let rgitignore_rule = RGitIgnoreRule {
            rule: "a/".to_string(),
            rgitignore_path: rgitignore_path.clone(),
            line_number: 1,
        };
        assert_eq!(
            is_ignored_by_rule(&dir.path().join("a"), &rgitignore_rule).unwrap(),
            Some(true)
        );
        assert_eq!(
            is_ignored_by_rule(&dir.path().join("b/a"), &rgitignore_rule).unwrap(),
            Some(true)
        );
        assert_eq!(
            is_ignored_by_rule(&dir.path().join("b/a/c"), &rgitignore_rule).unwrap(),
            None
        );
        assert_eq!(
            is_ignored_by_rule(&dir.path().join("b/acd"), &rgitignore_rule).unwrap(),
            None
        );
        assert_eq!(
            is_ignored_by_rule(&dir.path().join("c/a"), &rgitignore_rule).unwrap(),
            None
        );

        let rgitignore_rule = RGitIgnoreRule {
            rule: "!c".to_string(),
            rgitignore_path: rgitignore_path.clone(),
            line_number: 1,
        };
        assert_eq!(
            is_ignored_by_rule(&dir.path().join("a"), &rgitignore_rule).unwrap(),
            None
        );
        assert_eq!(
            is_ignored_by_rule(&dir.path().join("b/a"), &rgitignore_rule).unwrap(),
            None
        );
        assert_eq!(
            is_ignored_by_rule(&dir.path().join("b/a/c"), &rgitignore_rule).unwrap(),
            Some(false)
        );
        assert_eq!(
            is_ignored_by_rule(&dir.path().join("b/acd"), &rgitignore_rule).unwrap(),
            None
        );
        assert_eq!(
            is_ignored_by_rule(&dir.path().join("c/a"), &rgitignore_rule).unwrap(),
            Some(false)
        );

        let rgitignore_rule = RGitIgnoreRule {
            rule: "*.data".to_string(),
            rgitignore_path: rgitignore_path.clone(),
            line_number: 1,
        };
        assert_eq!(
            is_ignored_by_rule(&dir.path().join("b/e.data"), &rgitignore_rule).unwrap(),
            Some(true)
        );
        assert_eq!(
            is_ignored_by_rule(&dir.path().join("b/ecd.data"), &rgitignore_rule).unwrap(),
            Some(true)
        );

        let rgitignore_rule = RGitIgnoreRule {
            rule: "?.data".to_string(),
            rgitignore_path: rgitignore_path.clone(),
            line_number: 1,
        };
        assert_eq!(
            is_ignored_by_rule(&dir.path().join("b/e.data"), &rgitignore_rule).unwrap(),
            Some(true)
        );
        assert_eq!(
            is_ignored_by_rule(&dir.path().join("b/ecd.data"), &rgitignore_rule).unwrap(),
            None
        );

        let rgitignore_rule = RGitIgnoreRule {
            rule: "b/**/c".to_string(),
            rgitignore_path: rgitignore_path.clone(),
            line_number: 1,
        };
        assert_eq!(
            is_ignored_by_rule(&dir.path().join("b/a"), &rgitignore_rule).unwrap(),
            None
        );
        assert_eq!(
            is_ignored_by_rule(&dir.path().join("b/a/c"), &rgitignore_rule).unwrap(),
            Some(true)
        );
        assert_eq!(
            is_ignored_by_rule(&dir.path().join("b/acd"), &rgitignore_rule).unwrap(),
            None
        );
    }

    #[test]
    fn test_is_ignored() {
        let dir = tempdir().unwrap();
        init_rgit_dir(dir.path()).unwrap();

        let rgitignore_path = dir.path().join(".rgitignore");
        fs::write(&rgitignore_path, "*.txt\n").unwrap();

        fs::create_dir_all(dir.path().join("a")).unwrap();
        let subdir = dir.path().join("a");
        let sub_rgitignore_path = subdir.join(".rgitignore");
        fs::write(&sub_rgitignore_path, "import*.txt\n!important.txt\n").unwrap();

        fs::create_dir_all(subdir.join("b")).unwrap();
        let subsubdir = subdir.join("b");
        let subsub_rgitignore_path = subsubdir.join(".rgitignore");
        fs::write(&subsub_rgitignore_path, "i*.txt\n").unwrap();

        fs::write(dir.path().join("important.txt"), "").unwrap();
        fs::write(dir.path().join("a/important.txt"), "").unwrap();
        fs::write(dir.path().join("a/b/important.txt"), "").unwrap();
        fs::write(dir.path().join("a/b/test.txt"), "").unwrap();

        assert_eq!(
            is_ignored(&dir.path().join("important.txt"))
                .unwrap()
                .is_ignored,
            true
        );
        assert_eq!(
            is_ignored(&dir.path().join("a/important.txt"))
                .unwrap()
                .is_ignored,
            false
        );
        assert_eq!(
            is_ignored(&dir.path().join("a/b/important.txt"))
                .unwrap()
                .is_ignored,
            true,
        );
        assert_eq!(
            is_ignored(&dir.path().join("a/b/test.txt"))
                .unwrap()
                .is_ignored,
            true
        );
    }
}
