import type { BaseEntity, Visibility } from './common';

export interface Share extends BaseEntity {
  document_id: number;
  share_type?: string;
  target_type?: string;
  target_id?: number;
  granted_by?: number;
  expire_at?: string;
}

export interface VisibilityUpdate {
  visibility: Visibility;
  owner_id?: number;
  project_id?: number;
  team_id?: number;
}
