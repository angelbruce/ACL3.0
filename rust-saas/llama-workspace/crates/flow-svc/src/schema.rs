
use diesel::table;
use diesel::joinable;
use diesel::allow_tables_to_appear_in_same_query;

table! {
    flow_runtime_sessions (id) {
        id -> Int8,
        flow_id -> Int8,
        flow_runtime_id -> Text,
        creator_id -> Int8,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}


table! {
    flow_runtime_session_items (id) {
        id -> Int8,
        flow_id -> Int8,
        flow_runtime_id -> Text,
        flow_runtime_session_id -> Int8,
        flow_runtime_node_id -> Text,
        session_type -> Text,
        content -> Text,
        action_id -> Int8,
        created_at -> Timestamp,
        creator_id -> Int8,
    }
}