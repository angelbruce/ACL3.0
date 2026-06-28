//! 领域本体定义模块
//!
//! 定义了知识图谱的实体类型、关系类型和属性结构。
//! 遵循自定义领域本体框架，针对任务管理、文档处理等业务场景。

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// 本体定义
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Ontology {
    pub name: String,
    pub description: String,
    pub entity_types: Vec<EntityType>,
    pub relation_types: Vec<RelationType>,
}

/// 实体类型定义
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntityType {
    pub name: String,
    pub description: String,
    pub properties: Vec<Property>,
    pub parent: Option<String>,
}

/// 关系类型定义
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RelationType {
    pub name: String,
    pub description: String,
    pub source_types: Vec<String>,
    pub target_types: Vec<String>,
}

/// 属性定义
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Property {
    pub name: String,
    pub prop_type: String,
    pub required: bool,
    pub description: String,
}

/// 获取领域本体定义
pub fn get_domain_ontology() -> Ontology {
    Ontology {
        name: "TaskDocumentDomain".to_string(),
        description: "任务文档管理领域的知识图谱本体".to_string(),
        entity_types: vec![
            EntityType {
                name: "Task".to_string(),
                description: "任务实体，表示需要完成的工作项".to_string(),
                parent: None,
                properties: vec![
                    Property {
                        name: "status".to_string(),
                        prop_type: "string".to_string(),
                        required: false,
                        description: "任务状态".to_string(),
                    },
                    Property {
                        name: "priority".to_string(),
                        prop_type: "string".to_string(),
                        required: false,
                        description: "任务优先级".to_string(),
                    },
                    Property {
                        name: "category".to_string(),
                        prop_type: "string".to_string(),
                        required: false,
                        description: "任务分类".to_string(),
                    },
                ],
            },
            EntityType {
                name: "Document".to_string(),
                description: "文档实体，表示知识载体".to_string(),
                parent: None,
                properties: vec![
                    Property {
                        name: "type".to_string(),
                        prop_type: "string".to_string(),
                        required: false,
                        description: "文档类型".to_string(),
                    },
                    Property {
                        name: "format".to_string(),
                        prop_type: "string".to_string(),
                        required: false,
                        description: "文档格式".to_string(),
                    },
                ],
            },
            EntityType {
                name: "Decision".to_string(),
                description: "决策实体，表示已做出的决定".to_string(),
                parent: None,
                properties: vec![
                    Property {
                        name: "context".to_string(),
                        prop_type: "string".to_string(),
                        required: false,
                        description: "决策背景".to_string(),
                    },
                    Property {
                        name: "conclusion".to_string(),
                        prop_type: "string".to_string(),
                        required: false,
                        description: "决策结论".to_string(),
                    },
                ],
            },
            EntityType {
                name: "Person".to_string(),
                description: "人员实体".to_string(),
                parent: None,
                properties: vec![
                    Property {
                        name: "role".to_string(),
                        prop_type: "string".to_string(),
                        required: false,
                        description: "角色".to_string(),
                    },
                ],
            },
            EntityType {
                name: "Organization".to_string(),
                description: "组织实体".to_string(),
                parent: None,
                properties: vec![
                    Property {
                        name: "type".to_string(),
                        prop_type: "string".to_string(),
                        required: false,
                        description: "组织类型".to_string(),
                    },
                ],
            },
            EntityType {
                name: "Product".to_string(),
                description: "产品/系统实体".to_string(),
                parent: None,
                properties: vec![
                    Property {
                        name: "version".to_string(),
                        prop_type: "string".to_string(),
                        required: false,
                        description: "版本".to_string(),
                    },
                ],
            },
            EntityType {
                name: "Technology".to_string(),
                description: "技术/工具实体".to_string(),
                parent: None,
                properties: vec![
                    Property {
                        name: "category".to_string(),
                        prop_type: "string".to_string(),
                        required: false,
                        description: "技术分类".to_string(),
                    },
                ],
            },
            EntityType {
                name: "Concept".to_string(),
                description: "概念实体，表示抽象概念或术语".to_string(),
                parent: None,
                properties: vec![
                    Property {
                        name: "domain".to_string(),
                        prop_type: "string".to_string(),
                        required: false,
                        description: "所属领域".to_string(),
                    },
                ],
            },
        ],
        relation_types: vec![
            RelationType {
                name: "requires".to_string(),
                description: "任务需要文档".to_string(),
                source_types: vec!["Task".to_string()],
                target_types: vec!["Document".to_string()],
            },
            RelationType {
                name: "resolves".to_string(),
                description: "决策解决任务".to_string(),
                source_types: vec!["Decision".to_string()],
                target_types: vec!["Task".to_string()],
            },
            RelationType {
                name: "assigned_to".to_string(),
                description: "任务分配给人员".to_string(),
                source_types: vec!["Task".to_string()],
                target_types: vec!["Person".to_string()],
            },
            RelationType {
                name: "belongs_to".to_string(),
                description: "实体属于组织".to_string(),
                source_types: vec!["Task".to_string(), "Document".to_string(), "Person".to_string()],
                target_types: vec!["Organization".to_string()],
            },
            RelationType {
                name: "uses".to_string(),
                description: "任务使用技术/工具".to_string(),
                source_types: vec!["Task".to_string()],
                target_types: vec!["Technology".to_string(), "Product".to_string()],
            },
            RelationType {
                name: "references".to_string(),
                description: "文档引用文档".to_string(),
                source_types: vec!["Document".to_string()],
                target_types: vec!["Document".to_string()],
            },
            RelationType {
                name: "related_to".to_string(),
                description: "概念相关".to_string(),
                source_types: vec!["Concept".to_string()],
                target_types: vec!["Concept".to_string(), "Task".to_string(), "Document".to_string()],
            },
            RelationType {
                name: "implements".to_string(),
                description: "技术实现产品".to_string(),
                source_types: vec!["Technology".to_string()],
                target_types: vec!["Product".to_string()],
            },
            RelationType {
                name: "created_by".to_string(),
                description: "文档由人员创建".to_string(),
                source_types: vec!["Document".to_string()],
                target_types: vec!["Person".to_string()],
            },
            RelationType {
                name: "provides".to_string(),
                description: "产品提供服务".to_string(),
                source_types: vec!["Product".to_string()],
                target_types: vec!["Concept".to_string()],
            },
        ],
    }
}

