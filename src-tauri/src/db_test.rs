//! 数据库功能测试模块
use crate::database::{SQLiteDatabase, DocumentCreate, DocumentUpdate};
use std::path::Path;

pub async fn test_database_operations() -> Result<(), Box<dyn std::error::Error>> {
    println!("开始测试数据库操作...");
    
    // 创建临时数据库
    let db_path = Path::new("test_database.db");
    let db = SQLiteDatabase::new(db_path).await?;
    
    println!("✓ 数据库连接成功");
    
    // 测试创建文档
    let doc_create = DocumentCreate {
        title: "测试文档".to_string(),
        content: "这是测试内容".to_string(),
        embedding: Some(vec![0.1, 0.2, 0.3]),
        metadata: Some(serde_json::json!({"author": "test"})),
    };
    
    let created_doc = db.create_document(doc_create).await?;
    println!("✓ 文档创建成功: {}", created_doc.id);
    
    // 测试获取文档
    let retrieved_doc = db.get_document(&created_doc.id).await?;
    assert!(retrieved_doc.is_some());
    println!("✓ 文档查询成功");
    
    // 测试更新文档
    let doc_update = DocumentUpdate {
        title: Some("更新后的标题".to_string()),
        content: None,
        embedding: None,
        metadata: None,
    };
    
    let updated_doc = db.update_document(&created_doc.id, doc_update).await?;
    assert!(updated_doc.is_some());
    assert_eq!(updated_doc.unwrap().title, "更新后的标题");
    println!("✓ 文档更新成功");
    
    // 测试列表查询
    let docs = db.list_documents(10, 0).await?;
    assert!(!docs.is_empty());
    println!("✓ 文档列表查询成功，共 {} 条记录", docs.len());
    
    // 测试删除文档
    let deleted = db.delete_document(&created_doc.id).await?;
    assert!(deleted);
    println!("✓ 文档删除成功");
    
    // 清理测试文件
    std::fs::remove_file(db_path)?;
    println!("✓ 测试完成，清理临时文件");
    
    Ok(())
}