#[cfg(test)]
mod search_tests {
    use crate::search::NeuralSearch;

    #[test]
    fn test_search_creation() {
        let search = NeuralSearch::new();
        let results = search.search("fn", &vec![("test.rs".to_string(), "fn main() {}".to_string())]);
        assert!(!results.is_empty());
    }

    #[test]
    fn test_exact_match() {
        let search = NeuralSearch::new();
        let results = search.search(
            "main",
            &vec![("main.rs".to_string(), "fn main() {}".to_string())],
        );
        assert!(!results.is_empty());
        assert!(results[0].score > 0.5);
    }

    #[test]
    fn test_no_match() {
        let search = NeuralSearch::new();
        let results = search.search(
            "xyz123",
            &vec![("test.rs".to_string(), "fn main() {}".to_string())],
        );
        assert!(results.is_empty());
    }

    #[test]
    fn test_search_by_type() {
        let search = NeuralSearch::new();
        let results = search.search_by_type(
            "Config",
            &vec![("config.rs".to_string(), "struct Config {}".to_string())],
        );
        assert!(!results.is_empty());
    }

    #[test]
    fn test_search_by_function() {
        let search = NeuralSearch::new();
        let results = search.search_by_function(
            "process",
            &vec![("lib.rs".to_string(), "fn process() {}".to_string())],
        );
        assert!(!results.is_empty());
    }
}
