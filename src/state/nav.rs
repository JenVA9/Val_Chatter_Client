#[derive(Debug, Clone, Default, PartialEq)]
pub struct NavState {
    pub selected_map: Option<i64>,
    pub selected_agent: Option<i64>,
    pub selected_site: Option<i64>,
    pub selected_type: Option<i64>,
}

impl NavState {
    pub fn selected_node_ids(&self) -> Vec<i64> {
        let mut ids = Vec::new();
        if let Some(id) = self.selected_map { ids.push(id); }
        if let Some(id) = self.selected_agent { ids.push(id); }
        if let Some(id) = self.selected_site { ids.push(id); }
        if let Some(id) = self.selected_type { ids.push(id); }
        ids
    }

    pub fn is_empty(&self) -> bool {
        self.selected_map.is_none()
            && self.selected_agent.is_none()
            && self.selected_site.is_none()
            && self.selected_type.is_none()
    }
}
