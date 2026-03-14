use std::path::Path;

pub fn normalize_test_name(input: &str) -> String {
    let base = Path::new(input)
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or(input);
    base.strip_suffix(".json").unwrap_or(base).to_string()
}

pub fn resolve_test_name(input: &str, available: &[String]) -> anyhow::Result<String> {
    let normalized = normalize_test_name(input);

    let mut case_insensitive_match = None;

    for name in available {
        if name == input || name == &normalized {
            return Ok(name.clone());
        }
        if case_insensitive_match.is_none() && name.eq_ignore_ascii_case(&normalized) {
            case_insensitive_match = Some(name.clone());
        }
    }

    if let Some(found) = case_insensitive_match {
        return Ok(found);
    }

    anyhow::bail!("test not found")
}

fn levenshtein(a: &str, b: &str) -> usize {
    let b_chars = b.chars().collect::<Vec<_>>();
    let mut prev = (0..=b_chars.len()).collect::<Vec<_>>();
    let mut curr = vec![0; b_chars.len() + 1];

    for (i, ca) in a.chars().enumerate() {
        curr[0] = i + 1;
        for (j, cb) in b_chars.iter().enumerate() {
            let cost = usize::from(ca != *cb);
            curr[j + 1] = (prev[j + 1] + 1).min(curr[j] + 1).min(prev[j] + cost);
        }
        std::mem::swap(&mut prev, &mut curr);
    }

    prev[b_chars.len()]
}

fn test_suggestions(query: &str, available: &[String]) -> Vec<String> {
    let normalized = normalize_test_name(query);
    let mut ranked = available
        .iter()
        .map(|name| {
            let is_prefix = name.starts_with(&normalized) || normalized.starts_with(name);
            let contains = name.contains(&normalized);
            let distance = levenshtein(&normalized, name);
            (
                !is_prefix, // prefixes first
                !contains,  // then contains
                distance,
                name.len().abs_diff(normalized.len()),
                name.clone(),
            )
        })
        .collect::<Vec<_>>();

    ranked.sort();
    ranked
        .into_iter()
        .take(5)
        .map(|(_, _, _, _, n)| n)
        .collect()
}

pub fn build_invalid_test_message(input: &str, available: &[String]) -> String {
    let suggestions = test_suggestions(input, available);
    let available_preview = available
        .iter()
        .take(10)
        .cloned()
        .collect::<Vec<_>>()
        .join(", ");
    let suggested = suggestions.join(", ");

    if suggestions.is_empty() {
        format!(
            "Test '{}' was not found. Available tests: {}",
            input, available_preview
        )
    } else {
        format!(
            "Test '{}' was not found. Did you mean: {}? Available tests: {}. You can pass tests with or without the '.json' suffix.",
            input, suggested, available_preview
        )
    }
}
