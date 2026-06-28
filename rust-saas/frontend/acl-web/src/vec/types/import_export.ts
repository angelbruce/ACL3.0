export interface ImportError {
  index: number;
  title: string;
  error: string;
}

export interface ImportResult {
  success_count: number;
  failed_count: number;
  total_count: number;
  errors: ImportError[];
  document_ids?: number[];
}

export interface ExportEntity {
  id: number;
  name: string;
  entity_type: string;
  description?: string;
  document_id?: number;
}

export interface ExportRelation {
  id: number;
  source_id: number;
  target_id: number;
  relation_type: string;
  document_id?: number;
}

export interface KnowledgeGraphExport {
  entities: ExportEntity[];
  relations: ExportRelation[];
  entity_count: number;
  relation_count: number;
}
