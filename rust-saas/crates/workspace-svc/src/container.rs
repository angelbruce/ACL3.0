use std::collections::HashMap;


pub struct ProjectContainerConfig {
    pub project_id: i64,
    pub container_id: String,
    pub image: String,
    pub command: String,
    pub ports: Vec<i16>,
    pub volumes: Vec<String>,
    pub env: Vec<String>,
    pub networks: Vec<String>,
    pub labels: Vec<String>,
    pub restart: String,
    pub privileged: bool,
    pub readonly: bool,
    pub user: String,
    pub group: String,
    pub working_dir: String,
    pub stdin_open: bool,
    pub tty: bool,
    pub detach: bool,
}


pub struct ProjectContainerStatus {
    pub project_id: i64,
    pub container_id: String,
    pub status: String,
    pub message: String,
}



pub struct ProjectContainerService {
    pub repo: WorkspaceRepository,
    pub container_service: ContainerService,
    pub container_status_repo: ContainerStatusRepository,
    pub container_status_service: ContainerStatusService,
    pub container_status_repo: ContainerStatusRepository,
    
}