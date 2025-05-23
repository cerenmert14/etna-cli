use crate::{config::EtnaConfig, store::Store};


pub(crate) fn invoke(hash: Option<String>, name: Option<String>, show_all: bool) -> anyhow::Result<()> {
    #[derive(Debug)]
    enum HashOrName {
        Hash,
        Name
    }
    use HashOrName::*;

    let (typ, val) = match (hash, name) {
        (Some(_), Some(_)) => anyhow::bail!("cannot set both hash and name for the experiment"),
        (Some(hash), _) => (Hash, hash),
        (_, Some(name)) => (Name, name),
        (None, None) => anyhow::bail!("has to set either hash or name"),
    };

    log::trace!("showing experiment ({val}: {typ:?}) (show_all: {show_all})");
    let etna_config = EtnaConfig::get_etna_config()?;

    let store = Store::load(&etna_config.store_path())?;

    match (typ, show_all) {
        (Name, true) => {
            let experiments = store.get_all_experiments_by_name(&val);
            for experiment in experiments {
                println!("{:#?}", experiment);
            }
        }
        (Name, false) => {
            println!("{:#?}", store.get_experiment_by_name(&val)?);
        }
        (Hash, _) => {
            println!("{:#?}", store.get_experiment_by_id(&val)?);
        }
    };

    Ok(())
}
