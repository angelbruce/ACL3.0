import type { BaseEntity } from './common';

export interface Entity extends BaseEntity {
  name?: string;
  entity_type?: string;
  description?: string;
  source_document_id?: number;
  count?: number;
}

export interface Relation {
  id: number;
  source_entity_id: number;
  target_entity_id: number;
  relation_type?: string;
  relation_strength?: number;
  evidence_text?: string;
  source_document_id?: number;
  confidence?: number;
}

export interface GraphData {
  entities: Entity[];
  relations: Relation[];
}

export interface GraphNode {
  id: string;
  name: string;
  type: string;
  symbolSize?: number;
}

export interface GraphEdge {
  source: string;
  target: string;
  relation: string;
}
