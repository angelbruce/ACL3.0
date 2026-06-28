import type { BaseEntity, Status } from './common';

export interface Version extends BaseEntity {
  document_id: number;
  version_number: number;
  change_note?: string;
  created_by?: number;
  created_at: string;
}

export interface VersionDiff {
  version_a_id: number;
  version_b_id: number;
  version_a_number: number;
  version_b_number: number;
  diffs: Array<{
    diff_type: 'Added' | 'Removed' | 'Unchanged';
    content: string;
    old_line_start: number;
    new_line_start: number;
    line_count: number;
  }>;
}
