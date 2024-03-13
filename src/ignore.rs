// use anyhow::Result;
// use regex::Regex;
// use std::collections::HashMap;
// use std::env;
// use std::fs::File;
// use std::io::{BufRead, BufReader};
// use std::path::{Path, PathBuf};

// #[derive(Debug)]
// struct IgnoreRule {
//     pattern: String,
//     negated: bool, // true if the pattern is negated
//     only_dir: bool,
//     ignore_file_path: PathBuf,
// }

// impl IgnoreRule {
//     fn new(pattern: String, negated: bool, only_dir: bool, ignore_file_path: PathBuf) -> Self {
//         IgnoreRule {
//             pattern,
//             negated,
//             only_dir,
//             ignore_file_path,
//         }
//     }

//     fn matches(&self, path: &Path, ignore_dir: &Path) -> Result<bool> {
//         let path_str = path.to_str().unwrap();
//         let pattern = self
//             .pattern
//             .replace("/**", "(/.*)?")
//             .replace("**", ".*")
//             .replace("*", "[^/]*")
//             .replace("?", "[^/]");
//         let regex_pattern = if self.pattern.starts_with('/') {
//             format!("^{}", pattern)
//         } else if self.pattern.starts_with("**/") {
//             format!("^.*{}", pattern)
//         } else if self.pattern.contains('/') {
//             format!("^{}", pattern)
//         } else if !self.pattern.contains('*') && !self.pattern.contains('?') {
//             format!("^(.*/)?{}$", regex::escape(&self.pattern))
//         } else {
//             format!("^(.*/)?{}", pattern)
//         };

//         let regex = Regex::new(&regex_pattern)?;
//         let matched = regex.is_match(path_str);

//         if self.only_dir && !matched {
//             return Ok(false);
//         }

//         if self.pattern.starts_with('/') && !path_str.starts_with(ignore_dir.to_str().unwrap()) {
//             return Ok(false);
//         }

//         Ok(matched)
//     }
// }

// #[derive(Debug)]
// pub struct RGitIgnore {
//     rules: Vec<IgnoreRule>,
// }

// impl RGitIgnore {
//     pub fn new(ignore_file_path: &Path) -> Result<Self> {
//         let mut rules = Vec::new();
//         rules.push(IgnoreRule::new(
//             ".rgit".to_string(),
//             false,
//             false,
//             ignore_file_path.to_path_buf(),
//         ));

//         Ok(RGitIgnore { rules })
//     }

//     pub fn add_rule(&mut self, pattern: String, ignore_file_path: &Path) {
//         let negated = pattern.starts_with('!');
//         let only_dir = pattern.ends_with('/');
//         let pattern = pattern
//             .trim_start_matches('!')
//             .trim_end_matches('/')
//             .to_string();

//         self.rules.push(IgnoreRule::new(
//             pattern,
//             negated,
//             only_dir,
//             ignore_file_path.to_path_buf(),
//         ));
//     }

//     pub fn is_ignored(&self, path: &Path, ignore_dir: &Path) -> Result<bool> {
//         let mut excluded = false;
//         let mut parent_excluded = false;

//         let rules_by_depth: HashMap<usize, Vec<&IgnoreRule>> = self
//             .rules
//             .iter()
//             .filter(|rule| path.starts_with(&rule.ignore_file_path.parent().unwrap()))
//             .fold(HashMap::new(), |mut acc, rule| {
//                 let depth = rule.ignore_file_path.components().count();
//                 acc.entry(depth).or_default().push(rule);
//                 acc
//             });

//         let mut depths: Vec<usize> = rules_by_depth.keys().cloned().collect();
//         depths.sort_unstable();

//         for depth in depths {
//             for rule in &rules_by_depth[&depth] {
//                 if rule.matches(path, ignore_dir)? {
//                     if rule.negated {
//                         if excluded && !parent_excluded {
//                             excluded = false;
//                         }
//                     } else {
//                         excluded = true;
//                         if rule.only_dir && path.parent().map_or(false, |p| p == path) {
//                             parent_excluded = true;
//                         }
//                     }
//                 }
//             }
//         }

