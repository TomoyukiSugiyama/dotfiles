use crate::config::Config;
use std::collections::{HashMap, HashSet, VecDeque};
use std::fmt;
use std::fs;
use std::path::PathBuf;

pub(crate) struct Tools {
    pub root: String,
    ordered_ids: Vec<String>,
    items: HashMap<String, ToolItem>,
}

#[derive(Clone)]
pub(crate) struct ToolItem {
    pub id: String,
    pub name: String,
    pub root: String,
    pub file: String,
    pub dependencies: Vec<String>,
}

impl ToolItem {
    pub(crate) fn display_name(&self) -> String {
        format!("{} ({})", self.name, self.id)
    }
}

#[derive(Debug)]
pub enum ToolError {
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
        let mut dependency_map: HashMap<String, Vec<String>> = HashMap::new();
        let mut name_counts: HashMap<String, usize> = HashMap::new();

        for tool in config.tools() {
            let dependencies = tool.dependencies();
            let id = generate_tool_id(&mut name_counts, &items, tool);

            if dependencies.iter().any(|dependency| dependency == &id) {
                return Err(ToolError::SelfDependency(id));
            }
            if items.contains_key(&id) {
                return Err(ToolError::DuplicateId(id));
            }
            dependency_map.insert(id.clone(), dependencies.clone());
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

        Self::validate_dependencies(&items, &dependency_map)?;

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

    pub(crate) fn execution_stages(&self) -> Vec<Vec<ToolItem>> {
        let mut remaining = self.items.clone();
        let mut processed = HashSet::new();
        let mut stages = Vec::new();

        while !remaining.is_empty() {
            let mut stage: Vec<ToolItem> = self
                .ordered_ids
                .iter()
                .filter_map(|id| remaining.get(id))
                .filter(|tool| tool.dependencies.iter().all(|dep| processed.contains(dep)))
                .cloned()
                .collect();

            if stage.is_empty() {
                break;
            }

            stage.sort_by(|a, b| a.name.cmp(&b.name));

            for tool in &stage {
                processed.insert(tool.id.clone());
            }
            for tool in &stage {
                remaining.remove(&tool.id);
            }

            stages.push(stage);
        }

        stages
    }

    pub(crate) fn dependency_map_lines(&self, highlight_id: Option<&str>) -> Vec<String> {
        let mut lines = Vec::new();
        let mut visited = HashSet::new();

        let mut roots: Vec<&ToolItem> = self
            .ordered_ids
            .iter()
            .filter_map(|id| self.items.get(id))
            .filter(|tool| tool.dependencies.is_empty())
            .collect();

        if roots.is_empty() {
            roots = self
                .ordered_ids
                .iter()
                .filter_map(|id| self.items.get(id))
                .collect();
        }

        for (index, root) in roots.iter().enumerate() {
            if !visited.insert(root.id.clone()) {
                continue;
            }

            let marker = if Some(root.id.as_str()) == highlight_id {
                "*"
            } else {
                "-"
            };
            lines.push(format!("{marker} {}", root.display_name()));
            self.append_dependents_tree(root, "", highlight_id, &mut lines, &mut visited);

            if index + 1 != roots.len() && lines.last().is_some_and(|line| !line.is_empty()) {
                lines.push(String::new());
            }
        }

        for tool in self.ordered_ids.iter().filter_map(|id| self.items.get(id)) {
            if visited.contains(&tool.id) {
                continue;
            }

            if lines.last().is_some_and(|line| !line.is_empty()) {
                lines.push(String::new());
            }

            let marker = if Some(tool.id.as_str()) == highlight_id {
                "*"
            } else {
                "-"
            };
            lines.push(format!("{marker} {}", tool.display_name()));
            visited.insert(tool.id.clone());
            self.append_dependents_tree(tool, "", highlight_id, &mut lines, &mut visited);
        }

        if matches!(lines.last(), Some(last) if last.is_empty()) {
            lines.pop();
        }

        lines
    }

    pub(crate) fn execution_stage_index(&self, tool_id: &str) -> Option<usize> {
        let mut stage_map: HashMap<String, usize> = HashMap::new();

        for id in &self.ordered_ids {
            if let Some(tool) = self.items.get(id) {
                let max_dependency_stage = tool
                    .dependencies
                    .iter()
                    .filter_map(|dependency| stage_map.get(dependency))
                    .max()
                    .copied();

                let stage = max_dependency_stage.map_or(0, |stage| stage + 1);
                stage_map.insert(tool.id.clone(), stage);
            }
        }

        stage_map.get(tool_id).copied()
    }

    fn validate_dependencies(
        items: &HashMap<String, ToolItem>,
        dependency_map: &HashMap<String, Vec<String>>,
    ) -> Result<(), ToolError> {
        for (tool_id, dependencies) in dependency_map {
            for dependency in dependencies {
                if !items.contains_key(dependency) {
                    return Err(ToolError::MissingDependency {
                        tool_id: tool_id.clone(),
                        dependency_id: dependency.clone(),
                    });
                }
            }
        }
        Ok(())
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

    fn append_dependents_tree(
        &self,
        tool: &ToolItem,
        prefix: &str,
        highlight_id: Option<&str>,
        lines: &mut Vec<String>,
        visited: &mut HashSet<String>,
    ) {
        let mut dependents = self
            .items
            .values()
            .filter(|candidate| candidate.dependencies.iter().any(|dep| dep == &tool.id))
            .collect::<Vec<_>>();
        dependents.sort_by(|a, b| a.name.cmp(&b.name));

        let last_index = dependents.len().saturating_sub(1);
        for (index, dependent) in dependents.into_iter().enumerate() {
            let connector = if index == last_index { "`--" } else { "|--" };
            let marker = if Some(dependent.id.as_str()) == highlight_id {
                "*"
            } else {
                "-"
            };
            let newly_visited = visited.insert(dependent.id.clone());
            let mut line = format!("{prefix}{connector} {marker} {}", dependent.display_name());
            if !newly_visited {
                line.push_str(" (repeat)");
                lines.push(line);
                continue;
            }

            lines.push(line);

            let next_prefix = if index == last_index {
                format!("{prefix}    ")
            } else {
                format!("{prefix}|   ")
            };

            self.append_dependents_tree(dependent, &next_prefix, highlight_id, lines, visited);
        }
    }
}

impl fmt::Display for ToolError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
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

fn generate_tool_id(
    name_counts: &mut HashMap<String, usize>,
    items: &HashMap<String, ToolItem>,
    tool: &crate::config::Tool,
) -> String {
    if let Some(id) = tool.identifier() {
        id
    } else {
        let mut base = tool.name().to_lowercase().replace(' ', "-");
        if base.is_empty() {
            base = "tool".to_string();
        }

        let count = name_counts.entry(base.clone()).or_insert(0);
        let mut candidate = if *count == 0 {
            base.clone()
        } else {
            format!("{base}-{}", *count)
        };

        while items.contains_key(&candidate) {
            *count += 1;
            candidate = format!("{base}-{}", *count);
        }

        *count += 1;
        candidate
    }
}
