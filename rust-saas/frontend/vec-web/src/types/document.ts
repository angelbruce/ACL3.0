import type { BaseEntity, DocumentType, Status, Visibility } from './common';

export interface Document extends BaseEntity {
  title: string;
  content: string;
  document_type: DocumentType;
  file_name?: string;
  file_size?: number;
  status: Status;
  visibility: Visibility;
  category_id?: string;
  level?: number;
  version: number;
  embedding_status: Status;
  word_count: number;
  chunk_count: number;
}

export interface DocumentCreateRequest {
  title: string;
  content?: string;
  file?: File;
  visibility?: Visibility;
  category_id?: string;
}

export interface DocumentUpdateRequest {
  title?: string;
  content?: string;
  visibility?: Visibility;
  category_id?: string;
}

export interface Chunk {
  id: string;
  document_id: string;
  content: string;
  index: number;
  embedding?: number[];
  created_at: string;
}

export interface KnowledgePoint {
  id: string;
  document_id: string;
  content: string;
  confidence: number;
  keywords: string[];
  created_at: string;
}
