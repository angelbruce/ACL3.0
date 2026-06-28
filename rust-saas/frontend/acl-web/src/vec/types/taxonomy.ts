import type { BaseEntity } from './common';

export interface Category extends BaseEntity {
  category_name: string;
  category_type?: string;
  description?: string;
  parent_id?: number | null;
  level: number;
  sort_order: number;
  is_active: boolean;
  children?: Category[];
  document_count?: number;
}

export interface Level extends BaseEntity {
  level_name: string;
  level_value: number;
  level_type?: string;
  description?: string;
}
