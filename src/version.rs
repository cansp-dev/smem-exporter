use git_version::git_version;

pub const VERSION: &str = git_version!(
    args = ["--tags", "--always", "--dirty"],
    fallback = "0.0.0"
);
