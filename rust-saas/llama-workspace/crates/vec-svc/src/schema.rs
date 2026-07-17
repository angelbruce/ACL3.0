//! Vec-svc 数据库表结构 定义
//!
//!  diesel table! 宏定义表结构

use diesel::table;
use diesel::joinable;
use diesel::allow_tables_to_appear_in_same_query;

table! {
    documents (id) {
        id -> Int8,
        project_id -> Nullable<Int8>,
        title -> Nullable<Text>,
        topic -> Nullable<Text>,
        content -> Nullable<Text>,
        content_hash -> Nullable<Text>,
        source_type -> Nullable<Text>,
        source_url -> Nullable<Text>,
        file_path -> Nullable<Text>,
        file_type -> Nullable<Text>,
        status -> Nullable<Text>,
        visibility -> Nullable<Text>,
        boundary_level -> Nullable<Int4>,
        token_count -> Nullable<Int4>,
        version -> Int4,
        word_count -> Nullable<Int4>,
        chunk_count -> Int4,
        metadata -> Nullable<Jsonb>,
        created_at -> Timestamp,
        updated_at -> Timestamp,
        indexed_at -> Nullable<Timestamp>,
    }
}

table! {
    document_chunks (id) {
        id -> Int8,
        document_id -> Int8,
        chunk_index -> Int4,
        chunk_text -> Nullable<Text>,
        embedding_status -> Nullable<Text>,
        created_at -> Timestamp,
    }
}

