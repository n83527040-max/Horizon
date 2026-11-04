//! Aggregate distinct touch bindings that resolve to the same engine input.
use std::collections::BTreeMap;

/// Tracks bindings in event order, so a quick down/up between frames is not lost.
#[derive(Debug)]
pub struct InputOwners<T> {
    owners: BTreeMap<String, Vec<T>>,
}
impl<T> Default for InputOwners<T> {
    fn default() -> Self { Self { owners: BTreeMap::new() } }
}
impl<T: Clone + PartialEq> InputOwners<T> {
    pub fn contains(&self, input: &T) -> bool {
        self.owners.values().any(|inputs| inputs.contains(input))
    }
    /// Capture resolved inputs on down, retain them even if bindings change,
    /// and emit only the first press and last release for each input.
    pub fn edge(&mut self, token: &str, down: bool, inputs: Vec<T>) -> Vec<(T, bool)> {
        if down {
            if self.owners.contains_key(token) { return Vec::new(); }
            let mut unique = Vec::new();
            for input in inputs { if !unique.contains(&input) { unique.push(input); } }
            let edges = unique.iter().filter(|i| !self.contains(i)).cloned().map(|i| (i, true)).collect();
            self.owners.insert(token.to_owned(), unique);
            edges
        } else {
            self.owners.remove(token).unwrap_or_default().into_iter()
                .filter(|input| !self.contains(input)).map(|input| (input, false)).collect()
        }
    }
}
#[cfg(test)]
mod tests {
    use super::InputOwners;
    #[test]
    fn distinct_tokens_share_an_action_and_release_only_after_the_last_owner() {
        let mut o = InputOwners::default();
        assert_eq!(o.edge("action:jump", true, vec![1]), vec![(1, true)]);
        assert!(o.edge("key:space", true, vec![1]).is_empty());
        assert!(o.edge("action:jump", false, vec![]).is_empty());
        assert!(o.contains(&1));
        assert_eq!(o.edge("key:space", false, vec![]), vec![(1, false)]);
        assert!(!o.contains(&1));
    }
    #[test]
    fn repeat_duplicate_outputs_and_changed_mappings_do_not_strand_inputs() {
        let mut o = InputOwners::default();
        assert_eq!(o.edge("a", true, vec![1, 1, 2]), vec![(1, true), (2, true)]);
        assert!(o.edge("a", true, vec![3]).is_empty());
        assert_eq!(o.edge("a", false, vec![3]), vec![(1, false), (2, false)]);
        assert!(o.edge("a", false, vec![]).is_empty());
        assert_eq!(o.edge("tap", true, vec![7]), vec![(7, true)]);
        assert_eq!(o.edge("tap", false, vec![]), vec![(7, false)]);
    }
}
