use anyhow::Context;
use jaq_interpret::{Ctx, FilterT as _, RcIter, Val};

use crate::{
    commands::store::lib::jaq_compile,
    config::{EtnaConfig, ExperimentConfig},
    store::Store,
};

pub fn invoke(experiment: Option<String>, filter: String) -> anyhow::Result<()> {
    let etna_config = EtnaConfig::get_etna_config()?;

    let experiment_config = match experiment {
        Some(name) => ExperimentConfig::from_etna_config(&name, &etna_config).context(format!(
            "Failed to get experiment config for '{}'",
            name
        )),
        None => ExperimentConfig::from_current_dir().context("No experiment name is provided, and the current directory is not an experiment directory"),
    }?;
    // Load the Store
    let mut store = Store::load(&experiment_config.store)?;

    let filter = jaq_compile(&filter).context("Failed to compile jq query")?;

    store.metrics.retain(|metric| {
        let inputs = RcIter::new(core::iter::empty());
        let mut out = filter.run((Ctx::new([], &inputs), Val::from(metric.data.clone())));
        let result = out.next();
        let Some(Ok(Val::Bool(true))) = result else {
            return true;
        };
        false
    });

    store.save().context("Failed to save the store")?;

    Ok(())
}