/// 实体类型到中文描述的映射
pub fn entity_type_cn_map() -> HashMap<String, String> {
    let mut map = HashMap::new();
    map.insert("Task".to_string(), "任务".to_string());
    map.insert("Document".to_string(), "文档".to_string());
    map.insert("Decision".to_string(), "决策".to_string());
    map.insert("Person".to_string(), "人员".to_string());
    map.insert("Organization".to_string(), "组织".to_string());
    map.insert("Product".to_string(), "产品".to_string());
    map.insert("Technology".to_string(), "技术".to_string());
    map.insert("Concept".to_string(), "概念".to_string());
    map
}

/// 关系类型到中文描述的映射
pub fn relation_type_cn_map() -> HashMap<String, String> {
    let mut map = HashMap::new();
    map.insert("requires".to_string(), "需要".to_string());
    map.insert("resolves".to_string(), "解决".to_string());
    map.insert("assigned_to".to_string(), "分配给".to_string());
    map.insert("belongs_to".to_string(), "属于".to_string());
    map.insert("uses".to_string(), "使用".to_string());
    map.insert("references".to_string(), "引用".to_string());
    map.insert("related_to".to_string(), "相关".to_string());
    map.insert("implements".to_string(), "实现".to_string());
    map.insert("created_by".to_string(), "创建于".to_string());
    map.insert("provides".to_string(), "提供".to_string());
    map
}
