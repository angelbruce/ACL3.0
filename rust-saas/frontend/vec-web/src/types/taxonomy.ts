import type { BaseEntity } from './common';

export interface Category extends BaseEntity {
  name: string;
  description?: string;
  parent_id?: string;
  children?: Category[];
  document_count: number;
}

export interface Level extends BaseEntity {
  name: string;
  value: number;
  description?: string;
}
