use crate::{Budget, Result, invalid};
use brimp_runtime::runtime::native::{FileUpload, NativePage};
use serde::Deserialize;
use std::path::{Path, PathBuf};

#[derive(Deserialize)]
#[serde(tag = "action", rename_all = "kebab-case", deny_unknown_fields)]
pub enum Action {
    Click {
        selector: String,
    },
    Focus {
        selector: String,
    },
    Hover {
        selector: String,
    },
    Tap {
        selector: String,
    },
    Fill {
        selector: String,
        text: String,
    },
    Type {
        selector: String,
        text: String,
    },
    Press {
        selector: String,
        key: String,
        #[serde(default)]
        modifiers: u8,
    },
    Drag {
        source: String,
        destination: String,
    },
    Upload {
        selector: String,
        files: Vec<PathBuf>,
    },
    Evaluate {
        source: String,
    },
    Wait {
        seconds: f64,
    },
    WaitSelector {
        selector: String,
    },
    WaitNetworkIdle {
        #[serde(default = "quiet")]
        milliseconds: u64,
    },
    AdvanceTime {
        milliseconds: f64,
    },
}
fn quiet() -> u64 {
    500
}
pub struct Actions {
    values: Vec<Action>,
    uploads: Vec<Vec<FileUpload>>,
}
impl Actions {
    pub fn load(path: Option<&Path>) -> Result<Self> {
        let values: Vec<Action> = match path {
            Some(path) => {
                serde_json::from_slice(&std::fs::read(path).map_err(invalid)?).map_err(invalid)?
            }
            None => Vec::new(),
        };
        let base = path.and_then(Path::parent).unwrap_or(Path::new("."));
        let mut uploads = Vec::new();
        for action in &values {
            let mut loaded = Vec::new();
            match action {
                Action::Upload { files, .. } => {
                    for path in files {
                        let bytes = std::fs::read(base.join(path)).map_err(invalid)?;
                        loaded.push(FileUpload {
                            name: path
                                .file_name()
                                .ok_or_else(|| invalid("invalid upload filename"))?
                                .to_string_lossy()
                                .into_owned(),
                            bytes,
                            mime_type: "application/octet-stream".into(),
                            last_modified: 0,
                        });
                    }
                }
                Action::Wait { seconds } => {
                    seconds_duration(*seconds)?;
                }
                Action::AdvanceTime { milliseconds }
                    if !milliseconds.is_finite() || *milliseconds < 0. =>
                {
                    return Err(invalid("invalid clock advance"));
                }
                Action::Press { modifiers, .. } if *modifiers > 15 => {
                    return Err(invalid("invalid key modifiers"));
                }
                _ => {}
            }
            uploads.push(loaded);
        }
        Ok(Self { values, uploads })
    }
    pub fn run(&self, page: &NativePage, budget: &Budget, max_tasks: usize) -> Result<()> {
        for (action, uploads) in self.values.iter().zip(&self.uploads) {
            budget.remaining()?;
            match action {
                Action::Click { selector } => page.click(selector)?,
                Action::Focus { selector } => page.focus(selector)?,
                Action::Hover { selector } => page.hover(selector)?,
                Action::Tap { selector } => page.tap(selector)?,
                Action::Fill { selector, text } => page.fill(selector, text)?,
                Action::Type { selector, text } => page.type_text(selector, text)?,
                Action::Press {
                    selector,
                    key,
                    modifiers,
                } => page.press(selector, key, *modifiers)?,
                Action::Drag {
                    source,
                    destination,
                } => page.drag_to(source, destination)?,
                Action::Upload { selector, .. } => {
                    page.set_input_files(selector, uploads.clone())?
                }
                Action::Evaluate { source } => {
                    crate::evaluate(page, source)?;
                }
                Action::Wait { seconds } => budget.sleep(seconds_duration(*seconds)?)?,
                Action::WaitSelector { selector } => page.wait_for_selector(
                    selector,
                    budget.remaining()?,
                    budget.cancellation.clone(),
                )?,
                Action::WaitNetworkIdle { milliseconds } => page.wait_for_network_idle(
                    std::time::Duration::from_millis(*milliseconds),
                    budget.remaining()?,
                    budget.cancellation.clone(),
                )?,
                Action::AdvanceTime { milliseconds } => {
                    page.advance_time(*milliseconds, max_tasks)?;
                }
            }
        }
        Ok(())
    }
}
pub fn seconds_duration(seconds: f64) -> Result<std::time::Duration> {
    std::time::Duration::try_from_secs_f64(seconds)
        .map_err(|_| invalid("wait must be finite nonnegative seconds"))
}
