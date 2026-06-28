export interface PopularDocument {
  document_id: number;
  title: string;
  view_count: number;
}

export interface HotEntity {
  entity_id: number;
  name: string;
  entity_type: string;
  mention_count: number;
}

export interface SearchTrend {
  query: string;
  count: number;
  trend: number;
}

export interface AnalyticsSummary {
  total_documents: number;
  total_views: number;
  total_searches: number;
  total_entities: number;
  active_users: number;
  popular_documents: PopularDocument[];
  hot_entities: HotEntity[];
  search_trends: SearchTrend[];
}

export interface DocumentAnalytics {
  document_id: number;
  view_count: number;
  search_count: number;
  share_count: number;
  average_read_time: number;
}
