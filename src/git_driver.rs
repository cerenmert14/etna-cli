use std::{
    fs,
    path::{Path, PathBuf},
};

use anyhow::Context;
use git2::{build::CheckoutBuilder, AutotagOption, FetchOptions, Repository};
use tracing::debug;

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

    let signature = git2::Signature::now("Alperen Keles", "akeles@umd.edu")
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

    let signature = git2::Signature::now("ETNA Commit Bot", "akeles@umd.edu")
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

/// Get the hash of a path in a git repository
#[allow(dead_code)]
pub(crate) fn hash(repo_path: &Path, index_path: &Path) -> anyhow::Result<String> {
    debug!("repo path: {}", repo_path.display());
    let git_repo = git2::Repository::open(repo_path).context("Failed to open git repository")?;

    debug!("index path: {}", index_path.display());
    let mut index = git_repo.index().context("Failed to get index")?;
    index.clear().context("Failed to clear index")?;

    index
        .add_all([index_path], git2::IndexAddOption::DEFAULT, None)
        .context("Failed to add files to index")?;

    debug!(
        "index {:?}",
        index
            .iter()
            .map(|entry| std::ffi::CString::new(&entry.path[..]).unwrap())
            .collect::<Vec<_>>()
    );

    let tree_id = index.write_tree().context("Failed to write tree")?;
    let tree = git_repo.find_tree(tree_id).context("Failed to find tree")?;

    debug!("tree id: {}", tree.id());

    Ok(tree.id().to_string())
}

/// Get the hash of the head of a git repository
pub(crate) fn _head_hash(repo_path: &Path) -> anyhow::Result<String> {
    let git_repo = git2::Repository::open(repo_path).context("Failed to open git repository")?;
    let head = git_repo.head().context("Failed to get head")?;
    let head = head.peel_to_commit().context("Failed to peel to commit")?;
    Ok(head.id().to_string())
}

/// Create a repo with only `.git/` populated—no files checked out.
#[allow(dead_code)]
pub fn init_metadata_only(repo_path: &Path, branch: &str) -> anyhow::Result<()> {
    let url = std::env::var("ETNA_REMOTE")
        .unwrap_or_else(|_| "https://github.com/alpaylan/etna-cli.git".to_string());

    let repo = match Repository::open(repo_path) {
        Ok(r) => r,
        Err(_) => {
            fs::create_dir_all(repo_path)?;
            Repository::init(repo_path)?
        }
    };

    // Ensure origin URL
    match repo.find_remote("origin") {
        Ok(_) => repo.remote_set_url("origin", &url)?,
        Err(_) => {
            repo.remote("origin", &url)?;
        }
    }

    // Shallow fetch refs/heads/<branch> -> refs/remotes/origin/<branch>
    let refspec = format!("refs/heads/{b}:refs/remotes/origin/{b}", b = branch);
    let mut fo = FetchOptions::new();
    fo.download_tags(AutotagOption::None);
    fo.depth(1);

    let mut remote = repo.find_remote("origin")?;
    remote
        .fetch(&[&refspec], Some(&mut fo), None)
        .with_context(|| format!("fetching {branch} from {url}"))?;

    // Resolve fetched tip
    let remote_ref = format!("refs/remotes/origin/{branch}");
    let tip = repo.refname_to_id(&remote_ref)?;
    let commit = repo.find_commit(tip)?;

    // Optionally create/update a local branch (handy later), but…
    if repo
        .find_reference(&format!("refs/heads/{branch}"))
        .is_err()
    {
        let _ = repo.branch(branch, &commit, true)?;
    }

    // …critically, **detach** HEAD to a valid commit so checkout works even if the local
    // branch ref is missing/corrupt in future runs.
    repo.set_head_detached(commit.id())?;

    // No checkout → working tree stays empty.
    Ok(())
}

#[allow(dead_code)]
fn materialize_paths(repo_path: &Path, paths: &[&str]) -> anyhow::Result<()> {
    tracing::debug!(
        "Materializing paths '{:?}' from remote in repo at '{}'",
        paths,
        repo_path.display()
    );
    let repo = Repository::open(repo_path)?;
    let mut remote = repo.find_remote("origin")?;
    let mut fo = FetchOptions::new();
    fo.download_tags(AutotagOption::None);
    fo.depth(1);
    let refspec = format!("+refs/heads/{b}:refs/remotes/origin/{b}", b = "main");
    tracing::trace!("Fetching paths '{:?}'", paths);
    remote.fetch(&[&refspec], Some(&mut fo), None)?;
    // Turn on sparse-checkout with an empty pattern file first (matches nothing)
    tracing::trace!("Setting up sparse checkout for paths '{:?}'", paths);
    repo.config()?.set_bool("core.sparseCheckout", true)?;
    let info = repo.path().join("info/sparse-checkout");
    tracing::trace!("Writing sparse-checkout info to '{}'", info.display());
    fs::create_dir_all(info.parent().unwrap())?;
    fs::write(&info, String::new())?;

    // Now write the paths you want (cone-style)
    let content = paths
        .iter()
        .map(|p| format!("/{}\n", p))
        .collect::<String>();
    tracing::trace!(
        "Writing sparse-checkout paths to '{}':\n{}",
        info.display(),
        content
    );
    fs::write(&info, content)?;

    // Checkout only those paths
    let mut co = CheckoutBuilder::new();
    co.force().remove_untracked(true).remove_ignored(true);
    for p in paths {
        tracing::trace!("Checking out path '{}'", p);
        co.path(p);
    }
    tracing::debug!("Checking out paths '{:?}'", paths);
    repo.checkout_head(Some(&mut co))?;
    Ok(())
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

#[allow(dead_code)]
fn pull_path(repo_path: &Path, path: &Path) -> anyhow::Result<()> {
    tracing::debug!("Pulling path '{}' from remote", path.display());
    materialize_paths(repo_path, &[&path.display().to_string()])
        .context("Failed to materialize paths")
}

#[allow(dead_code)]
pub(crate) fn pull_workload(
    repo_path: &Path,
    language: &str,
    workload: &str,
) -> anyhow::Result<()> {
    tracing::debug!("Pulling workload '{language}/{workload}' from remote");
    let subdir = format!("workloads/{}/{}", language, workload);

    materialize_paths(repo_path, &[&subdir]).context("Failed to materialize paths")
}
