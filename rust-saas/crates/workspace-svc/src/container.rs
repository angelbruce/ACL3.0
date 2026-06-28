use std::collections::HashMap;
use std::fs::{self, File};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::thread::sleep;
use shared::models::ProjectContainerConfig;
use serde::{Serialize, Deserialize};
use std::env;
use futures::{Stream, StreamExt};
use tokio::io::AsyncBufReadExt;
use tokio_stream::wrappers::LinesStream;

use crate::llm_client::LlmClient;
use crate::llm_client::ToolExecutor;
use crate::repository::WorkspaceRepository;
use shared::models::{ChatMessage, MCPTool};

// pub const DEFAULT_MCP_SSE_PORT: &str = "80";

// fn get_mcp_sse_port(config: &ProjectContainerConfig) -> String {
//     if !config.environment.is_empty() {
//         let envs: Vec<&str> = config.environment.split(',').collect();
//         for env in envs {
//             let trimmed = env.trim();
//             if trimmed.starts_with("MCP_SSE_PORT=") {
//                 return trimmed.split('=').nth(1).unwrap_or(DEFAULT_MCP_SSE_PORT).to_string();
//             }
//         }
//     }
//     DEFAULT_MCP_SSE_PORT.to_string()
// }

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DockerComposeService {
    pub image: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub build: Option<BuildConfig>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub ports: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub volumes: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub environment: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub command: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub working_dir: Option<String>,
    #[serde(skip_serializing_if = "String::is_empty")]
    pub restart: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub privileged: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stdin_open: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tty: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub networks: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub container_name: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BuildConfig {
    pub context: String,
    pub dockerfile: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DockerComposeNetwork {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub driver: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub external: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DockerComposeVolume {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub driver: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DockerComposeConfig {
    pub version: String,
    #[serde(rename = "services")]
    pub services: HashMap<String, DockerComposeService>,
    #[serde(skip_serializing_if = "HashMap::is_empty")]
    pub networks: HashMap<String, DockerComposeNetwork>,
    #[serde(skip_serializing_if = "HashMap::is_empty")]
    pub volumes: HashMap<String, DockerComposeVolume>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ProjectDeploymentContext {
    pub user_id: i64,
    pub project_id: i64,
    pub project_name: String,
    pub agent_id: Option<i64>,
    pub model_id: Option<i64>,
    pub container_configs: Vec<ProjectContainerConfig>,
    pub project_files: Vec<ProjectFileInfo>,
    pub container_name: String,
    pub mcp_server_url: String,
    pub debug_dir: PathBuf,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ProjectFileInfo {
    pub id: i64,
    pub name: String,
    pub directory: String,
    pub content: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ContainerDeploymentResult {
    pub success: bool,
    pub message: String,
    pub debug_dir: String,
    pub container_names: Vec<String>,
    pub docker_compose_path: String,
}

pub fn get_debug_directory(user_id: i64, project_id: i64) -> PathBuf {
    PathBuf::from(format!("/debug/{}/{}", user_id, project_id))
}

pub fn get_container_volume_directory(user_id: i64, project_id: i64, container_name: &str) -> PathBuf {
    get_debug_directory(user_id, project_id).join("volumes").join(container_name)
}

/// 生成容器名称: {user_id}-{project_id}-{config_id}
pub fn format_container_name(user_id: i64, project_id: i64, config_id: i64) -> String {
    format!("{}-{}-{}", user_id, project_id, config_id)
}

pub fn format_volume_name(user_id: i64, project_id: i64, name: &str) -> String {
    let sanitized: String = name.chars()
        .map(|c| if c.is_alphanumeric() || c == '.' || c == '_' || c == '-' { c } else { '_' })
        .collect();
    format!("vol_{}_{}_{}", user_id, project_id, sanitized)
}

pub fn ensure_debug_directory(user_id: i64, project_id: i64) -> Result<PathBuf, String> {
    let debug_dir = get_debug_directory(user_id, project_id);
    match fs::create_dir_all(&debug_dir) {
        Ok(_) => Ok(debug_dir),
        Err(e) => Err(format!("Failed to create debug directory: {}", e)),
    }
}

pub fn ensure_container_volume_directory(user_id: i64, project_id: i64, container_name: &str) -> Result<PathBuf, String> {
    let vol_dir = get_container_volume_directory(user_id, project_id, container_name);
    match fs::create_dir_all(&vol_dir) {
        Ok(_) => Ok(vol_dir),
        Err(e) => Err(format!("Failed to create volume directory for container {}: {}", container_name, e)),
    }
}

pub fn write_project_files_to_volume(
    user_id: i64,
    project_id: i64,
    container_name: &str,
    files: &[ProjectFileInfo],
) -> Result<(), String> {
    let vol_dir = get_container_volume_directory(user_id, project_id, container_name);
    
    for file in files {
        let file_path = if file.directory.is_empty() {
            vol_dir.join(&file.name)
        } else {
            let dir_path = vol_dir.join(&file.directory);
            if let Err(e) = fs::create_dir_all(&dir_path) {
                return Err(format!("Failed to create directory {}: {}", dir_path.display(), e));
            }
            dir_path.join(&file.name)
        };
        
        if let Some(content) = &file.content {
            if let Err(e) = fs::write(&file_path, content) {
                return Err(format!("Failed to write file {}: {}", file_path.display(), e));
            }
            println!("[DEBUG] Wrote file: {}", file_path.display());
        }
    }
    
    Ok(())
}

pub fn write_file_to_volume(
    vol_dir: &Path,
    file: &ProjectFileInfo,
) -> Result<String, String> {
    let file_path = if file.directory.is_empty() {
        vol_dir.join(&file.name)
    } else {
        let dir_path = vol_dir.join(&file.directory);
        match fs::create_dir_all(&dir_path) {
            Ok(_) => dir_path.join(&file.name),
            Err(e) => return Err(format!("Failed to create directory {}: {}", file.directory, e)),
        }
    };

    if let Some(content) = &file.content {
        match File::create(&file_path) {
            Ok(mut f) => match f.write_all(content.as_bytes()) {
                Ok(_) => Ok(file_path.to_string_lossy().to_string()),
                Err(e) => return Err(format!("Failed to write file {}: {}", file_path.display(), e)),
            },
            Err(e) => return Err(format!("Failed to create file {}: {}", file_path.display(), e)),
        }
    } else {
        Ok("File has no content".to_string())
    }
}

/// 刷新单个项目文件到容器卷
pub async fn refresh_container_file(
    user_id: i64,
    project_id: i64,
    config_id: i64,
    file_id: i64,
    content: &str,
) -> Result<String, String> {
    let repo = WorkspaceRepository::new();
    
    // 获取项目文件信息
    let file = repo.get_project_file_by_id(file_id, user_id).await
        .map_err(|e| format!("Failed to get file: {}", e))?;
    
    let file_info = ProjectFileInfo {
        id: file.id,
        name: file.name,
        directory: file.directory.unwrap_or_default(),
        content: Some(content.to_string()),
    };
    
    let container_name = format_container_name(user_id, project_id, config_id);
    let vol_dir = get_container_volume_directory(user_id, project_id, &container_name);
    
    // 确保卷目录存在
    ensure_container_volume_directory(user_id, project_id, &container_name)?;
    
    // 写入文件
    let file_path = write_file_to_volume(&vol_dir, &file_info)?;
    
    Ok(format!("File {} refreshed at {}", file_info.name, file_path))
}

/// ContainerDeployer 的刷新文件方法
impl ContainerDeployer {
    /// 刷新单个文件到容器并执行命令
    pub async fn refresh_file_and_execute(
        user_id: i64,
        project_id: i64,
        config_id: i64,
        file_id: i64,
        content: &str,
        command: &str,
    ) -> Result<String, String> {
        // 1. 刷新文件到卷
        refresh_container_file(user_id, project_id, config_id, file_id, content).await?;
        
        // 2. 检查容器是否运行
        let container_name = format_container_name(user_id, project_id, config_id);
        let output = std::process::Command::new("docker")
            .args(&["inspect", "-f", "{{.State.Running}}", &container_name])
            .output()
            .map_err(|e| format!("Failed to inspect container: {}", e))?;
        
        let is_running = String::from_utf8_lossy(&output.stdout).trim() == "true";
        
        if !is_running {
            return Err("Container is not running".to_string());
        }
        
        // 3. 获取文件在容器中的路径
        let repo = WorkspaceRepository::new();
        let file = repo.get_project_file_by_id(file_id, user_id).await
            .map_err(|e| format!("Failed to get file: {}", e))?;
        
        let working_dir = "/app";
        let container_file_path = if file.directory.is_some() && !file.directory.as_ref().unwrap().is_empty() {
            format!("{}/{}", working_dir, file.directory.as_ref().unwrap())
        } else {
            working_dir.to_string()
        };
        
        // 4. 使用 docker cp 将文件复制到容器
        let local_file_path = if file.directory.is_some() && !file.directory.as_ref().unwrap().is_empty() {
            let vol_dir = get_container_volume_directory(user_id, project_id, &container_name);
            vol_dir.join(file.directory.as_ref().unwrap()).join(&file.name)
        } else {
            let vol_dir = get_container_volume_directory(user_id, project_id, &container_name);
            vol_dir.join(&file.name)
        };
        
        let dest_path = format!("{}:{}", &container_name, &container_file_path);
        let cp_result = std::process::Command::new("docker")
            .args(&["cp", &local_file_path.to_string_lossy(), &dest_path])
            .output()
            .map_err(|e| format!("Failed to copy file to container: {}", e))?;
        
        if !cp_result.status.success() {
            let stderr = String::from_utf8_lossy(&cp_result.stderr);
            return Err(format!("Failed to copy file: {}", stderr));
        }
        
        // 5. 执行命令
        let exec_result = Self::execute_command_stream(
            user_id,
            project_id,
            config_id,
            command,
        ).await?;
        
        // 收集命令输出
        let mut output = String::new();
        use futures::StreamExt;
        let mut stream = exec_result;
        while let Some(result) = stream.next().await {
            match result {
                Ok(line) => output.push_str(&line),
                Err(e) => output.push_str(&format!("Error: {}", e)),
            }
        }
        
        Ok(output)
    }
}

pub fn generate_dockerfile_content(config: &ProjectContainerConfig) -> String {
    let mut dockerfile = String::new();
    let base_image = "app-debug-base:latest".to_string();
    dockerfile.push_str(&format!("FROM {}\n\n", base_image));

    if !config.working_dir.is_empty() {
        dockerfile.push_str(&format!("WORKDIR {}\n\n", config.working_dir));
    }

    if !config.environment.is_empty() {
        dockerfile.push_str("# Environment variables\n");
        let envs: Vec<&str> = config.environment.split(',').collect();
        for env in envs {
            let trimmed = env.trim();
            if !trimmed.is_empty() {
                dockerfile.push_str(&format!("ENV {}\n", trimmed));
            }
        }
        dockerfile.push('\n');
    }

    dockerfile
}

pub fn write_dockerfile(debug_dir: &Path, config_id: i64, content: &str) -> Result<String, String> {
    let dockerfile_name = format!("dockerfile-{}", config_id);
    let dockerfile_path = debug_dir.join(&dockerfile_name);

    match File::create(&dockerfile_path) {
        Ok(mut f) => match f.write_all(content.as_bytes()) {
            Ok(_) => Ok(dockerfile_name),
            Err(e) => Err(format!("Failed to write Dockerfile {}: {}", dockerfile_path.display(), e)),
        },
        Err(e) => Err(format!("Failed to create Dockerfile {}: {}", dockerfile_path.display(), e)),
    }
}

pub fn generate_docker_compose_config(
    context: &ProjectDeploymentContext,
    dockerfile_names: &HashMap<i64, String>,
) -> DockerComposeConfig {
    let mut services = HashMap::new();
    let mut networks = HashMap::new();
    let mut volumes = HashMap::new();

    // 项目内部网络
    let project_network_name = format_container_name(context.user_id, context.project_id, 0);

    networks.insert(
        project_network_name.clone(),
        DockerComposeNetwork {
            driver: Some("bridge".to_string()),
            external: None,
            name: None,
        },
    );

    // 外部网络：workspace-svc 所在的网络，让调试容器能被 workspace-svc 访问
    let external_network_name = "rust-saas_default".to_string();
    networks.insert(
        external_network_name.clone(),
        DockerComposeNetwork {
            driver: None,
            external: Some(true),
            name: Some(external_network_name.clone()),
        },
    );

    for config in &context.container_configs {
        if config.container_name == "docker-compose.yml" {
            continue;
        }

        let service_name = format!("{}", config.id);
        let container_name = format_container_name(context.user_id, context.project_id, config.id);

        let mut ports: Vec<String> = Vec::new();
        if !config.published_ports.is_empty() {
            let port_list: Vec<&str> = config.published_ports.split(',').collect();
            for port in port_list {
                let trimmed = port.trim();
                if !trimmed.is_empty() {
                    if trimmed.contains(':') {
                        let internal = trimmed.split(':').last().unwrap_or(trimmed);
                        ports.push(internal.to_string());
                    } else {
                        ports.push(trimmed.to_string());
                    }
                }
            }
        }

        // let mcp_sse_port = get_mcp_sse_port(config);
        
        // if !ports.contains(&mcp_sse_port) {
        //     ports.push(mcp_sse_port.clone());
        // }

        // volumes - 使用 bind mount（宿主机路径:容器路径）
        use std::collections::HashMap;
        let mut vol_map: HashMap<String, String> = HashMap::new();

        println!("[DEBUG] Container config: name={}, volumes='{}'", config.container_name, config.volumes);

        // 先解析配置中的卷映射
        if !config.volumes.is_empty() {
            let volume_strings: Vec<&str> = config.volumes.split(',').collect();
            for vol in volume_strings {
                let trimmed = vol.trim();
                if !trimmed.is_empty() {
                    let parts: Vec<&str> = trimmed.splitn(2, ':').collect();
                    if parts.len() == 2 {
                        let host_path = parts[0];
                        let container_path = parts[1];
                        
                        let absolute_host_path = if host_path.starts_with("./") || host_path.starts_with(".\\") {
                            let relative = &host_path[2..];
                            get_container_volume_directory(context.user_id, context.project_id, &config.container_name)
                                .join(relative)
                                .to_string_lossy()
                                .trim_end_matches('/')
                                .trim_end_matches('\\')
                                .to_string()
                        } else if host_path.starts_with("/") || host_path.starts_with("\\") {
                            host_path.trim_end_matches('/').trim_end_matches('\\').to_string()
                        } else {
                            get_container_volume_directory(context.user_id, context.project_id, &config.container_name)
                                .join(host_path)
                                .to_string_lossy()
                                .trim_end_matches('/')
                                .trim_end_matches('\\')
                                .to_string()
                        };
                        
                        let vol_entry = format!("{}:{}", absolute_host_path, container_path);
                        println!("[DEBUG] Parsed volume: host_path='{}', container_path='{}', entry='{}'", absolute_host_path, container_path, vol_entry);
                        vol_map.insert(container_path.to_string(), vol_entry);
                    }
                }
            }
        }

        // 如果配置中没有指定 /app 的映射，才添加默认代码卷映射
        let vol_dir = get_container_volume_directory(context.user_id, context.project_id, &config.container_name);
        println!("[DEBUG] vol_map before default check: {:?}", vol_map);
        if !vol_map.contains_key("/app") {
            let default_host_path = vol_dir.to_string_lossy().trim_end_matches('/').trim_end_matches('\\').to_string();
            let default_entry = format!("{}:/app", default_host_path);
            println!("[DEBUG] Adding default volume: {}", default_entry);
            vol_map.insert("/app".to_string(), default_entry);
        }

        println!("[DEBUG] Final vol_map: {:?}", vol_map);
        let vol_list: Vec<String> = vol_map.into_values().collect();
        println!("[DEBUG] Final vol_list: {:?}", vol_list);

        let mut envs: Vec<String> = Vec::new();
        if !config.environment.is_empty() {
            let env_strings: Vec<&str> = config.environment.split(',').collect();
            for env in env_strings {
                let trimmed = env.trim();
                if !trimmed.is_empty() {
                    envs.push(trimmed.to_string());
                }
            }
        }

        envs.push(format!("PROJECT_ID={}", context.project_id));
        envs.push(format!("USER_ID={}", context.user_id));

        let build = if let Some(dockerfile_name) = dockerfile_names.get(&config.id) {
            Some(BuildConfig {
                context: ".".to_string(),
                dockerfile: dockerfile_name.clone(),
            })
        } else {
            None
        };

        let networks_val = Some(vec![project_network_name.clone(), external_network_name.clone()]);
        println!("[DEBUG] Setting networks for service {}: {:?}", service_name, networks_val);
        
        let service = DockerComposeService {
            image: "app-debug-base:latest".to_string(),
            build,
            ports,
            volumes: vol_list,
            environment: envs,
            command: None,
            working_dir: if config.working_dir.is_empty() { None } else { Some(config.working_dir.clone()) },
            restart: "unless-stopped".to_string(),
            privileged: Some(true),
            stdin_open: Some(true),
            tty: Some(true),
            networks: networks_val,
            container_name: Some(container_name),
        };

        println!("[DEBUG] Service networks: {:?}", service.networks);
        services.insert(service_name, service);
    }

    DockerComposeConfig {
        version: "3.8".to_string(),
        services,
        networks,
        volumes,
    }
}

pub fn write_docker_compose(
    debug_dir: &Path,
    config: &DockerComposeConfig,
) -> Result<String, String> {
    let compose_path = debug_dir.join("docker-compose.yml");

    match serde_yaml::to_string(&config) {
        Ok(yaml_content) => {
            match File::create(&compose_path) {
                Ok(mut f) => match f.write_all(yaml_content.as_bytes()) {
                    Ok(_) => Ok(compose_path.to_string_lossy().to_string()),
                    Err(e) => Err(format!("Failed to write docker-compose.yml: {}", e)),
                },
                Err(e) => Err(format!("Failed to create docker-compose.yml: {}", e)),
            }
        }
        Err(e) => Err(format!("Failed to serialize docker-compose config: {}", e)),
    }
}

pub fn generate_llm_deployment_prompt(context: &ProjectDeploymentContext) -> String {
    let mut prompt = String::new();

    prompt.push_str("作为一个专业的项目部署助手，你会锲而不舍的解决项目部署过程中遇到的问题。请按照以下步骤完成工作区项目的 Docker 容器部署：\n\n");
    prompt.push_str("【工作区项目的 Docker 容器部署：\n\n");
    prompt.push_str("【项目信息】\n");
    prompt.push_str(&format!("用户ID: {}\n", context.user_id));
    prompt.push_str(&format!("项目ID: {}\n", context.project_id));
    prompt.push_str(&format!("项目名称: {}\n", context.project_name));
    prompt.push_str(&format!("Agent ID: {:?}\n", context.agent_id));
    prompt.push_str(&format!("Model ID: {:?}\n", context.model_id));
    prompt.push_str("\n");
    prompt.push_str("【工作区项目的 Docker 容器名称规则】\n");
    prompt.push_str("容器名称格式: 用户ID_项目ID_容器名称\n");
    prompt.push_str("容器名称示例: 1234567890_1234567890_app-debug-base\n");

    prompt.push_str("【工作区项目的 Docker 容器完整启动方法】\n");
    prompt.push_str("**容器启动后就会常驻，不需要单独设定常驻命令，但是项目需要执行其他命令才能运行起来，容器启动不代表项目启动**。\n");
    prompt.push_str("容器启动前，需要你检测容器启动必须具备哪些生产环境，启动后，直接执行生产环境安装的命令，这个需要你通过MCP工具execute_command来完成。\n");
    prompt.push_str("容器启动后，需要**进入到容器中编译项目、打包项目、执行项目，这个需要你通过MCP工具execute_command来完成**。\n");
    prompt.push_str("项目容器启动例子(Vue-cli):\n");
    prompt.push_str("1. 进入到容器中，检测到vue项目，执行npm install\n");
    prompt.push_str("2. 编译项目\n，执行npm run build\n");
    prompt.push_str("3. 打包项目\n，执行npm run build\n，执行npm run build:prod\n");
    prompt.push_str("4. 执行项目\n，执行npm run dev或者执行npm run serve\n");
    prompt.push_str("项目容器启动例子(java-maven):\n");
    prompt.push_str("1. 进入到容器中，检测到java-maven项目，执行mvn install\n");
    prompt.push_str("2. 编译项目\n，执行mvn compile\n");
    prompt.push_str("3. 打包项目\n，执行mvn package\n");
    prompt.push_str("4. 执行项目\n，执行mvn run或者执行java -jar target/your-project-name.jar\n");
    prompt.push_str("项目容器启动例子(java-spring-boot):\n");
    prompt.push_str("1. 进入到容器中，检测到java-maven项目，执行mvn install\n");
    prompt.push_str("2. 编译项目\n，执行mvn compile\n");
    prompt.push_str("3. 打包项目\n，执行mvn package\n");
    prompt.push_str("4. 执行项目\n，执行mvn run或者执行java -jar target/your-project-name.jar\n");
    prompt.push_str("项目容器启动例子(go-lang):\n");
    prompt.push_str("1. 进入到容器中，检测到go-lang项目，执行go install\n");
    prompt.push_str("2. 编译项目\n，执行go build\n");
    prompt.push_str("3. 打包项目\n，执行go package\n");
    prompt.push_str("4. 执行项目\n，执行go run\n");


    prompt.push_str("【项目文件列表】（包含文件内容）\n");
    for file in &context.project_files {
        let path = if file.directory.is_empty() {
            file.name.clone()
        } else {
            format!("{}/{}", file.directory, file.name)
        };
        prompt.push_str(&format!("- 文件路径: {}\n", path));
        if let Some(content) = &file.content {
            let preview = if content.len() > 500 {
                format!("{}\n...(内容过长，已截断)", &content[0..500])
            } else {
                content.clone()
            };
            prompt.push_str(&format!("  文件内容:\n{}\n", preview));
        }
        prompt.push_str("\n");
    }
    prompt.push_str("\n");

    prompt.push_str("【容器配置】\n");
    for config in &context.container_configs {
        if config.container_name == "docker-compose.yml" {
            continue;
        }
        let vol_dir = get_container_volume_directory(context.user_id, context.project_id, &config.container_name);
        prompt.push_str(&format!("容器名称: {}\n", config.container_name));
        prompt.push_str(&format!("镜像: {}\n", config.image_name));
        prompt.push_str(&format!("端口映射: {}\n", config.published_ports));
        prompt.push_str(&format!("环境变量: {}\n", config.environment));
        prompt.push_str(&format!("工作目录: {}\n", config.working_dir));
        prompt.push_str(&format!("启动命令: {}\n", config.command));
        prompt.push_str(&format!("卷映射目录: {}\n", vol_dir.display()));
        prompt.push_str("\n");
    }
    prompt.push_str("\n");

    // prompt.push_str("【MCP 服务说明】\n");
    // let mcp_sse_port = get_mcp_sse_port(config);
    // prompt.push_str(&format!("每个容器启动后，内部端口 {} 会映射到宿主机的随机端口（docker-compose 自动分配）。\n", mcp_sse_port));
    // prompt.push_str("容器内置了 MCP-SSE 服务，可以通过以下方式注册和使用工具：\n\n");
    // prompt.push_str("1. **注册 MCP 工具**：\n");
    // prompt.push_str("   - 容器启动后，MCP-SSE 服务会在端口 80 上监听\n");
    // prompt.push_str("   - 使用 execute_command 工具执行：curl -X POST http://localhost:80/mcp/register -d '{\"name\": \"tool_name\", \"description\": \"tool_description\"}'\n");
    // prompt.push_str("   - 或使用容器内的 mcp-cli 命令行工具进行注册\n");
    // prompt.push_str("\n");
    prompt.push_str("1. **可用的 MCP 工具**：\n");
    prompt.push_str("   - execute_command: 在容器内执行命令\n");
    // prompt.push_str("   - file_reader: 读取文件内容\n");
    // prompt.push_str("   - file_writer: 写入文件内容\n");
    // prompt.push_str("   - environment_check: 检测代码环境（Python/Node.js/Go/Rust/Java 等）\n");
    // prompt.push_str("   - build_check: 检测编译结果\n");
    // prompt.push_str("   - process_monitor: 监控进程状态\n");
    // prompt.push_str("   - port_check: 检查端口监听状态\n");
    prompt.push_str("\n");
    prompt.push_str("2. **使用 MCP 工具**：\n");
    prompt.push_str("   - 通过 LLM 调用时，工具名称为 execute_command\n");
    prompt.push_str("   - 在容器内可以通过 HTTP 请求调用 MCP 服务\n");
    prompt.push_str("   - 示例：curl -X POST http://localhost:80/mcp/call -d '{\"tool\": \"execute_command\", \"args\": {\"command\": \"ls -la\"}}'\n");
    prompt.push_str("\n");

    prompt.push_str("【部署任务】\n");
    prompt.push_str("你的任务是：\n");
    // prompt.push_str("1. **分析文件归属**：根据文件内容和容器配置，分析每个文件应该属于哪个容器。\n");
    // prompt.push_str("   - 如果是微服务项目，每个微服务应该有独立的代码文件\n");
    // prompt.push_str("   - 如果是单体项目，所有文件可以放在同一个容器中\n");
    // prompt.push_str("   - 配置文件可能需要共享或分别放置\n");
    // prompt.push_str("\n");
    // prompt.push_str("1. **创建卷映射目录**：为每个容器创建独立的卷目录\n");
    // prompt.push_str("   - 目录格式：/debug/{user_id}/{project_id}/volumes/{container_name}/\n");
    // prompt.push_str("   - 使用 execute_command 工具创建目录：mkdir -p /debug/{user_id}/{project_id}/volumes/{container_name}\n");
    // prompt.push_str("\n");
    // prompt.push_str("2. **分配文件到容器**：将分析后的文件写入对应容器的卷目录\n");
    // prompt.push_str("   - 使用 execute_command 工具和 echo/cat 命令写入文件\n");
    // prompt.push_str("   - 保持原有的目录结构\n");
    // prompt.push_str("   - 例如：echo 'content' > /debug/{user_id}/{project_id}/volumes/{container_name}/path/to/file\n");
    // prompt.push_str("\n");
    // prompt.push_str("1. **修改 docker-compose.yml**：根据实际的卷映射需求调整配置\n");
    // prompt.push_str("   - 确保每个容器的卷映射正确指向外部目录\n");
    // prompt.push_str("   - 内部端口 80 是 MCP-SSE 通信端口，docker-compose 会自动映射到宿主机随机端口\n");
    // prompt.push_str("\n");
    prompt.push_str("1. **启动容器**：你在使用的时候，应该是已经启动了， 如果没有启动，请使用 docker-compose up -d 启动所有容器\n");
    prompt.push_str("\n");
    // prompt.push_str("2. **获取映射端口**：使用 docker-compose ps 或 docker port 命令获取容器端口映射信息\n");
    // prompt.push_str("   - 示例：docker-compose ps --format json | jq '.[] | {name, ports}'\n");
    // prompt.push_str("   - 或：docker port {container_name} 80\n");
    prompt.push_str("\n");
    // prompt.push_str("6. **注册 MCP 工具**：根据代码类型注册相应的 MCP 工具\n"); 
    // prompt.push_str("   - 使用 execute_command 工具在容器内执行注册命令\n");
    // prompt.push_str("   - 例如：docker exec {container_name} curl -X POST http://localhost:80/mcp/register -d '{...}'\n");
    // prompt.push_str("\n");
    prompt.push_str("3. **检测代码环境**：分析代码内容，确定编程语言和版本\n");
    // prompt.push_str("   - 使用 environment_check 工具检测编程语言和版本，这个工具目前还不存在，你自己先分析一下。\n");
    // prompt.push_str("   - 使用 file_reader 工具读取配置文件（package.json, requirements.txt, go.mod 等类似的配置文件）\n");
    // prompt.push_str("   - 根据检测结果安装必要的依赖\n");
    prompt.push_str("\n");
    prompt.push_str("4. **安装依赖和启动应用**：根据代码类型安装必要软件并启动应用，特别注意编译项目需要的安装环境，编译环境以及运行环境。\n");
    prompt.push_str("   - 使用 execute_command 工具执行安装命令\n");
    // prompt.push_str("   - 使用 build_check 工具检测编译结果，此工具也不知道存在与否，如果不存在，需要你自己根据execute_command进行检测\n");
    // prompt.push_str("   - 使用 port_check 工具检查应用端口是否正常监听\n");
    prompt.push_str("\n");
    prompt.push_str("5. **监控和验证**：\n");
    // prompt.push_str("   - 使用 process_monitor 工具监控应用进程状态\n");
    // prompt.push_str("   - 使用 file_reader 工具查看应用日志\n");
    prompt.push_str("   - 确保 MCP-SSE 服务（端口 80）正常运行\n");
    prompt.push_str("\n");

    prompt.push_str("【注意事项】\n");
    prompt.push_str("- 容器名称格式：{user_id}-{project_id}-{容器配置编号}\n");
    prompt.push_str("- 卷目录格式：/debug/{user_id}/{project_id}/volumes/{容器名称}/\n");
    prompt.push_str("- 卷映射格式：/debug/{user_id}/{project_id}/volumes/{容器名称}/:/app（或配置的工作目录）\n");
    prompt.push_str("- 容器内部端口 80 是 MCP-SSE 通信端口，docker-compose 会自动映射到宿主机随机端口\n");
    prompt.push_str("- 获取映射端口后，才能通过 HTTP 访问容器内的 MCP 服务\n");
    prompt.push_str("- 如果项目有特殊依赖，请根据代码内容判断并安装\n");
    prompt.push_str("- 使用 execute_command 工具执行所有操作\n");
    prompt.push_str("\n");

    prompt.push_str("请开始执行部署操作，并在完成后报告部署结果。");

    prompt
}

pub fn generate_llm_execute_command_prompt(command: &str, context: &ProjectDeploymentContext) -> String {
    format!(
        "请在工作区项目容器中执行以下命令。\n\n项目信息：用户ID={}, 项目ID={}\n\n命令：{}\n\n请执行命令并返回执行结果。如果命令涉及容器操作，请使用正确的容器名称格式：USER_ID-PROJECT_ID-容器名称",
        context.user_id, context.project_id, command
    )
}

pub struct ContainerDeployer;

impl ContainerDeployer {
    pub async fn deploy_project(
        user_id: i64,
        project_id: i64,
        project_name: String,
        agent_id: Option<i64>,
        model_id: Option<i64>,
        container_configs: Vec<ProjectContainerConfig>,
        project_files: Vec<ProjectFileInfo>,
    ) -> Result<ContainerDeploymentResult, String> {
        let debug_dir = ensure_debug_directory(user_id, project_id)?;

        for config in &container_configs {
            if config.container_name == "docker-compose.yml" {
                continue;
            }
            ensure_container_volume_directory(user_id, project_id, &config.container_name)?;
        }

        let mut dockerfile_names: HashMap<i64, String> = HashMap::new();
        let mut container_names: Vec<String> = Vec::new();

        for config in &container_configs {
            if config.container_name == "docker-compose.yml" {
                continue;
            }

            let dockerfile_content = generate_dockerfile_content(config);
            let name = write_dockerfile(&debug_dir, config.id, &dockerfile_content)?;
            dockerfile_names.insert(config.id, name);
            container_names.push(format_container_name(user_id, project_id, config.id));
        }

        let context = ProjectDeploymentContext {
            user_id,
            project_id,
            project_name,
            agent_id,
            model_id,
            container_configs: container_configs.clone(),
            project_files,
            container_name: String::new(),
            mcp_server_url: String::new(),
            debug_dir: debug_dir.clone(),
        };

        let compose_config = generate_docker_compose_config(&context, &dockerfile_names);
        let compose_path = write_docker_compose(&debug_dir, &compose_config)?;

        let repo = WorkspaceRepository::new();
        for config in &container_configs {
            if config.container_name == "docker-compose.yml" {
                continue;
            }

            match repo.get_container_files_for_deployment(project_id, config.id).await {
                Ok(container_files) => {
                    write_project_files_to_volume(user_id, project_id, &config.container_name, &container_files)?;
                    println!("[deploy_project] Wrote {} files to container {}", container_files.len(), config.container_name);
                }
                Err(e) => {
                    println!("[deploy_project] Failed to get files for container {}: {}, using all files as fallback", config.container_name, e);
                    write_project_files_to_volume(user_id, project_id, &config.container_name, &context.project_files)?;
                }
            }
        }

        Ok(ContainerDeploymentResult {
            success: true,
            message: "Deployment files generated successfully".to_string(),
            debug_dir: debug_dir.to_string_lossy().to_string(),
            container_names,
            docker_compose_path: compose_path,
        })
    }

    pub async fn start_containers(debug_dir: &PathBuf) -> Result<ContainerStartResult, String> {
        // 从目录路径提取 user_id 和 project_id
        // 目录格式: /debug/{user_id}/{project_id}
        let project_name = debug_dir
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("default");
        
        let parent_name = debug_dir
            .parent()
            .and_then(|p| p.file_name())
            .and_then(|n| n.to_str())
            .unwrap_or("0");
        
        // 使用 user_id-project_id 作为项目名称，确保网络名称与容器名称匹配
        let compose_project_name = format!("{}-{}", parent_name, project_name);
        
        // 启动之前：从 docker-compose.yml 读取容器名称和网络名称，删除冲突的资源
        let compose_path = debug_dir.join("docker-compose.yml");
        if compose_path.exists() {
            if let Ok(content) = std::fs::read_to_string(&compose_path) {
                for line in content.lines() {
                    if line.contains("container_name:") {
                        let parts: Vec<&str> = line.split(":").collect();
                        if parts.len() >= 2 {
                            let container_name = parts[1].trim().replace("\"", "").replace("'", "");
                            println!("remove container {}",container_name.clone());
                            if !container_name.is_empty() {
                                let _ = Command::new("docker")
                                    .arg("rm")
                                    .arg("-f")
                                    .arg(&container_name)
                                    .output();
                            }
                        }
                    } else if line.starts_with(" ") && line.contains(":") && !line.contains("container_name:") && !line.contains("image:") && !line.contains("ports:") && !line.contains("volumes:") && !line.contains("environment:") && !line.contains("command:") && !line.contains("working_dir:") {
                        let parts: Vec<&str> = line.split(":").collect();
                        if parts.len() >= 2 {
                            let network_name = parts[0].trim();
                            if !network_name.is_empty() && network_name != "external" && network_name != "name" && network_name != "driver" {
                                let full_network_name = format!("{}_{}", compose_project_name, network_name);
                                println!("remove network: {}",full_network_name.clone());
                                let _ = Command::new("docker")
                                    .arg("network")
                                    .arg("rm")
                                    .arg(&full_network_name)
                                    .output();
                            }
                        }
                    }
                }
            }
        }
        
        let output = Command::new("docker-compose")
            .current_dir(debug_dir)
            .arg("-p")
            .arg(&compose_project_name)
            .arg("up")
            .arg("-d")
            .arg("--force-recreate")
            .output()
            .map_err(|e| format!("Failed to execute docker-compose: {}", e))?;

        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();

        if !output.status.success() {
            return Err(format!("docker-compose up failed: {}", stderr));
        }

        let port_info = Self::get_port_mappings(debug_dir).await?;

        Ok(ContainerStartResult {
            output: format!("{}\n{}", stdout, stderr),
            port_mappings: port_info,
        })
    }

    async fn get_port_mappings(debug_dir: &PathBuf) -> Result<Vec<PortMapping>, String> {
        // 从目录路径提取 user_id 和 project_id
        let project_name = debug_dir
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("default");
        
        let parent_name = debug_dir
            .parent()
            .and_then(|p| p.file_name())
            .and_then(|n| n.to_str())
            .unwrap_or("0");
        
        let compose_project_name = format!("{}-{}", parent_name, project_name);
        
        let output = Command::new("docker-compose")
            .current_dir(debug_dir)
            .arg("-p")
            .arg(&compose_project_name)
            .arg("ps")
            .arg("--format")
            .arg("json")
            .output()
            .map_err(|e| format!("Failed to get container ports: {}", e))?;

        let stdout = String::from_utf8_lossy(&output.stdout).to_string();

        let mut mappings: Vec<PortMapping> = Vec::new();
        for line in stdout.lines() {
            if !line.trim().is_empty() {
                match serde_json::from_str::<DockerComposePsInfo>(line) {
                    Ok(info) => {
                        let ports = info.ports.clone().unwrap_or_default();
                        if !ports.is_empty() {
                            for port in ports.split(',') {
                                let trimmed = port.trim();
                                if !trimmed.is_empty() {
                                    let parts: Vec<&str> = trimmed.split("->").collect();
                                    if parts.len() == 2 {
                                        mappings.push(PortMapping {
                                            container_name: info.name.clone(),
                                            host_port: parts[0].trim().to_string(),
                                            container_port: parts[1].trim().to_string(),
                                        });
                                    }
                                }
                            }
                        }
                    }
                    Err(_) => continue,
                }
            }
        }

        Ok(mappings)
    }

    pub async fn call_llm_for_deployment(context: &ProjectDeploymentContext) -> Result<String, String> {
        if context.model_id.is_none() {
            return Err("No model configured for project".to_string());
        }

        let model_id = context.model_id.unwrap();
        let container_name = context.container_name.clone();
        let mcp_server_url = context.mcp_server_url.clone();

        println!("[DEBUG] Direct LLM call: model_id={}, container={}, mcp_url={}",
                 model_id, container_name, mcp_server_url);

        // 1. 从 llm-svc 获取模型信息
        let llm_svc_url = env::var("LLM_SVC_URL")
            .unwrap_or_else(|_| "http://llm-svc:8080".to_string());

        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(120))
            .build()
            .map_err(|e| format!("Failed to create HTTP client: {}", e))?;

        let model_resp = client
            .get(format!("{}/api/models/{}", llm_svc_url, model_id))
            .send()
            .await
            .map_err(|e| format!("Failed to get model: {}", e))?;

        if !model_resp.status().is_success() {
            return Err(format!("Failed to get model info: {}", model_resp.status()));
        }

        let model: serde_json::Value = model_resp.json()
            .await
            .map_err(|e| format!("Failed to parse model: {}", e))?;

        let model_base_url = model.get("access_url").and_then(|v| v.as_str())
            .ok_or("Model access_url not found")?;
        let model_api_key = model.get("api_key").and_then(|v| v.as_str())
            .ok_or("Model api_key not found")?;
        let model_name = model.get("name").and_then(|v| v.as_str())
            .ok_or("Model name not found")?;

        println!("[DEBUG] Model: name={}, base_url={}", model_name, model_base_url);

        // 2. 构造 LLM 客户端（使用从 llm-svc 复制的 LlmClient 逻辑）
        let llm_client = LlmClient::new(model_base_url, model_api_key, model_name);

        // 3. 从容器 MCP-SSE 服务动态获取工具列表
        // 工具列表完全从容器获取，不使用写死的 fallback，因为工具会更新
        let tool_executor = ToolExecutor::new(HashMap::new(), &mcp_server_url, None);
        println!("[DEBUG] ToolExecutor created with URL: {}", mcp_server_url);
        let mut count = 0;
        //尝试3分钟
        let max_attempts = 60;
        loop {
                let result = tool_executor.list_tools(None).await;
                match result {
                Ok(t) => {
                    println!("[DEBUG] Retrieved {} tools from container MCP-SSE", t.len());
                    break;
                },
                Err(e) => {
                    println!("[DEBUG] Failed to get tools from container: {}", e);

                    //每个一秒检测一次工具列表是否可用
                    tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
                    count += 1;
                    if count >= max_attempts {
                        break;
                    }
                }
            }
        }
        let tools = match tool_executor.list_tools(None).await {
            Ok(t) => {
                println!("[DEBUG] Retrieved {} tools from container MCP-SSE", t.len());
                t
            },
            Err(e) => {
                println!("[DEBUG] Failed to get tools from container: {}", e);
                Vec::new()
            }
        };

        // 4. 构建消息
        let prompt = generate_llm_deployment_prompt(context);
        let messages = vec![
            ChatMessage {
                role: "system".to_string(),
                content: Some("[ROLE]你是一个专业的项目部署助手。你需要分析项目文件，智能执行命令来启动调试服务器。请使用 execute_command工具来执行命令（如 npm install、npm run serve 等）。工作目录默认为 /app。".to_string()),
                tool_call_id: None,
                name: None,
                tool_calls: None,
            },
            ChatMessage {
                role: "system".to_string(),
                content: Some("[TOOL USAGE] 你必须使用 execute_command 工具来执行命令，使用格式为 execute_command(command, workDir)，必须传入 command 和 workDir 参数，其中command是执行的命令，必须传入，不能省略，workDir是命令执行的工作目录，必须传入，不能省略。".to_string()),
                tool_call_id: None,
                name: None,
                tool_calls: None,
            },
              ChatMessage {
                role: "system".to_string(),
                content: Some("[TOOL USAGE RULE] 当你使用 execute_command 执行命令时，首选确定 command 命令存在，例如 mvn 命令，首先通过 whereis mvn 查看是否存在，如果不存在，需要你通过系统命令 apt install <package-name> 安装命令。".to_string()),
                tool_call_id: None,
                name: None,
                tool_calls: None,
            },
            ChatMessage {
                role: "system".to_string(),
                content: Some("[TOOL FUNCTION SCHEMA]  {\"name\":\"execute_command\",\"description\":\"Executes a system command and returns the output.  requires \\u0060command\\u0060 and \\u0060workDir\\u0060.the command is the \\u0060command\\u0060 you want to execute, the \\u0060workDir\\u0060 is the command executing in the /app directory or other effective directory.\",\"inputSchema\":{\"type\":\"object\",\"properties\":{\"command\":{\"description\":\"the command to execute\",\"type\":\"string\"},\"workDir\":{\"description\":\"working directory\",\"type\":\"string\"}},\"required\":[\"command\",\"workDir\"]}}) ".to_string()),
                tool_call_id: None,
                name: None,
                tool_calls: None,
            },
           
            ChatMessage {
                role: "user".to_string(),
                content: Some(prompt),
                tool_call_id: None,
                name: None,
                tool_calls: None,
            },
        ];


        // 5. 调用 LLM + 工具执行循环
        // 将容器 MCP-SERVER URL 作为 ToolExecutor 的 default URL，
        // 这样工具调用会通过 JSON-RPC 转发到容器内的 MCP-SERVER。
        let mcp_servers: HashMap<i64, String> = HashMap::new();
        let mut stream = llm_client
            .chat_with_tools(messages, Some(&tools), mcp_servers, &mcp_server_url, 10
                , context.user_id, context.project_id, None, Some(context.debug_dir.clone()))
            .await
            .map_err(|e| format!("LLM chat_with_tools failed: {}", e))?;

        // 6. 收集最终输出内容
        let mut full_content = String::new();
        while let Some(chunk) = stream.next().await {
            if !chunk.content.is_empty() && chunk.content != "[DONE]" {
                full_content.push_str(&chunk.content);
            }
            if let Some(reason) = &chunk.finish_reason {
                println!("[DEBUG] Stream finished: reason={}", reason);
                break;
            }
        }

        Ok(full_content)
    }

    pub async fn execute_command_in_container(
        user_id: i64,
        project_id: i64,
        config_id: i64,
        command: &str,
    ) -> Result<ExecuteCommandResult, String> {
        let debug_dir = get_debug_directory(user_id, project_id);
        if !debug_dir.exists() {
            return Err("Debug directory does not exist".to_string());
        }

        let full_container_name = format_container_name(user_id, project_id, config_id);

        let cmd = format!("docker exec {} {}", full_container_name, command);

        let output = Command::new("bash")
            .current_dir(&debug_dir)
            .arg("-c")
            .arg(&cmd)
            .output()
            .map_err(|e| format!("Failed to execute command: {}", e))?;

        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();

        Ok(ExecuteCommandResult {
            success: output.status.success(),
            output: stdout,
            error: if output.status.success() { None } else { Some(stderr) },
        })
    }

    pub async fn execute_command_stream(
        user_id: i64,
        project_id: i64,
        config_id: i64,
        command: &str,
    ) -> Result<impl Stream<Item = Result<String, String>>, String> {
        let debug_dir = get_debug_directory(user_id, project_id);
        if !debug_dir.exists() {
            return Err("Debug directory does not exist".to_string());
        }

        let full_container_name = format_container_name(user_id, project_id, config_id);
        let cmd = format!("docker exec {} {}", full_container_name, command);

        let mut child = tokio::process::Command::new("bash")
            .current_dir(&debug_dir)
            .arg("-c")
            .arg(&cmd)
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()
            .map_err(|e| format!("Failed to execute command: {}", e))?;

        let stdout = child.stdout.take().ok_or("Failed to get stdout")?;
        let stderr = child.stderr.take().ok_or("Failed to get stderr")?;

        let stdout_reader = tokio::io::BufReader::new(stdout);
        let stderr_reader = tokio::io::BufReader::new(stderr);

        let stdout_lines = LinesStream::new(stdout_reader.lines()).map(|line| {
            line.map_err(|e| e.to_string()).map(|line| format!("stdout: {}", line))
        });
        let stderr_lines = LinesStream::new(stderr_reader.lines()).map(|line| {
            line.map_err(|e| e.to_string()).map(|line| format!("stderr: {}", line))
        });

        let combined = futures::stream::select(stdout_lines, stderr_lines);

        Ok(combined)
    }

    pub async fn stop_containers(debug_dir: &PathBuf) -> Result<String, String> {
        // 从目录路径提取 user_id 和 project_id
        let project_name = debug_dir
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("default");
        
        let parent_name = debug_dir
            .parent()
            .and_then(|p| p.file_name())
            .and_then(|n| n.to_str())
            .unwrap_or("0");
        
        let compose_project_name = format!("{}-{}", parent_name, project_name);
        
        let output = Command::new("docker-compose")
            .current_dir(debug_dir)
            .arg("-p")
            .arg(&compose_project_name)
            .arg("down")
            .output()
            .map_err(|e| format!("Failed to execute docker-compose down: {}", e))?;

        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();

        if !output.status.success() {
            return Err(format!("docker-compose down failed: {}", stderr));
        }

        Ok(format!("{}\n{}", stdout, stderr))
    }

    pub async fn get_container_status(user_id: i64, project_id: i64) -> Result<Vec<ContainerStatus>, String> {
        let output = Command::new("docker")
            .arg("ps")
            .arg("--format")
            .arg("{{.Names}}|{{.Command}}|{{.State}}|{{.Ports}}")
            .output()
            .map_err(|e| format!("Failed to execute docker ps: {}", e))?;

        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        println!("stdout: {}", stdout);

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr).to_string();
            return Err(format!("docker ps failed: {}", stderr));
        }

        let mut statuses: Vec<ContainerStatus> = Vec::new();
        for line in stdout.lines() {
            if !line.trim().is_empty() {
                let parts: Vec<&str> = line.splitn(4, '|').collect();
                if parts.len() >= 3 {
                    statuses.push(ContainerStatus {
                        name: parts[0].to_string(),
                        command: parts.get(1).unwrap_or(&"").to_string(),
                        state: parts.get(2).unwrap_or(&"").to_string(),
                        ports: parts.get(3).unwrap_or(&"").to_string(),
                    });
                }
            }
        }

        Ok(statuses)
    }

    pub async fn get_container_logs(user_id: i64, project_id: i64, container_name: Option<&str>, tail: Option<usize>) -> Result<String, String> {
        let debug_dir = get_debug_directory(user_id, project_id);
        if !debug_dir.exists() {
            return Err("Debug directory does not exist".to_string());
        }

        let mut cmd = vec!["logs".to_string()];
        if let Some(n) = tail {
            cmd.push("--tail".to_string());
            cmd.push(n.to_string());
        }
        if let Some(name) = container_name {
            cmd.push(name.to_string());
        }

        let output = Command::new("docker-compose")
            .current_dir(&debug_dir)
            .args(&cmd)
            .output()
            .map_err(|e| format!("Failed to execute docker-compose logs: {}", e))?;

        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();

        if !output.status.success() {
            return Err(format!("docker-compose logs failed: {}", stderr));
        }

        Ok(format!("{}\n{}", stdout, stderr))
    }

    pub async fn cleanup_debug_directory(user_id: i64, project_id: i64) -> Result<String, String> {
        let debug_dir = get_debug_directory(user_id, project_id);
        if !debug_dir.exists() {
            return Ok("Debug directory does not exist, nothing to clean".to_string());
        }

        let _ = Self::stop_containers(&debug_dir).await;

        match fs::remove_dir_all(&debug_dir) {
            Ok(_) => Ok(format!("Successfully cleaned debug directory: {}", debug_dir.display())),
            Err(e) => Err(format!("Failed to clean debug directory: {}", e)),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ExecuteCommandResult {
    pub success: bool,
    pub output: String,
    pub error: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ContainerStatus {
    pub name: String,
    pub command: String,
    pub state: String,
    pub ports: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ContainerStartResult {
    pub output: String,
    pub port_mappings: Vec<PortMapping>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PortMapping {
    pub container_name: String,
    pub host_port: String,
    pub container_port: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct DockerComposePsInfo {
    pub name: String,
    pub command: String,
    pub state: String,
    pub ports: Option<String>,
}

