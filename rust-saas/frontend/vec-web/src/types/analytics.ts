export interface AnalyticsSummary {
  total_documents: number;
  total_chunks: number;
  total_entities: number;
  total_relations: number;
  total_search_queries: number;
  total_access_count: number;
  top_documents: TopDocument[];
  top_searches: TopSearch[];
}

export interface TopDocument {
  document_id: string;
  title: string;
  access_count: number;
}

export interface TopSearch {
  query: string;
  count: number;
}

export interface DocumentAnalytics {
  document_id: string;
  title: string;
  total_access: number;
  search_count: number;
  access_trend: DailyStat[];
}

export interface DailyStat {
  date: string;
  count: number;
}
