// src/algorithms/co_occurrence.rs
use std::collections::HashMap;
use ahash::RandomState;

/// A struct to manage identifier-to-ID mapping and co-occurrence counts.
#[derive(Debug)] // Added derive for Debug for easier printing in tests
pub struct CoOccurrenceCounter {
    /// Maps identifier strings to their unique integer IDs.
    identifier_to_id: HashMap<String, u32, RandomState>,
    /// Stores the counts for each unique pair of integer IDs.
    /// The tuple (u32, u32) always stores the smaller ID first to ensure uniqueness.
    co_occurrence_counts: HashMap<(u32, u32), u32, RandomState>,
    /// The next available ID to assign to a new identifier.
    next_id: u32,
}

impl CoOccurrenceCounter {
    /// Creates a new, empty CoOccurrenceCounter.
    pub fn new() -> Self {
        CoOccurrenceCounter {
            identifier_to_id: HashMap::with_hasher(RandomState::new()),
            co_occurrence_counts: HashMap::with_hasher(RandomState::new()),
            next_id: 0,
        }
    }

    /// Processes a list of identifiers, updating the co-occurrence counts.
    pub fn process_list(&mut self, identifiers: &[String]) {
        let mut current_list_ids: Vec<u32> = Vec::with_capacity(identifiers.len());
        for id_str in identifiers {
            let id = *self.identifier_to_id.entry(id_str.clone()).or_insert_with(|| {
                let new_id = self.next_id;
                self.next_id += 1;
                new_id
            });
            current_list_ids.push(id);
        }

        if identifiers.len() < 2 {
            return;
        }

        for i in 0..current_list_ids.len() {
            for j in (i + 1)..current_list_ids.len() {
                let id1 = current_list_ids[i];
                let id2 = current_list_ids[j];

                let pair = if id1 < id2 { (id1, id2) } else { (id2, id1) };

                *self.co_occurrence_counts.entry(pair).or_insert(0) += 1;
            }
        }
    }

    /// Returns the current co-occurrence counts.
    #[cfg(test)]
    pub fn get_co_occurrence_counts(&self) -> &HashMap<(u32, u32), u32, RandomState> {
        &self.co_occurrence_counts
    }

    // /// Returns the mapping from identifier strings to their IDs.
    #[cfg(test)]
    pub fn get_identifier_to_id_map(&self) -> &HashMap<String, u32, RandomState> {
        &self.identifier_to_id
    }

    /// A helper to get the identifier string for a given ID.
    pub fn get_id_to_identifier_map(&self) -> HashMap<u32, String> {
        self.identifier_to_id.iter().map(|(s, &id)| (id, s.clone())).collect()
    }

