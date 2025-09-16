use anyhow::Context;
use jaq_interpret::{Ctx, FilterT as _, RcIter, Val};

use crate::{commands::store::lib::jaq_compile, store::Store};

pub fn invoke(mut store: Store, filter: String) -> anyhow::Result<()> {
    let filter = jaq_compile(&filter).context("Failed to compile jq query")?;

    store.retain(|metric| {
        let inputs = RcIter::new(core::iter::empty());
        let mut out = filter.run((Ctx::new([], &inputs), Val::from(metric.data.clone())));
        let result = out.next();
        let Some(Ok(Val::Bool(true))) = result else {
            return true;
        };
        false
    });

    Ok(())
}
