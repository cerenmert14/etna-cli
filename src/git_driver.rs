use std::path::{Path, PathBuf};

use anyhow::Context;
use tracing::debug;

const ETNA_COMMITTER_NAME: &str = "ETNA Commit Bot";
const ETNA_COMMITTER_EMAIL: &str = "etna-bot@users.noreply.github.com";

pub(crate) fn initialize_git_repo(path: &PathBuf, msg: &str) -> anyhow::Result<()> {
    // Initialize a git repository
    let git_repo = git2::Repository::init(path).context("Failed to initialize git repository")?;
    let mut index = git_repo.index().context("Failed to get index")?;
    index
        .add_all(["*"].iter(), git2::IndexAddOption::DEFAULT, None)
        .context("Failed to add files to index")?;
    index.write().context("Failed to write index")?;
    let tree_id = index.write_tree().context("Failed to write tree")?;
    let tree = git_repo.find_tree(tree_id).context("Failed to find tree")?;

    if let Ok(head) = git_repo.head() {
        if let Ok(head_commit) = head.peel_to_commit() {
            if head_commit.tree_id() == tree_id {
                debug!("No changes detected; skipping commit");
                return Ok(());
            }
        }
    }

    let signature = git2::Signature::now(ETNA_COMMITTER_NAME, ETNA_COMMITTER_EMAIL)
        .context("Failed to create signature")?;
    git_repo
        .commit(Some("HEAD"), &signature, &signature, msg, &tree, &[])
        .context("Failed to commit")?;
    Ok(())
}

pub(crate) fn _change_branch(repo_path: &PathBuf, branch: &str) -> anyhow::Result<()> {
    // Change the branch of the etna repository
    let git_repo = git2::Repository::open(repo_path).context("Failed to open git repository")?;
    let mut remote = git_repo
        .find_remote("origin")
        .context("Failed to find remote")?;
    remote
        .fetch(&[branch], None, None)
        .context("Failed to fetch remote")?;

    debug!(
        "list of branches: {:?}",
        git_repo
            .branches(None)
            .unwrap()
            .map(|branch| branch.unwrap().0.name().unwrap().unwrap().to_string())
            .collect::<Vec<_>>()
    );

    let origin_branch = format!("origin/{}", branch);
    let branch = git_repo
        .find_branch(&origin_branch, git2::BranchType::Remote)
        .context("Failed to find branch")?;
    let branch = branch.into_reference();
    let branch = branch
        .peel_to_commit()
        .context("Failed to peel to commit")?;
    let branch = branch.into_object();

    git_repo
        .reset(&branch, git2::ResetType::Hard, None)
        .context("Failed to reset branch")?;

    Ok(())
}

/// Commit the entire repo with the given message.
pub(crate) fn commit(repo_path: &Path, message: &str) -> anyhow::Result<String> {
    debug!("repo path: {}", repo_path.display());
    let git_repo = git2::Repository::open(repo_path).context("Failed to open git repository")?;

    let mut index = git_repo.index().context("Failed to get index")?;
    index.clear().context("Failed to clear index")?;

    index
        .add_all(["*"], git2::IndexAddOption::DEFAULT, None)
        .context("Failed to add files to index")?;

    index.write().context("Failed to write index")?;
    debug!(
        "index {:?}",
        index
            .iter()
            .map(|entry| std::ffi::CString::new(&entry.path[..]).unwrap())
            .collect::<Vec<_>>()
    );

    let tree_id = index.write_tree().context("Failed to write tree")?;
    let tree = git_repo.find_tree(tree_id).context("Failed to find tree")?;

    let signature = git2::Signature::now(ETNA_COMMITTER_NAME, ETNA_COMMITTER_EMAIL)
        .context("Failed to create signature")?;

    git_repo
        .commit(
            Some("HEAD"),
            &signature,
            &signature,
            &format!("automated commit: '{message}'",),
            &tree,
            &[&git_repo
                .head()
                .context("Failed to get head")?
                .peel_to_commit()
                .context("Failed to peel to commit")?],
        )
        .context("Failed to commit")?;

    let head = git_repo.head().context("Failed to get head")?;
    let head = head.peel_to_commit().context("Failed to peel to commit")?;
    Ok(head.id().to_string())
}

/// Get the hash of the head of a git repository
pub(crate) fn _head_hash(repo_path: &Path) -> anyhow::Result<String> {
    let git_repo = git2::Repository::open(repo_path).context("Failed to open git repository")?;
    let head = git_repo.head().context("Failed to get head")?;
    let head = head.peel_to_commit().context("Failed to peel to commit")?;
    Ok(head.id().to_string())
}

pub(crate) fn init_repo_via_cli(repo_path: &Path) -> anyhow::Result<()> {
    let url = std::env::var("ETNA_REMOTE")
        .unwrap_or_else(|_| "https://github.com/alpaylan/etna-cli.git".into());
    let status = std::process::Command::new("git")
        .arg("clone")
        .arg("--branch")
        .arg("main")
        .arg(&url)
        .arg(repo_path)
        .status()
        .context("Failed to execute git clone")?;
    if !status.success() {
        anyhow::bail!("git clone failed with status: {}", status);
    }
    Ok(())
}

pub(crate) fn pull_via_cli(repo_path: &Path) -> anyhow::Result<()> {
    tracing::debug!("Pulling path from remote");
    tracing::debug!("run: 'git -C {} pull'", repo_path.display(),);
    let status = std::process::Command::new("git")
        .arg("-C")
        .arg(repo_path)
        .arg("pull")
        .status()
        .context("Failed to execute git pull")?;

    if !status.success() {
        anyhow::bail!("git pull failed with status: {}", status);
    }
    tracing::debug!("Pulled path from remote");
    Ok(())
}
