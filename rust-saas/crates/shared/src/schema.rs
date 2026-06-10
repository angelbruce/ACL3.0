use diesel::table;
use diesel::joinable;
use diesel::allow_tables_to_appear_in_same_query;

table! {
    agents (id) {
        id -> Int8,
        name -> Text,
        defination -> Nullable<Text>,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

table! {
    agent_tools (id) {
        id -> Int8,
        agent_id -> Int8,
        name -> Text,
        description -> Text,
        input_schema -> Text,
        output_schema -> Text,
        server_id -> Nullable<Int8>,
    }
}

table! {
    agent_skills (id) {
        id -> Int8,
        agent_id -> Int8,
        skill_prompt -> Text,
    }
}

table! {
    content_store_configs (id) {
        id -> Int8,
        agent_id -> Int8,
        store_type -> Text,
        config -> Text,
    }
}

table! {
    flows (id) {
        id -> Int8,
        name -> Text,
        config -> Jsonb,
        created_at -> Timestamp,
    }
}

table! {
    flow_runtimes (id) {
        id -> Int8,
        flow_id -> Int8,
        is_over -> Bool,
        created_at -> Timestamp,
    }
}

table! {
    flow_runtime_nodes (id) {
        id -> Int8,
        flow_runtime_id -> Int8,
        flow_id -> Int8,
        action_id -> Int8,
        action -> Text,
        prompt -> Nullable<Text>,
        status -> Text,
        next_choice -> Nullable<Text>,
        created_at -> Timestamp,
    }
}

table! {
    sessions (id) {
        id -> Int8,
        user_id -> Int8,
        description -> Nullable<Text>,
        agent_id -> Nullable<Int8>,
        model_id -> Nullable<Int8>,
        created_at -> Timestamp,
    }
}

table! {
    session_items (id) {
        id -> Int8,
        session_id -> Int8,
        description -> Text,
        session_type -> Text,
        created_at -> Timestamp,
    }
}

table! {
    llm_models (id) {
        id -> Int8,
        name -> Text,
        access_url -> Text,
        api_key -> Text,
        is_default -> Bool,
    }
}

table! {
    users (id) {
        id -> Int8,
        email -> Text,
        password_hash -> Text,
        created_at -> Timestamp,
    }
}

table! {
    mcp_servers (id) {
        id -> Int8,
        name -> Text,
        description -> Nullable<Text>,
        server_type -> Text,
        url -> Text,
        headers -> Nullable<Jsonb>,
        enabled -> Bool,
        stateless -> Bool,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

table! {
    departments (id) {
        id -> Int8,
        name -> Text,
        parent_id -> Nullable<Int8>,
        description -> Nullable<Text>,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

table! {
    personnel (id) {
        id -> Int8,
        user_id -> Nullable<Int8>,
        name -> Text,
        gender -> Nullable<Text>,
        email -> Nullable<Text>,
        wechat -> Nullable<Text>,
        phone -> Nullable<Text>,
        last_login_date -> Nullable<Timestamp>,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

table! {
    personnel_departments (id) {
        id -> Int8,
        personnel_id -> Int8,
        department_id -> Int8,
        created_at -> Timestamp,
    }
}

table! {
    menus (id) {
        id -> Int8,
        name -> Text,
        path -> Nullable<Text>,
        parent_id -> Nullable<Int8>,
        icon -> Nullable<Text>,
        sort_order -> Int4,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

table! {
    permissions (id) {
        id -> Int8,
        menu_id -> Int8,
        name -> Text,
        description -> Nullable<Text>,
        created_at -> Timestamp,
    }
}

table! {
    roles (id) {
        id -> Int8,
        name -> Text,
        description -> Nullable<Text>,
        is_super_admin -> Bool,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

table! {
    role_permissions (id) {
        id -> Int8,
        role_id -> Int8,
        permission_id -> Int8,
        created_at -> Timestamp,
    }
}

table! {
    personnel_roles (id) {
        id -> Int8,
        personnel_id -> Int8,
        role_id -> Int8,
        created_at -> Timestamp,
    }
}

table! {
    workspace_files (id) {
        id -> Int8,
        user_id -> Int8,
        file_path -> Text,
        file_name -> Text,
        file_size -> Int8,
        is_directory -> Bool,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

table! {
    kanban_boards (id) {
        id -> Int8,
        name -> Text,
        description -> Nullable<Text>,
        is_public -> Bool,
        created_by -> Int8,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

table! {
    kanban_items (id) {
        id -> Int8,
        board_id -> Int8,
        user_id -> Int8,
        file_path -> Text,
        file_name -> Text,
        shared_at -> Timestamp,
    }
}

table! {
    kanban_subscriptions (id) {
        id -> Int8,
        board_id -> Int8,
        user_id -> Int8,
        subscribed_at -> Timestamp,
    }
}

table! {
    projects (id) {
        id -> Int8,
        user_id -> Int8,
        name -> Text,
        purpose -> Text,
        description -> Nullable<Text>,
        model_id -> Nullable<Int8>,
        agent_id -> Nullable<Int8>,
        last_accessed_at -> Timestamp,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

table! {
    project_files (id) {
        id -> Int8,
        project_id -> Int8,
        name -> Text,
        content -> Nullable<Text>,
        created_at -> Timestamp,
        updated_at -> Timestamp,
        directory -> Nullable<Text>,
        state -> Int4,
    }
}

table! {
    project_messages (id) {
        id -> Int8,
        project_id -> Int8,
        role -> Text,
        content -> Text,
        created_at -> Timestamp,
    }
}


table!{
    project_container_configs (id) {
        id -> Int8,
        project_id -> Int8,
        project_dir -> Text,
        published_ports -> Text,
        volumes -> Text,
        environment -> Text,
        command -> Text,
        working_dir -> Text,
        tags -> Text,
        container_name -> Text,
        cpu_usage -> Text,
        memory_usage -> Text,
        image_name -> Text,
        creator_id -> Int8,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
   

}
table! {
    project_summaries (id) {
        id -> Int8,
        user_id -> Int8,
        project_id -> Int8,
        file_name -> Text,
        summary -> Text,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

joinable!(agent_skills -> agents (agent_id));
joinable!(agent_tools -> agents (agent_id));
joinable!(content_store_configs -> agents (agent_id));
joinable!(flow_runtime_nodes -> flow_runtimes (flow_runtime_id));
joinable!(flow_runtimes -> flows (flow_id));
joinable!(session_items -> sessions (session_id));
joinable!(personnel_departments -> departments (department_id));
joinable!(personnel_departments -> personnel (personnel_id));
joinable!(role_permissions -> permissions (permission_id));
joinable!(role_permissions -> roles (role_id));
joinable!(personnel_roles -> personnel (personnel_id));
joinable!(personnel_roles -> roles (role_id));
joinable!(permissions -> menus (menu_id));
joinable!(project_files -> projects (project_id));
joinable!(project_messages -> projects (project_id));
joinable!(project_summaries -> projects (project_id));

allow_tables_to_appear_in_same_query!(
    agents,
    agent_tools,
    agent_skills,
    content_store_configs,
    flows,
    flow_runtimes,
    flow_runtime_nodes,
    sessions,
    session_items,
    llm_models,
    users,
    mcp_servers,
    departments,
    personnel,
    personnel_departments,
    menus,
    permissions,
    roles,
    role_permissions,
    personnel_roles,
    workspace_files,
    kanban_boards,
    kanban_items,
    kanban_subscriptions,
    projects,
    project_files,
    project_messages,
    project_summaries,
);