//         Ok(excluded)
//     }

//     pub fn load_ignore_files(path: &Path) -> Vec<PathBuf> {
//         let mut ignore_files = Vec::new();
//         Self::find_ignore_files(path, &mut ignore_files);
//         ignore_files
//     }

//     fn find_ignore_files(path: &Path, ignore_files: &mut Vec<PathBuf>) {
//         if let Ok(entries) = path.read_dir() {
//             for entry in entries.flatten() {
//                 if let Ok(file_type) = entry.file_type() {
//                     if file_type.is_dir() {
//                         Self::find_ignore_files(&entry.path(), ignore_files);
//                     } else if entry.file_name() == ".rgitignore" {
//                         ignore_files.push(entry.path());
//                     }
//                 }
//             }
//         }
//     }
// }

// pub fn load_ignore_rules(ignore_files: &[PathBuf]) -> Result<RGitIgnore> {
//     let mut rgitignore = RGitIgnore::new()?;

//     for ignore_file in ignore_files {
//         if let Ok(file) = File::open(ignore_file) {
//             let reader = BufReader::new(file);
//             for line in reader.lines().flatten() {
//                 if !line.is_empty() && !line.starts_with('#') {
//                     rgitignore.add_rule(line, ignore_file);
//                 }
//             }
//         }
//     }

//     Ok(rgitignore)
// }

// #[cfg(test)]
// mod tests {
//     use super::*;
//     use std::fs::{self, write, File};
//     use tempfile::tempdir;

//     #[test]
//     fn test_is_ignored() {
//         let temp_dir = tempdir().unwrap();

//         fs::create_dir_all(temp_dir.path().join("test")).unwrap();
//         File::create(temp_dir.path().join("file.txt")).unwrap();
//         File::create(temp_dir.path().join("file.data")).unwrap();
//         File::create(temp_dir.path().join("important.txt")).unwrap();
//         File::create(temp_dir.path().join("test/file.txt")).unwrap();
//         fs::create_dir_all(temp_dir.path().join("other")).unwrap();
//         File::create(temp_dir.path().join("other/file.txt")).unwrap();
//         fs::create_dir_all(temp_dir.path().join("temp")).unwrap();
//         File::create(temp_dir.path().join("temp/file.data")).unwrap();
//         fs::create_dir_all(temp_dir.path().join("subdir/temp")).unwrap();
//         File::create(temp_dir.path().join("subdir/temp/file.data")).unwrap();
//         fs::create_dir_all(temp_dir.path().join(".github")).unwrap();
//         File::create(temp_dir.path().join(".github/file.data")).unwrap();

//         let mut rgitignore = RGitIgnore::new(
//             &temp_dir
//                 .path()
//                 .join(".rgitignore")
//                 .to_path_buf()
//                 .as_path(),
//         ).unwrap();
//         let ignore_file_pathbuf = temp_dir.path().join(".rgitignore");
//         let ignore_file_path = ignore_file_pathbuf.as_path();
//         rgitignore.add_rule("*.txt".to_string(), ignore_file_path);
//         rgitignore.add_rule("!important.txt".to_string(), ignore_file_path);
//         rgitignore.add_rule("test/".to_string(), ignore_file_path);
//         rgitignore.add_rule("**/temp/".to_string(), ignore_file_path);
//         rgitignore.add_rule(".git".to_string(), ignore_file_path);

//         let ignore_dir = temp_dir.path();

