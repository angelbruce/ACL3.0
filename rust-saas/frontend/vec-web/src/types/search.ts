export interface SearchResult {
  id: string;
  document_id: string;
  document_title: string;
  content: string;
  chunk_id: string;
  score: number;
  rerank_score?: number;
  highlighted_content: string;
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
