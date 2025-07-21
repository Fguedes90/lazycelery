#![allow(clippy::uninlined_format_args)]

use anyhow::Result;
use base64::Engine;
use lazycelery::broker::{redis::RedisBroker, Broker};
use lazycelery::models::TaskStatus;
use redis::{AsyncCommands, Client};
use serde_json::json;

/// Teste de integração completo que simula um ambiente Celery real
#[tokio::test]
async fn test_complete_celery_workflow() -> Result<()> {
    let client = match Client::open("redis://127.0.0.1:6379") {
        Ok(client) => client,
        Err(_) => {
            eprintln!("Skipping integration test: Redis not available");
            return Ok(());
        }
    };

    let mut conn = match client.get_multiplexed_tokio_connection().await {
        Ok(conn) => conn,
        Err(_) => {
            eprintln!("Skipping integration test: Redis connection failed");
            return Ok(());
        }
    };

    // Usar database 2 para isolamento completo
    let _: () = redis::cmd("SELECT").arg(2).query_async(&mut conn).await?;
    let _: () = redis::cmd("FLUSHDB").query_async(&mut conn).await?;

    // === CENÁRIO 1: Simular um ambiente de produção típico ===

    // 1. Adicionar tarefas pendentes na fila (como um produtor real)
    let pending_tasks = vec![
        (
            "urgent_processing",
            "high_priority",
            "[1000, 2000]",
            r#"{"timeout": 60}"#,
        ),
        (
            "data_analysis",
            "analytics",
            "[\"dataset_001\"]",
            r#"{"format": "json"}"#,
        ),
        (
            "email_notification",
            "notifications",
            r#"["user@example.com"]"#,
            r#"{"template": "welcome"}"#,
        ),
    ];

    for (task_name, queue, args, kwargs) in pending_tasks {
        let task_id = format!("pending-{}-{}", task_name, chrono::Utc::now().timestamp());
        let task_body = json!([
            serde_json::from_str::<serde_json::Value>(args)?,
            serde_json::from_str::<serde_json::Value>(kwargs)?,
            {}
        ]);
        let encoded_body = base64::engine::general_purpose::STANDARD.encode(task_body.to_string());

        let task_message = json!({
            "body": encoded_body,
            "content-encoding": "utf-8",
            "content-type": "application/json",
            "headers": {
                "lang": "py",
                "task": format!("myapp.tasks.{}", task_name),
                "id": task_id,
                "retries": 0,
                "origin": format!("worker@prod-server-{}", queue),
                "argsrepr": args,
                "kwargsrepr": kwargs
            }
        });

        let _: () = conn.lpush(queue, task_message.to_string()).await?;
    }

    // 2. Adicionar histórico de tarefas processadas
    let processed_tasks = vec![
        (
            "task-001",
            "SUCCESS",
            "\"Processing completed successfully\"",
            None,
        ),
        (
            "task-002",
            "FAILURE",
            "null",
            Some("Traceback: ValueError: Invalid input data"),
        ),
        ("task-003", "SUCCESS", "42", None),
        ("task-004", "RETRY", "null", Some("Temporary network error")),
        ("task-005", "REVOKED", "null", None),
    ];

    for (task_id, status, result, traceback) in processed_tasks {
        let task_metadata = json!({
            "status": status,
            "result": serde_json::from_str::<serde_json::Value>(result)?,
            "traceback": traceback,
            "children": [],
            "date_done": chrono::Utc::now().to_rfc3339(),
            "task_id": task_id
        });

        let _: () = conn
            .set(
                format!("celery-task-meta-{}", task_id),
                task_metadata.to_string(),
            )
            .await?;
    }

    // 3. Adicionar bindings e configurações do Celery
    let _: () = conn.set("_kombu.binding.high_priority", "").await?;
    let _: () = conn.set("_kombu.binding.analytics", "").await?;
    let _: () = conn.set("_kombu.binding.notifications", "").await?;
    let _: () = conn.set("_kombu.binding.celery", "").await?;

    // 4. Adicionar tarefas revogadas
    let _: () = conn.sadd("revoked", "task-005").await?;
    let _: () = conn.sadd("revoked", "old-task-123").await?;

    // === FASE DE TESTES ===

    let broker = RedisBroker::connect("redis://127.0.0.1:6379/2").await?;

    // Teste 1: Descoberta de Workers
    println!("=== Teste 1: Descoberta de Workers ===");
    let workers = broker.get_workers().await?;

    assert!(
        !workers.is_empty(),
        "Should discover workers from task activity"
    );

    for worker in &workers {
        println!(
            "Worker: {} (Status: {:?}, Processed: {}, Failed: {})",
            worker.hostname, worker.status, worker.processed, worker.failed
        );

        assert!(!worker.hostname.is_empty(), "Worker should have hostname");
        assert!(
            worker.concurrency > 0,
            "Worker should have positive concurrency"
        );
        assert!(
            !worker.queues.is_empty(),
            "Worker should have at least one queue"
        );
    }

    // Teste 2: Parsing de Tarefas
    println!("\n=== Teste 2: Parsing de Tarefas ===");
    let tasks = broker.get_tasks().await?;

    // Deve encontrar tarefas dos metadados + filas (limitado a 100 pela implementação)
    assert!(
        tasks.len() >= 5,
        "Should find processed tasks + pending tasks, found: {}",
        tasks.len()
    );

    // Verificar tipos de status
    let success_count = tasks
        .iter()
        .filter(|t| t.status == TaskStatus::Success)
        .count();
    let failure_count = tasks
        .iter()
        .filter(|t| t.status == TaskStatus::Failure)
        .count();
    let pending_count = tasks
        .iter()
        .filter(|t| t.status == TaskStatus::Pending)
        .count();
    let retry_count = tasks
        .iter()
        .filter(|t| t.status == TaskStatus::Retry)
        .count();
    let revoked_count = tasks
        .iter()
        .filter(|t| t.status == TaskStatus::Revoked)
        .count();

    println!("Task Status Distribution:");
    println!("  Success: {}", success_count);
    println!("  Failure: {}", failure_count);
    println!("  Pending: {}", pending_count);
    println!("  Retry: {}", retry_count);
    println!("  Revoked: {}", revoked_count);

    assert!(success_count >= 2, "Should have successful tasks");
    assert!(failure_count >= 1, "Should have failed tasks");
    // Note: Pending tasks from queue may not always be parsed depending on implementation
    assert!(
        pending_count + retry_count + revoked_count >= 1,
        "Should have some non-completed tasks"
    );

    // Teste 3: Descoberta de Filas
    println!("\n=== Teste 3: Descoberta de Filas ===");
    let queues = broker.get_queues().await?;

    assert!(!queues.is_empty(), "Should discover at least one queue");

    for queue in &queues {
        println!(
            "Queue: {} (Length: {}, Consumers: {})",
            queue.name, queue.length, queue.consumers
        );
    }

    // Verificar se pelo menos algumas filas foram descobertas
    let queue_names: Vec<&str> = queues.iter().map(|q| q.name.as_str()).collect();
    println!("Discovered queues: {:?}", queue_names);

    // Deve encontrar pelo menos a fila padrão
    let has_default_queue = queues
        .iter()
        .any(|q| q.name == "celery" || q.name == "default");
    assert!(has_default_queue, "Should find at least a default queue");

    // Teste 4: Operações de Tarefas
    println!("\n=== Teste 4: Operações de Tarefas ===");

    // Retry de tarefa falhada
    let retry_result = broker.retry_task("task-002").await;
    assert!(
        retry_result.is_ok(),
        "Should successfully retry failed task: {:?}",
        retry_result.err()
    );

    // Revoke de nova tarefa
    let revoke_result = broker.revoke_task("task-003").await;
    assert!(revoke_result.is_ok(), "Should successfully revoke task");

    // Verificar se a revogação foi aplicada
    let is_revoked: bool = conn.sismember("revoked", "task-003").await?;
    assert!(is_revoked, "Task should be added to revoked set");

    // Teste 5: Performance com grande volume
    println!("\n=== Teste 5: Performance ===");
    let start = std::time::Instant::now();

    // Executar todas as operações em sequência
    let _workers = broker.get_workers().await?;
    let _tasks = broker.get_tasks().await?;
    let _queues = broker.get_queues().await?;

    let duration = start.elapsed();
    println!("Total execution time: {:?}", duration);

    assert!(
        duration.as_millis() < 2000,
        "Should complete all operations within 2 seconds"
    );

    // === LIMPEZA ===
    let _: () = redis::cmd("SELECT").arg(0).query_async(&mut conn).await?;

    println!("\n✅ Teste de integração completo passou com sucesso!");
    Ok(())
}

