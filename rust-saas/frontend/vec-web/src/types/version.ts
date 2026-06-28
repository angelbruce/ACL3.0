import type { BaseEntity, Status } from './common';

export interface Version extends BaseEntity {
  document_id: string;
  version_number: number;
  title: string;
  content: string;
  status: Status;
  created_by?: string;
  change_summary?: string;
}

export interface VersionDiff {
  version_id: string;
  compared_version_id: string;
  added: string[];
  removed: string[];
  modified: string[];
}
