
use crate::app_state::EnhancedSearchResult;

pub struct RerankService {
    bm25_weight: f32,
    vector_weight: f32,
    keyword_boost: f32,
}

impl RerankService {
    pub fn new() -> Self {
        Self {
            bm25_weight: 0.3,
            vector_weight: 0.7,
            keyword_boost: 1.2,
        }
    }

    pub fn with_weights(bm25_weight: f32, vector_weight: f32) -> Self {
        Self {
            bm25_weight,
            vector_weight,
            keyword_boost: 1.2,
        }
    }

    pub fn rerank(
        &self,
        query: &str,
        results: Vec<EnhancedSearchResult>,
    ) -> Vec<EnhancedSearchResult> {
        let query_terms: Vec<&str> = query
            .split_whitespace()
            .filter(|t| !t.is_empty())
            .collect();

        if query_terms.is_empty() || results.is_empty() {
            return results;
        }

        let mut scored: Vec<(f32, EnhancedSearchResult)> = results
            .into_iter()
            .map(|result| {
                let bm25_score = self.calculate_bm25(&query_terms, &result.content);
                let vector_score = result.score;
                let final_score = self.vector_weight * vector_score
                    + self.bm25_weight * bm25_score;

                (final_score, result)
            })
            .collect();

        scored.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));

        scored
            .into_iter()
            .map(|(score, mut result)| {
                result.score = score;
                result
            })
            .collect()
    }

    fn calculate_bm25(&self, query_terms: &[&str], document: &str) -> f32 {
        let doc_lower = document.to_lowercase();
        let doc_len = doc_lower.len() as f32;
        let avg_doc_len = 500.0;
        let k1 = 1.5;
        let b = 0.75;

        let mut score = 0.0;

        for term in query_terms {
            let term_lower = term.to_lowercase();
            let tf = self.term_frequency(&term_lower, &doc_lower) as f32;

            if tf == 0.0 {
                continue;
            }

            let idf = self.inverse_document_frequency(term_lower.len() as f32);

            let numerator = tf * (k1 + 1.0);
            let denominator = tf + k1 * (1.0 - b + b * doc_len / avg_doc_len);

            score += idf * (numerator / denominator);
        }

        score.min(1.0)
    }

    fn term_frequency(&self, term: &str, document: &str) -> usize {
        document.matches(term).count()
    }

    fn inverse_document_frequency(&self, term_len: f32) -> f32 {
        let base = 10000.0;
        let adjusted = (base / (term_len * 10.0 + 1.0)).ln();
        adjusted.max(0.1).min(2.0)
    }

    pub fn hybrid_search_boost(
        &self,
        vector_results: Vec<EnhancedSearchResult>,
        keyword_results: Vec<EnhancedSearchResult>,
    ) -> Vec<EnhancedSearchResult> {
        use std::collections::HashMap;

        let mut combined: HashMap<i64, (f32, EnhancedSearchResult)> = HashMap::new();

        for (i, result) in vector_results.into_iter().enumerate() {
            let rank_score = 1.0 / (i as f32 + 1.0);
            let score = self.vector_weight * result.score + 0.1 * rank_score;
            combined.insert(result.id, (score, result));
        }

        for (i, result) in keyword_results.into_iter().enumerate() {
            let rank_score = 1.0 / (i as f32 + 1.0);
            let boost = self.bm25_weight * result.score + 0.1 * rank_score;

            if let Some(existing) = combined.get_mut(&result.id) {
                existing.0 += boost * self.keyword_boost;
            } else {
                combined.insert(result.id, (boost * 0.5, result));
            }
        }

        let mut results: Vec<(f32, EnhancedSearchResult)> = combined.into_values().collect();
        results.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));

        results
            .into_iter()
            .map(|(score, mut result)| {
                result.score = score.min(1.0);
                result
            })
            .collect()
    }

    pub fn diversify_results(
        &self,
        results: Vec<EnhancedSearchResult>,
        diversity_window: usize,
    ) -> Vec<EnhancedSearchResult> {
        if results.len() <= diversity_window || diversity_window <= 1 {
            return results;
        }

        let mut diversified = Vec::with_capacity(results.len());
        let mut seen_docs = std::collections::HashSet::new();

        for result in &results {
            if let Some(doc_id) = result.document_id {
                if seen_docs.contains(&doc_id) {
                    continue;
                }
                seen_docs.insert(doc_id);
            }
            diversified.push(result.clone());

            if diversified.len() >= diversity_window {
                break;
            }
        }

        for result in results {
            if !diversified.iter().any(|r| r.id == result.id) {
                diversified.push(result);
            }
        }

        diversified
    }
}

impl Default for RerankService {
    fn default() -> Self {
        Self::new()
    }
}
