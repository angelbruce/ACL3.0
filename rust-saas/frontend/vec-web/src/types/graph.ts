import type { BaseEntity } from './common';

export interface Entity extends BaseEntity {
  name: string;
  entity_type: string;
  description?: string;
  count: number;
}

export interface Relation extends BaseEntity {
  source_id: string;
  target_id: string;
  relation_type: string;
  weight: number;
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
