use jaq_interpret::{Filter, ParseCtx};

/// Compile a JQ program for filtering metrics.
pub fn jaq_compile(program: &str) -> anyhow::Result<Filter> {
    let mut defs = ParseCtx::new(Vec::new());
    defs.insert_natives(jaq_core::core());
    defs.insert_defs(jaq_std::std());

    let parser = jaq_parse::defs();

    // parse the include file
    let (f, errs) = jaq_parse::parse(include_str!("lib.jq"), parser);
    anyhow::ensure!(
        errs.is_empty(),
        format!("Failed to parse lib.jq {:?} with errors: {:?}", f, errs)
    );

    if let Some(f) = f {
        defs.insert_defs(f);
    } else {
        anyhow::bail!("Failed to parse lib.jq '{:?}' with errors: {:?}", f, errs);
    }

    // parse the filter
    let (f, errs) = jaq_parse::parse(program, jaq_parse::main());
    anyhow::ensure!(
        errs.is_empty(),
        format!(
            "Failed to parse the jq program {:?} with errors: {:?}",
            program, errs
        )
    );

    anyhow::ensure!(
        f.is_some(),
        format!(
            "Failed to parse the jq program {:?} with errors: {:?}",
            f, errs
        )
    );

    // compile the filter in the context of the given definitions
    let f = defs.compile(f.unwrap());
    anyhow::ensure!(
        defs.errs.is_empty(),
        format!(
            "Failed to compile the jq program with errors: {:?}",
            defs.errs
                .iter()
                .map(|e| format!("({}, {:?})", e.0, e.1))
                .collect::<Vec<String>>()
        )
    );

    Ok(f)
}
