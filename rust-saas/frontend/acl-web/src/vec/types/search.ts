export interface SearchResult {
  id: number;
  document_id?: number;
  document_topic?: string;
  document_title?: string;
  content: string;
  chunk_id?: number;
  chunk_index?: number;
  score: number;
  rerank_score?: number;
  highlighted_content?: string;
  created_at?: string;
}

export interface SearchResponse {
  results: SearchResult[];
  query: string;
  total: number;
}

export interface SearchRequest {
  query: string;
  limit?: number;
  offset?: number;
  category_id?: string;
  min_level?: number;
  max_level?: number;
}

export interface SearchSuggestion {
  text: string;
  weight: number;
}

export interface AutocompleteItem {
  text: string;
  type: 'keyword' | 'document' | 'entity';
}