table! {
    project_rag_configs (id) {
        id -> Int8,
        project_id -> Nullable<Int8>,
        chunk_size -> Int4,
        chunk_overlap -> Int4,
        chunk_strategy -> Nullable<Text>,
        min_chunk_size -> Int4,
        top_k -> Int4,
        min_score -> Float8,
        rerank -> Bool,
        rerank_top_k -> Int4,
        search_type -> Nullable<Text>,
        temperature -> Float8,
        max_tokens -> Int4,
        context_window -> Int4,
        batch_size -> Int4,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

table! {
    knowledge_points (id) {
        id -> Int8,
        document_id -> Int8,
        point_type -> Nullable<Text>,
        point_content -> Nullable<Text>,
        confidence -> Nullable<Float8>,
        keywords -> Nullable<Jsonb>,
        created_at -> Timestamp,
    }
}

table! {
    knowledge_entities (id) {
        id -> Int8,
        project_id -> Nullable<Int8>,
        name -> Nullable<Text>,
        entity_type -> Nullable<Text>,
        description -> Nullable<Text>,
        aliases -> Nullable<Jsonb>,
        confidence -> Nullable<Float8>,
        source_document_id -> Nullable<Int8>,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

table! {
    knowledge_relations (id) {
        id -> Int8,
        project_id -> Nullable<Int8>,
        source_entity_id -> Int8,
        target_entity_id -> Int8,
        relation_type -> Nullable<Text>,
        relation_strength -> Nullable<Float8>,
        evidence_text -> Nullable<Text>,
        source_document_id -> Nullable<Int8>,
        confidence -> Nullable<Float8>,
        created_at -> Timestamp,
    }
}

table! {
    verification_conflicts (id) {
        id -> Int8,
        project_id -> Nullable<Int8>,
        query_text -> Nullable<Text>,
        llm_summary -> Nullable<Text>,
        conflict_type -> Nullable<Text>,
        conflict_description -> Nullable<Text>,
        confidence_score -> Nullable<Float8>,
        resolved -> Bool,
        resolution -> Nullable<Text>,
        resolved_at -> Nullable<Timestamp>,
        created_at -> Timestamp,
    }
}

table! {
    document_categories (id) {
        id -> Int8,
        project_id -> Nullable<Int8>,
        category_name -> Nullable<Text>,
        category_type -> Nullable<Text>,
        parent_id -> Nullable<Int8>,
        level -> Int4,
        description -> Nullable<Text>,
        icon -> Nullable<Text>,
        color -> Nullable<Text>,
        sort_order -> Int4,
        is_active -> Bool,
        created_at -> Timestamp,
    }
}

table! {
    document_levels (id) {
        id -> Int8,
        project_id -> Nullable<Int8>,
        level_name -> Nullable<Text>,
        level_type -> Nullable<Text>,
        level_value -> Int4,
        description -> Nullable<Text>,
        icon -> Nullable<Text>,
        color -> Nullable<Text>,
        created_at -> Timestamp,
    }
}

table! {
    document_category_mappings (id) {
        id -> Int8,
        document_id -> Int8,
        category_id -> Int8,
        confidence -> Nullable<Float8>,
        is_primary -> Bool,
        created_at -> Timestamp,
    }
}

table! {
    document_level_mappings (id) {
        id -> Int8,
        document_id -> Int8,
        level_id -> Int8,
        confidence -> Nullable<Float8>,
        is_primary -> Bool,
        created_at -> Timestamp,
    }
}

table! {
    document_boundaries (id) {
        id -> Int8,
        document_id -> Int8,
        boundary_type -> Nullable<Text>,
        owner_id -> Nullable<Int8>,
        project_id -> Nullable<Int8>,
        team_id -> Nullable<Int8>,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

table! {
    document_shares (id) {
        id -> Int8,
        document_id -> Int8,
        share_type -> Nullable<Text>,
        target_type -> Nullable<Text>,
        target_id -> Nullable<Int8>,
        granted_by -> Nullable<Int8>,
        expire_at -> Nullable<Timestamp>,
        created_at -> Timestamp,
    }
}

table! {
    tasks (id) {
        id -> Int8,
        task_type -> Text,
        status -> Text,
        payload -> Jsonb,
        progress -> Float4,
        message -> Nullable<Text>,
        created_at -> Timestamp,
        started_at -> Nullable<Timestamp>,
        completed_at -> Nullable<Timestamp>,
    }
}

table! {
    document_versions (id) {
        id -> Int8,
        document_id -> Int8,
        version_number -> Int4,
        content -> Text,
        title -> Nullable<Text>,
        change_note -> Nullable<Text>,
        created_by -> Nullable<Int8>,
        created_at -> Timestamp,
    }
}

table! {
    access_logs (id) {
        id -> Int8,
        document_id -> Nullable<Int8>,
        user_id -> Nullable<Int8>,
        access_type -> Varchar,
        ip_address -> Nullable<Varchar>,
        user_agent -> Nullable<Text>,
        created_at -> Timestamp,
    }
}

table! {
    search_logs (id) {
        id -> Int8,
        user_id -> Nullable<Int8>,
        project_id -> Nullable<Int8>,
        query_text -> Text,
        result_count -> Int4,
        response_time_ms -> Nullable<Int4>,
        ip_address -> Nullable<Varchar>,
        created_at -> Timestamp,
    }
}

joinable!(document_chunks -> documents (document_id));
joinable!(knowledge_points -> documents (document_id));
joinable!(knowledge_entities -> documents (source_document_id));
joinable!(knowledge_relations -> knowledge_entities (source_entity_id));
joinable!(knowledge_relations -> documents (source_document_id));
joinable!(document_category_mappings -> documents (document_id));
joinable!(document_category_mappings -> document_categories (category_id));
joinable!(document_level_mappings -> documents (document_id));
joinable!(document_level_mappings -> document_levels (level_id));
joinable!(document_boundaries -> documents (document_id));
joinable!(document_shares -> documents (document_id));

allow_tables_to_appear_in_same_query!(
    documents,
    document_chunks,
    project_rag_configs,
    knowledge_points,
    knowledge_entities,
    knowledge_relations,
    verification_conflicts,
    document_categories,
    document_levels,
    document_category_mappings,
    document_level_mappings,
    document_boundaries,
    document_shares,
    tasks,
    document_versions,
    access_logs,
    search_logs,
);