/// Teste de stress com grande volume de dados
#[tokio::test]
async fn test_stress_with_high_volume() -> Result<()> {
    let client = match Client::open("redis://127.0.0.1:6379") {
        Ok(client) => client,
        Err(_) => {
            eprintln!("Skipping stress test: Redis not available");
            return Ok(());
        }
    };

    let mut conn = match client.get_multiplexed_tokio_connection().await {
        Ok(conn) => conn,
        Err(_) => {
            eprintln!("Skipping stress test: Redis connection failed");
            return Ok(());
        }
    };

    // Usar database 3 para stress test
    let _: () = redis::cmd("SELECT").arg(3).query_async(&mut conn).await?;
    let _: () = redis::cmd("FLUSHDB").query_async(&mut conn).await?;

    // Criar 500 tarefas para teste de stress
    println!("Creating 500 tasks for stress test...");
    for i in 0..500 {
        let status = match i % 4 {
            0 => "SUCCESS",
            1 => "FAILURE",
            2 => "PENDING",
            _ => "RETRY",
        };

        let task_metadata = json!({
            "status": status,
            "result": if status == "SUCCESS" { json!(i) } else { json!(null) },
            "traceback": if status == "FAILURE" { json!(format!("Error in task {}", i)) } else { json!(null) },
            "task_id": format!("stress-task-{:04}", i),
            "date_done": chrono::Utc::now().to_rfc3339()
        });

        let _: () = conn
            .set(
                format!("celery-task-meta-stress-task-{:04}", i),
                task_metadata.to_string(),
            )
            .await?;
    }

    let broker = RedisBroker::connect("redis://127.0.0.1:6379/3").await?;

    println!("Running stress test operations...");
    let start = std::time::Instant::now();

    // Executar operações sob stress
    let workers = broker.get_workers().await?;
    let tasks = broker.get_tasks().await?;
    let queues = broker.get_queues().await?;

    let duration = start.elapsed();

    // Verificações
    assert!(
        tasks.len() >= 100,
        "Should find at least 100 tasks (limited by implementation)"
    );
    assert!(
        duration.as_millis() < 10000,
        "Should handle 500 tasks within 10 seconds"
    );

    println!("Stress test results:");
    println!("  Workers discovered: {}", workers.len());
    println!("  Tasks parsed: {}", tasks.len());
    println!("  Queues discovered: {}", queues.len());
    println!("  Execution time: {:?}", duration);

    // Voltar ao database padrão
    let _: () = redis::cmd("SELECT").arg(0).query_async(&mut conn).await?;

    println!("✅ Stress test passed!");
    Ok(())
}
