use crate::config::{self, Config};
use std::collections::{HashMap, HashSet, VecDeque};
use std::fmt;
use std::fs;
use std::path::PathBuf;

#[derive(Clone)]
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
    ConfigLoad(String),
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
        let (tools, _) = Self::load(true)?;
        Ok(tools)
    }

    pub(crate) fn new_relaxed() -> Result<(Self, Vec<String>), ToolError> {
        Self::load(false)
    }

    #[cfg(test)]
    pub(crate) fn new_empty() -> Self {
        Self {
            root: "~/.dotfiles".to_string(),
            ordered_ids: vec![],
            items: HashMap::new(),
        }
    }

    fn load(strict: bool) -> Result<(Self, Vec<String>), ToolError> {
        let config = Self::load_config()?;
        let root = config.root().to_string();
        let mut items = Self::build_tool_items(&config)?;
        let (dependency_map, warnings) = Self::sanitize_dependencies(&mut items, strict);

        Self::validate_dependencies(&items, &dependency_map)?;

        let ordered_ids = Self::topological_order(&items)?;
        Ok((
            Self {
                root,
                ordered_ids,
                items,
            },
            warnings,
        ))
    }

    fn load_config() -> Result<Config, ToolError> {
        Config::new().map_err(|error| ToolError::ConfigLoad(error.to_string()))
    }

    fn build_tool_items(config: &Config) -> Result<HashMap<String, ToolItem>, ToolError> {
        let mut items = HashMap::new();
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

        Ok(items)
    }

    fn sanitize_dependencies(
        items: &mut HashMap<String, ToolItem>,
        strict: bool,
    ) -> (HashMap<String, Vec<String>>, Vec<String>) {
        if strict {
            let dependency_map = Self::dependency_map_from_items(items);
            return (dependency_map, Vec::new());
        }

        let valid_ids: HashSet<String> = items.keys().cloned().collect();
        let mut warnings = Vec::new();

        for (tool_id, item) in items.iter_mut() {
            let mut retained = Vec::with_capacity(item.dependencies.len());
            for dependency in std::mem::take(&mut item.dependencies) {
                if valid_ids.contains(&dependency) {
                    retained.push(dependency);
                } else {
                    warnings.push(format!(
                        "Tool '{}' references missing dependency '{}'",
                        tool_id, dependency
                    ));
                }
            }
            item.dependencies = retained;
        }

        let dependency_map = Self::dependency_map_from_items(items);
        (dependency_map, warnings)
    }

    fn dependency_map_from_items(
        items: &HashMap<String, ToolItem>,
    ) -> HashMap<String, Vec<String>> {
        items
            .iter()
            .map(|(id, item)| (id.clone(), item.dependencies.clone()))
            .collect()
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

    pub(crate) fn index_of(&self, tool_id: &str) -> Option<usize> {
        self.ordered_ids.iter().position(|id| id == tool_id)
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

            lines.push(self.format_marked_tool(root, highlight_id));
            self.append_dependents_tree(root, "", highlight_id, &mut lines, &mut visited);

            if index + 1 != roots.len() {
                Self::push_blank_line(&mut lines);
            }
        }

        for tool in self.ordered_ids.iter().filter_map(|id| self.items.get(id)) {
            if visited.contains(&tool.id) {
                continue;
            }

            Self::push_blank_line(&mut lines);
            lines.push(self.format_marked_tool(tool, highlight_id));
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

    pub(crate) fn tool_path(&self, tool: &ToolItem) -> PathBuf {
        let mut root = config::expand_home_path(&self.root);
        root.push(&tool.root);
        root.push(&tool.file);
        root
    }

    pub(crate) fn root(&self) -> &str {
        &self.root
    }

    fn format_marked_tool(&self, tool: &ToolItem, highlight_id: Option<&str>) -> String {
        format!(
            "{} {}",
            Self::marker_for(highlight_id, &tool.id),
            tool.display_name()
        )
    }

    fn marker_for(highlight_id: Option<&str>, tool_id: &str) -> &'static str {
        if Some(tool_id) == highlight_id {
            "*"
        } else {
            "-"
        }
    }

    fn push_blank_line(lines: &mut Vec<String>) {
        if lines.last().is_some_and(|line| !line.is_empty()) {
            lines.push(String::new());
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
            let newly_visited = visited.insert(dependent.id.clone());
            let mut line = format!(
                "{prefix}{connector} {} {}",
                Self::marker_for(highlight_id, &dependent.id),
                dependent.display_name()
            );
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
            ToolError::ConfigLoad(message) => {
                write!(f, "Failed to load config: {message}")
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config;

    fn create_tool(name: &str, id: Option<&str>, dependencies: Vec<&str>) -> config::Tool {
        config::Tool {
            id: id.map(|s| s.to_string()),
            name: Some(name.to_string()),
            root: None,
            file: None,
            dependencies: dependencies.into_iter().map(|s| s.to_string()).collect(),
        }
    }

    fn create_tool_item(id: &str, dependencies: Vec<&str>) -> ToolItem {
        ToolItem {
            id: id.to_string(),
            name: id.to_string(),
            root: id.to_string(),
            file: format!("{id}.sh"),
            dependencies: dependencies.into_iter().map(|s| s.to_string()).collect(),
        }
    }

    #[test]
    fn test_generate_tool_id() {
        let mut name_counts = HashMap::new();
        let mut items = HashMap::new();

        let tool1 = create_tool("Tool A", Some("tool-a"), vec![]);
        let id1 = generate_tool_id(&mut name_counts, &items, &tool1);
        assert_eq!(id1, "tool-a");
        items.insert(id1.clone(), create_tool_item(&id1, vec![]));

        let tool2 = create_tool("Tool B", None, vec![]);
        let id2 = generate_tool_id(&mut name_counts, &items, &tool2);
        assert_eq!(id2, "tool-b");
        items.insert(id2.clone(), create_tool_item(&id2, vec![]));

        let tool3 = create_tool("Tool B", None, vec![]);
        let id3 = generate_tool_id(&mut name_counts, &items, &tool3);
        assert_eq!(id3, "tool-b-1");
    }

    #[test]
    fn test_topological_order_valid() {
        let mut items = HashMap::new();
        items.insert("a".to_string(), create_tool_item("a", vec![]));
        items.insert("b".to_string(), create_tool_item("b", vec!["a"]));
        items.insert("c".to_string(), create_tool_item("c", vec!["b"]));
        let order = Tools::topological_order(&items).unwrap();
        assert_eq!(order, vec!["a", "b", "c"]);
    }

    #[test]
    fn test_topological_order_cycle() {
        let mut items = HashMap::new();
        items.insert("a".to_string(), create_tool_item("a", vec!["c"]));
        items.insert("b".to_string(), create_tool_item("b", vec!["a"]));
        items.insert("c".to_string(), create_tool_item("c", vec!["b"]));
        let result = Tools::topological_order(&items);
        assert!(matches!(result, Err(ToolError::CycleDetected)));
    }

    #[test]
    fn test_execution_stages() {
        let mut items = HashMap::new();
        items.insert("a".to_string(), create_tool_item("a", vec![]));
        items.insert("b".to_string(), create_tool_item("b", vec!["a"]));
        items.insert("c".to_string(), create_tool_item("c", vec![]));
        items.insert("d".to_string(), create_tool_item("d", vec!["b", "c"]));

        let tools = Tools {
            root: "/".to_string(),
            ordered_ids: vec!["a".to_string(), "c".to_string(), "b".to_string(), "d".to_string()],
            items,
        };

        let stages = tools.execution_stages();
        assert_eq!(stages.len(), 3);
        assert_eq!(stages[0].len(), 2);
        assert!(stages[0].iter().any(|t| t.id == "a"));
        assert!(stages[0].iter().any(|t| t.id == "c"));
        assert_eq!(stages[1].len(), 1);
        assert_eq!(stages[1][0].id, "b");
        assert_eq!(stages[2].len(), 1);
        assert_eq!(stages[2][0].id, "d");
    }
}