    /// Gets co-occurrence metrics for a specific identifier.
    pub fn get_metrics_for_identifier(&self, target_id_str: &str) -> HashMap<String, u32> {
        let mut metrics = HashMap::new();

        let Some(&target_id) = self.identifier_to_id.get(target_id_str) else {
            return metrics;
        };

        let id_to_str_map = self.get_id_to_identifier_map();

        for (&(id_a, id_b), &count) in self.co_occurrence_counts.iter() {
            if id_a == target_id {
                let co_occurring_id_str = id_to_str_map.get(&id_b).unwrap();
                metrics.insert(co_occurring_id_str.clone(), count);
            } else if id_b == target_id {
                let co_occurring_id_str = id_to_str_map.get(&id_a).unwrap();
                metrics.insert(co_occurring_id_str.clone(), count);
            }
        }
        metrics
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const ID1_STR: &str = "ard:Y3JpZDovL2Rhc2Vyc3RlLmRlL3RhZ2Vzc2NoYXUyNA";
    const ID2_STR: &str = "zdf:zdf-magazin-royale-102";
    const ID3_STR: &str = "arte:RC-026195_de";
    const ID4_STR: &str = "some:other:identifier";

    #[test]
    fn test_single_list_processing() {
        let mut counter = CoOccurrenceCounter::new();
        let list1 = vec![ID1_STR.to_string(), ID2_STR.to_string(), ID3_STR.to_string()];
        counter.process_list(&list1);
        let id_map = counter.get_identifier_to_id_map();
        let id1 = *id_map.get(ID1_STR).unwrap();
        let id2 = *id_map.get(ID2_STR).unwrap();
        let id3 = *id_map.get(ID3_STR).unwrap();
        let counts = counter.get_co_occurrence_counts();
        assert_eq!(*counts.get(&(id1.min(id2), id1.max(id2))).unwrap(), 1);
        assert_eq!(*counts.get(&(id1.min(id3), id1.max(id3))).unwrap(), 1);
        assert_eq!(*counts.get(&(id2.min(id3), id2.max(id3))).unwrap(), 1);
        assert_eq!(counts.len(), 3);
    }

    #[test]
    fn test_multiple_lists_and_cumulative_counts() {
        let mut counter = CoOccurrenceCounter::new();
        let list1 = vec![ID1_STR.to_string(), ID2_STR.to_string(), ID3_STR.to_string()];
        counter.process_list(&list1);
        let list2 = vec![ID2_STR.to_string(), ID3_STR.to_string(), ID4_STR.to_string()];
        counter.process_list(&list2);
        let list3 = vec![ID1_STR.to_string(), ID3_STR.to_string()];
        counter.process_list(&list3);

        let id_map = counter.get_identifier_to_id_map();
        let id1 = *id_map.get(ID1_STR).unwrap();
        let id2 = *id_map.get(ID2_STR).unwrap();
        let id3 = *id_map.get(ID3_STR).unwrap();
        let id4 = *id_map.get(ID4_STR).unwrap();
        let counts = counter.get_co_occurrence_counts();

        println!("Identifier to ID map: {:?}", id_map);
        println!("Co-occurrence counts: {:?}", counts);

        assert_eq!(*counts.get(&(id1.min(id2), id1.max(id2))).unwrap(), 1);
        assert_eq!(*counts.get(&(id1.min(id3), id1.max(id3))).unwrap(), 2);
        assert_eq!(*counts.get(&(id2.min(id3), id2.max(id3))).unwrap(), 2);
        assert_eq!(*counts.get(&(id2.min(id4), id2.max(id4))).unwrap(), 1);
        assert_eq!(*counts.get(&(id3.min(id4), id3.max(id4))).unwrap(), 1);
        assert_eq!(counts.len(), 5);
    }

    #[test]
    fn test_empty_and_single_element_lists() {
        let mut counter = CoOccurrenceCounter::new();
        counter.process_list(&[]);
        assert!(counter.get_co_occurrence_counts().is_empty());
        assert!(counter.get_identifier_to_id_map().is_empty());

        counter.process_list(&[ID1_STR.to_string()]);
        assert!(counter.get_co_occurrence_counts().is_empty());
        assert_eq!(counter.get_identifier_to_id_map().len(), 1);
        assert!(counter.get_identifier_to_id_map().contains_key(ID1_STR));
    }

    #[test]
    fn test_get_metrics_for_identifier_existing() {
        let mut counter = CoOccurrenceCounter::new();
        let list1 = vec![ID1_STR.to_string(), ID2_STR.to_string(), ID3_STR.to_string()];
        counter.process_list(&list1);
        let list2 = vec![ID2_STR.to_string(), ID3_STR.to_string(), ID4_STR.to_string()];
        counter.process_list(&list2);
        let list3 = vec![ID1_STR.to_string(), ID3_STR.to_string()];
        counter.process_list(&list3);

        let metrics = counter.get_metrics_for_identifier(ID1_STR);
        assert_eq!(metrics.len(), 2);
        assert_eq!(*metrics.get(ID2_STR).unwrap(), 1);
        assert_eq!(*metrics.get(ID3_STR).unwrap(), 2);

        let metrics_for_id2 = counter.get_metrics_for_identifier(ID2_STR);
        assert_eq!(metrics_for_id2.len(), 3);
        assert_eq!(*metrics_for_id2.get(ID1_STR).unwrap(), 1);
        assert_eq!(*metrics_for_id2.get(ID3_STR).unwrap(), 2);
        assert_eq!(*metrics_for_id2.get(ID4_STR).unwrap(), 1);
    }

    #[test]
    fn test_get_metrics_for_identifier_non_existing() {
        let mut counter = CoOccurrenceCounter::new();
        let list1 = vec![ID1_STR.to_string(), ID2_STR.to_string()];
        counter.process_list(&list1);

        let metrics = counter.get_metrics_for_identifier("non_existent_id");
        assert!(metrics.is_empty());
    }
}