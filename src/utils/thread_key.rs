pub fn get_canonical_key(mut node_ids: Vec<i64>) -> String {
    node_ids.sort_unstable();
    node_ids.iter().map(|id| id.to_string()).collect::<Vec<_>>().join("-")
}
