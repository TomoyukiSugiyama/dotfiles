use crate::config::Config;
use std::collections::{HashMap, VecDeque};
use std::fmt;
use std::fs;
use std::path::PathBuf;

pub(crate) struct Tools {
    pub root: String,
    ordered_ids: Vec<String>,
    items: HashMap<String, ToolItem>,
}

pub(crate) struct ToolItem {
    pub id: String,
    pub name: String,
    pub root: String,
    pub file: String,
    pub dependencies: Vec<String>,
}

#[derive(Debug)]
pub enum ToolError {
    MissingId {
        tool_name: String,
    },
    DuplicateId(String),
    MissingDependency {
        tool_id: String,
        dependency_id: String,
    },
    SelfDependency(String),
    CycleDetected,
}

impl Tools {
    pub(crate) fn new() -> Result<Self, ToolError> {
        let config = Config::new().expect("Failed to load config");
        let root = config.root().to_string();
        let mut items = HashMap::new();

        for tool in config.tools() {
            let id = tool.identifier().ok_or_else(|| ToolError::MissingId {
                tool_name: tool.name(),
            })?;
            let dependencies = tool.dependencies();
            if dependencies.iter().any(|dependency| dependency == &id) {
                return Err(ToolError::SelfDependency(id));
            }
            if items.contains_key(&id) {
                return Err(ToolError::DuplicateId(id));
            }
            items.insert(
                id.clone(),
                ToolItem {
                    id,
                    name: tool.name(),
                    root: tool.root_name(),
                    file: tool.file_name(),
                    dependencies,
                },
            );
        }

        let ordered_ids = Self::topological_order(&items)?;

        Ok(Self {
            root,
            ordered_ids,
            items,
        })
    }

    pub(crate) fn iter(&self) -> impl Iterator<Item = &ToolItem> {
        self.ordered_ids.iter().filter_map(|id| self.items.get(id))
    }

    pub(crate) fn get_by_index(&self, index: usize) -> Option<&ToolItem> {
        self.ordered_ids
            .get(index)
            .and_then(|id| self.items.get(id))
    }

    pub(crate) fn file_path(&self, tool: &ToolItem) -> String {
        self.tool_path(tool).to_string_lossy().into_owned()
    }

    pub(crate) fn raw_script(&self, tool: &ToolItem) -> Option<String> {
        fs::read_to_string(self.tool_path(tool)).ok()
    }

    fn tool_path(&self, tool: &ToolItem) -> PathBuf {
        let mut root = self.expand_home_path(&self.root);
        root.push(&tool.root);
        root.push(&tool.file);
        root
    }

    fn expand_home_path(&self, path: &str) -> PathBuf {
        if let Some(stripped) = path.strip_prefix("~/")
            && let Ok(home) = std::env::var("HOME")
        {
            PathBuf::from(home).join(stripped)
        } else {
            PathBuf::from(path)
        }
    }

    fn topological_order(items: &HashMap<String, ToolItem>) -> Result<Vec<String>, ToolError> {
        let mut in_degree: HashMap<&str, usize> = HashMap::new();
        let mut graph: HashMap<&str, Vec<&str>> = HashMap::new();

        for (id, item) in items.iter() {
            let id_str = id.as_str();
            in_degree.entry(id_str).or_insert(0);
            for dependency in &item.dependencies {
                let dependency_item =
                    items
                        .get(dependency)
                        .ok_or_else(|| ToolError::MissingDependency {
                            tool_id: id.clone(),
                            dependency_id: dependency.clone(),
                        })?;
                graph
                    .entry(dependency_item.id.as_str())
                    .or_default()
                    .push(id_str);
                *in_degree.entry(id_str).or_insert(0) += 1;
            }
        }

        let mut queue: VecDeque<&str> = in_degree
            .iter()
            .filter_map(|(id, &degree)| if degree == 0 { Some(*id) } else { None })
            .collect();

        let mut order = Vec::with_capacity(items.len());
        while let Some(id) = queue.pop_front() {
            order.push(id.to_string());
            if let Some(neighbors) = graph.get(id) {
                for &neighbor in neighbors {
                    if let Some(degree) = in_degree.get_mut(neighbor) {
                        *degree -= 1;
                        if *degree == 0 {
                            queue.push_back(neighbor);
                        }
                    }
                }
            }
        }

        if order.len() == items.len() {
            Ok(order)
        } else {
            Err(ToolError::CycleDetected)
        }
    }
}

impl fmt::Display for ToolError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ToolError::MissingId { tool_name } => {
                write!(
                    f,
                    "Tool '{tool_name}' must declare an Id when using dependencies"
                )
            }
            ToolError::DuplicateId(id) => write!(f, "Duplicate tool id detected: {id}"),
            ToolError::MissingDependency {
                tool_id,
                dependency_id,
            } => write!(
                f,
                "Tool '{tool_id}' references missing dependency '{dependency_id}'"
            ),
            ToolError::SelfDependency(id) => {
                write!(f, "Tool '{id}' cannot depend on itself")
            }
            ToolError::CycleDetected => write!(f, "Cycle detected in tool dependencies"),
        }
    }
}

impl std::error::Error for ToolError {}
