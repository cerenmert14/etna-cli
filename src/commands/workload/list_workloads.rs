use tabled::settings::{Extract, Style};

use crate::{experiment::ExperimentMetadata, workload::WorkloadMetadata};

pub fn invoke(
    experiment: ExperimentMetadata,
    language: String,
    kind: String,
) -> anyhow::Result<()> {
    match kind.as_str() {
        "experiment" => {
            let mut languages = experiment
                .workloads()
                .into_iter()
                .filter(|workload| language == "all" || language == workload.language)
                .collect::<Vec<WorkloadMetadata>>();

            languages.sort_by(|a, b| a.language.cmp(&b.language).then(a.name.cmp(&b.name)));

            let mut table = vec![("Language", "Name")];
            for workload in languages.iter() {
                table.push((&workload.language, &workload.name));
            }

            let mut table = tabled::Table::new(table);

            table
                .with(Extract::segment(1.., ..))
                .with(Style::modern_rounded());

            println!("{}", table);
        }
        "available" => {
            anyhow::bail!("'available' kind is not implemented yet");
        }
        _ => {
            anyhow::bail!("Invalid kind: {}", kind);
        }
    }

    Ok(())
}
