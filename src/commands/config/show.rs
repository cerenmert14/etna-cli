use tabled::{
    settings::{themes::ColumnNames, Extract, Style},
    Table,
};

pub fn invoke() -> anyhow::Result<()> {
    let etna_config = crate::config::EtnaConfig::get_etna_config()?;
    let table = vec![
        ("", "".to_string()),
        ("Path", etna_config.etna_dir.display().to_string()),
    ];

    let mut table = Table::new(table);

    table
        .with(Extract::segment(1.., ..))
        .with(Style::modern_rounded())
        .with(ColumnNames::default());

    println!("{}", table);

    Ok(())
}
