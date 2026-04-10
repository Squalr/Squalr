use crate::virtual_snapshots::{virtual_snapshot_query::VirtualSnapshotQuery, virtual_snapshot_query_result::VirtualSnapshotQueryResult};
use std::{
    collections::HashMap,
    time::{Duration, Instant},
};

#[derive(Clone, Debug)]
pub struct VirtualSnapshot {
    queries: Vec<VirtualSnapshotQuery>,
    query_results: HashMap<String, VirtualSnapshotQueryResult>,
    refresh_interval: Duration,
    query_version: u64,
    last_applied_query_version: Option<u64>,
    last_refresh_started_at: Option<Instant>,
    last_refresh_completed_at: Option<Instant>,
    is_refresh_in_progress: bool,
    refresh_in_progress_query_version: Option<u64>,
    generation: u64,
}

impl VirtualSnapshot {
    pub fn new(refresh_interval: Duration) -> Self {
        Self {
            queries: Vec::new(),
            query_results: HashMap::new(),
            refresh_interval,
            query_version: 0,
            last_applied_query_version: None,
            last_refresh_started_at: None,
            last_refresh_completed_at: None,
            is_refresh_in_progress: false,
            refresh_in_progress_query_version: None,
            generation: 0,
        }
    }

    pub fn get_queries(&self) -> &[VirtualSnapshotQuery] {
        &self.queries
    }

    pub fn get_query_results(&self) -> &HashMap<String, VirtualSnapshotQueryResult> {
        &self.query_results
    }

    pub fn get_generation(&self) -> u64 {
        self.generation
    }

    pub fn get_last_refresh_completed_at(&self) -> Option<Instant> {
        self.last_refresh_completed_at
    }

    pub fn get_is_refresh_in_progress(&self) -> bool {
        self.is_refresh_in_progress
    }

    pub fn set_refresh_interval(
        &mut self,
        refresh_interval: Duration,
    ) {
        self.refresh_interval = refresh_interval;
    }

    pub fn set_queries(
        &mut self,
        queries: Vec<VirtualSnapshotQuery>,
    ) {
        if self.queries == queries {
            return;
        }

        self.queries = queries;
        self.query_version = self.query_version.saturating_add(1);

        let active_query_ids = self
            .queries
            .iter()
            .map(|query| query.get_query_id().to_string())
            .collect::<std::collections::HashSet<String>>();

        self.query_results
            .retain(|query_id, _| active_query_ids.contains(query_id));
    }

    pub fn should_refresh(
        &self,
        now: Instant,
    ) -> bool {
        if self.queries.is_empty() || self.is_refresh_in_progress {
            return false;
        }

        if self.last_applied_query_version != Some(self.query_version) {
            return true;
        }

        self.last_refresh_completed_at
            .map(|last_refresh_completed_at| now.duration_since(last_refresh_completed_at) >= self.refresh_interval)
            .unwrap_or(true)
    }

    pub fn mark_refresh_started(
        &mut self,
        now: Instant,
    ) -> u64 {
        self.last_refresh_started_at = Some(now);
        self.is_refresh_in_progress = true;
        self.refresh_in_progress_query_version = Some(self.query_version);

        self.query_version
    }

    pub fn apply_refresh_results(
        &mut self,
        refresh_query_version: u64,
        query_results: HashMap<String, VirtualSnapshotQueryResult>,
        now: Instant,
    ) {
        if self.refresh_in_progress_query_version != Some(refresh_query_version) {
            return;
        }

        self.query_results = query_results;
        self.last_applied_query_version = Some(refresh_query_version);
        self.last_refresh_completed_at = Some(now);
        self.is_refresh_in_progress = false;
        self.refresh_in_progress_query_version = None;
        self.generation = self.generation.saturating_add(1);
    }

    pub fn cancel_refresh_if_version_matches(
        &mut self,
        refresh_query_version: u64,
    ) {
        if self.refresh_in_progress_query_version != Some(refresh_query_version) {
            return;
        }

        self.is_refresh_in_progress = false;
        self.refresh_in_progress_query_version = None;
    }
}

#[cfg(test)]
mod tests {
    use super::VirtualSnapshot;
    use crate::virtual_snapshots::virtual_snapshot_query::VirtualSnapshotQuery;
    use squalr_engine_api::structures::{memory::pointer::Pointer, structs::symbolic_struct_definition::SymbolicStructDefinition};
    use std::collections::HashMap;
    use std::time::{Duration, Instant};

    fn create_pointer_query(query_id: &str) -> VirtualSnapshotQuery {
        VirtualSnapshotQuery::Pointer {
            query_id: query_id.to_string(),
            pointer: Pointer::new(0x1234, vec![0x10], "game.exe".to_string()),
            symbolic_struct_definition: SymbolicStructDefinition::new_anonymous(Vec::new()),
        }
    }

    #[test]
    fn set_queries_marks_snapshot_dirty_when_query_set_changes() {
        let mut virtual_snapshot = VirtualSnapshot::new(Duration::from_millis(500));
        let now = Instant::now();

        virtual_snapshot.set_queries(vec![create_pointer_query("ammo")]);

        assert!(virtual_snapshot.should_refresh(now));
    }

    #[test]
    fn set_queries_does_not_mark_snapshot_dirty_when_query_set_is_unchanged() {
        let mut virtual_snapshot = VirtualSnapshot::new(Duration::from_millis(500));
        let now = Instant::now();
        let queries = vec![create_pointer_query("ammo")];

        virtual_snapshot.set_queries(queries.clone());
        let refresh_query_version = virtual_snapshot.mark_refresh_started(now);
        virtual_snapshot.apply_refresh_results(refresh_query_version, HashMap::new(), now);
        virtual_snapshot.set_queries(queries);

        assert!(!virtual_snapshot.should_refresh(now + Duration::from_millis(100)));
    }
}
