use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};

use crossterm::style::Color;

use crate::list_picker::{self, PickerItem};

pub fn select_model(models: &[String]) -> io::Result<Option<String>> {
    if models.is_empty() {
        println!("Select model: ");
        return Ok(None);
    }

    let models = ordered_models(models);
    let items: Vec<PickerItem> = models
        .iter()
        .map(|model| PickerItem {
            display: if model.installed {
                format!("{} (installed)", model.model)
            } else {
                model.model.clone()
            },
            value: model.model.clone(),
            color: if model.installed {
                Some(Color::Green)
            } else {
                None
            },
        })
        .collect();

    let selected = list_picker::select_value(&items, "Select model:")?;

    Ok(selected)
}

pub fn print_model_list(models: &[String]) -> io::Result<()> {
    let mut stdout = io::stdout();

    for model in ordered_models(models) {
        if model.installed {
            writeln!(stdout, "{} (installed)", model.model)?;
        } else {
            writeln!(stdout, "{}", model.model)?;
        }
    }

    Ok(())
}

fn ordered_models(models: &[String]) -> Vec<ModelEntry> {
    let mut entries: Vec<ModelEntry> = models
        .iter()
        .map(|model| ModelEntry {
            model: model.clone(),
            installed: model_is_installed(model),
        })
        .collect();

    sort_model_entries(&mut entries);
    entries
}

fn model_is_installed(model: &str) -> bool {
    cache_roots()
        .into_iter()
        .map(|root| root.join(repo_cache_dir_name(model)).join("snapshots"))
        .any(|snapshots| snapshots.is_dir() && has_entries(&snapshots))
}

fn has_entries(path: &Path) -> bool {
    match fs::read_dir(path) {
        Ok(mut entries) => entries.next().is_some(),
        Err(_) => false,
    }
}

fn cache_roots() -> Vec<PathBuf> {
    let mut roots = Vec::new();

    if let Some(path) = std::env::var_os("HUGGINGFACE_HUB_CACHE") {
        roots.push(PathBuf::from(path));
    }

    if let Some(path) = std::env::var_os("HF_HOME") {
        roots.push(PathBuf::from(path).join("hub"));
    }

    if let Some(path) = std::env::var_os("XDG_CACHE_HOME") {
        roots.push(PathBuf::from(path).join("huggingface").join("hub"));
    }

    if let Some(home) = std::env::var_os("HOME") {
        roots.push(
            PathBuf::from(home)
                .join(".cache")
                .join("huggingface")
                .join("hub"),
        );
    }

    roots
}

fn repo_cache_dir_name(model: &str) -> String {
    format!("models--{}", model.replace('/', "--"))
}

fn sort_model_entries(entries: &mut [ModelEntry]) {
    entries.sort_by(|a, b| b.installed.cmp(&a.installed));
}

#[derive(Clone)]
struct ModelEntry {
    model: String,
    installed: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ordered_models_put_installed_items_first() {
        let mut entries = vec![
            ModelEntry {
                model: "org/uninstalled-a".to_string(),
                installed: false,
            },
            ModelEntry {
                model: "org/installed-a".to_string(),
                installed: true,
            },
            ModelEntry {
                model: "org/uninstalled-b".to_string(),
                installed: false,
            },
            ModelEntry {
                model: "org/installed-b".to_string(),
                installed: true,
            },
        ];

        sort_model_entries(&mut entries);

        let ordered: Vec<_> = entries.into_iter().map(|entry| entry.model).collect();

        assert_eq!(
            ordered,
            vec![
                "org/installed-a".to_string(),
                "org/installed-b".to_string(),
                "org/uninstalled-a".to_string(),
                "org/uninstalled-b".to_string(),
            ]
        );
    }
}