//         assert!(rgitignore
//             .is_ignored(&temp_dir.path().join("file.txt"), ignore_dir)
//             .unwrap());
//         assert!(!rgitignore
//             .is_ignored(&temp_dir.path().join("file.data"), ignore_dir)
//             .unwrap());
//         assert!(!rgitignore
//             .is_ignored(&temp_dir.path().join("important.txt"), ignore_dir)
//             .unwrap());
//         assert!(rgitignore
//             .is_ignored(&temp_dir.path().join("test"), ignore_dir)
//             .unwrap());
//         assert!(rgitignore
//             .is_ignored(&temp_dir.path().join("test/file.txt"), ignore_dir)
//             .unwrap());
//         assert!(rgitignore
//             .is_ignored(&temp_dir.path().join("other/file.txt"), ignore_dir)
//             .unwrap());
//         assert!(rgitignore
//             .is_ignored(&temp_dir.path().join("temp/file.data"), ignore_dir)
//             .unwrap());
//         assert!(rgitignore
//             .is_ignored(&temp_dir.path().join("subdir/temp/file.data"), ignore_dir)
//             .unwrap());
//         assert!(!rgitignore
//             .is_ignored(&temp_dir.path().join(".github/file.data"), ignore_dir)
//             .unwrap());
//     }

//     #[test]
//     fn test_load_ignore_rules() {
//         let temp_dir = tempdir().unwrap();
//         let ignore_file_path = temp_dir.path().join(".rgitignore");

//         fs::create_dir_all(temp_dir.path().join("test")).unwrap();
//         File::create(temp_dir.path().join("file.txt")).unwrap();
//         File::create(temp_dir.path().join("important.txt")).unwrap();
//         File::create(temp_dir.path().join("test/file.txt")).unwrap();

//         write(&ignore_file_path, "file.txt\n!important.txt\ntest/\n").unwrap();

//         let ignore_files = vec![ignore_file_path];
//         let rgitignore = load_ignore_rules(&ignore_files).unwrap();

//         assert!(rgitignore
//             .is_ignored(&temp_dir.path().join("file.txt"), temp_dir.path())
//             .unwrap());
//         assert!(!rgitignore
//             .is_ignored(&temp_dir.path().join("important.txt"), temp_dir.path())
//             .unwrap());
//         assert!(rgitignore
//             .is_ignored(&temp_dir.path().join("test"), temp_dir.path())
//             .unwrap());
//         assert!(rgitignore
//             .is_ignored(&temp_dir.path().join("test/file.txt"), temp_dir.path())
//             .unwrap());
//     }

//     #[test]
//     fn test_load_ignore_files() {
//         let temp_dir = tempdir().unwrap();
//         let subdir = temp_dir.path().join("subdir");
//         fs::create_dir_all(&subdir).unwrap();

//         let ignore_file_path1 = temp_dir.path().join(".rgitignore");
//         let ignore_file_path2 = subdir.join(".rgitignore");

//         write(&ignore_file_path1, "*.txt\n").unwrap();
//         write(&ignore_file_path2, "!important.txt\n").unwrap();

//         let ignore_files = RGitIgnore::load_ignore_files(temp_dir.path());

//         assert_eq!(ignore_files.len(), 2);
//         assert!(ignore_files.contains(&ignore_file_path1));
//         assert!(ignore_files.contains(&ignore_file_path2));

//         File::create(temp_dir.path().join("file.txt")).unwrap();
//         File::create(temp_dir.path().join("important.txt")).unwrap();
//         File::create(subdir.join("file.txt")).unwrap();
//         File::create(subdir.join("important.txt")).unwrap();

//         let rgitignore = load_ignore_rules(&ignore_files).unwrap();

//         assert!(rgitignore
//             .is_ignored(&temp_dir.path().join("file.txt"), temp_dir.path())
//             .unwrap());
//         assert!(rgitignore
//             .is_ignored(&temp_dir.path().join("important.txt"), temp_dir.path())
//             .unwrap());
//         assert!(rgitignore
//             .is_ignored(&subdir.join("file.txt"), temp_dir.path())
//             .unwrap());
//         assert!(!rgitignore
//             .is_ignored(&subdir.join("important.txt"), temp_dir.path())
//             .unwrap());
//     }
// }
