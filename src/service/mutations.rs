use std::collections::HashMap;
use std::path::{Path, PathBuf};

use crate::service::types::{FileMutationsInfo, MutationInfo, ServiceResult};

/// List all mutations in a directory using the marauders API
pub fn list_mutations(path: &Path) -> ServiceResult<Vec<FileMutationsInfo>> {
    let project = marauders::Project::new(path, None)?;
    let variations = marauders::list_variations(&project);

    // Group by file
    let mut file_map: HashMap<PathBuf, Vec<MutationInfo>> = HashMap::new();

    for var_info in variations {
        for (i, variant_name) in var_info.variants.iter().enumerate() {
            let mutation = MutationInfo {
                name: variant_name.clone(),
                active: var_info.active == i + 1, // active is 1-indexed for variants (0 = base)
                file: var_info.path.clone(),
                line: var_info.line,
                end_line: var_info.line, // API doesn't provide end_line, use start
            };
            file_map
                .entry(var_info.path.clone())
                .or_default()
                .push(mutation);
        }
    }

    Ok(file_map
        .into_iter()
        .map(|(file, mutations)| FileMutationsInfo { file, mutations })
        .collect())
}

/// Get mutations for a specific file with line locations
pub fn get_file_mutations(file_path: &Path) -> ServiceResult<FileMutationsInfo> {
    let parent = file_path
        .parent()
        .ok_or_else(|| anyhow::anyhow!("File has no parent directory"))?;

    let file_name = file_path
        .file_name()
        .and_then(|n| n.to_str())
        .ok_or_else(|| anyhow::anyhow!("Invalid file path"))?;

    let pattern = format!("**/{}", file_name);
    let project = marauders::Project::new(parent, Some(&pattern))?;
    let variations = marauders::list_variations(&project);

    let mutations: Vec<MutationInfo> = variations
        .iter()
        .filter(|v| v.path == file_path || v.path.ends_with(file_name))
        .flat_map(|var_info| {
            var_info
                .variants
                .iter()
                .enumerate()
                .map(|(i, name)| MutationInfo {
                    name: name.clone(),
                    active: var_info.active == i + 1,
                    file: var_info.path.clone(),
                    line: var_info.line,
                    end_line: var_info.line,
                })
        })
        .collect();

    Ok(FileMutationsInfo {
        file: file_path.to_path_buf(),
        mutations,
    })
}

/// Set a mutation variant as active
pub fn set_mutation(path: &Path, variant: &str, _glob: Option<&str>) -> ServiceResult<()> {
    let mut project = marauders::Project::new(path, None)?;
    marauders::set_variant(&mut project, variant)?;
    Ok(())
}

/// Reset all mutations in a directory
pub fn reset_mutations(path: &Path) -> ServiceResult<()> {
    let mut project = marauders::Project::new(path, None)?;
    marauders::reset_all(&mut project)?;
    Ok(())
}
