// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use serde::{Deserialize, Serialize}; // For serializing and deserializing JSON data
//use sqlx::sqlite::SqlitePoolOptions;
//use sqlx::{Pool, Sqlite}; // For database handling
use futures::TryStreamExt; // For streaming query results
use tauri::Manager; // Import Manager trait for manage function

#[derive(Debug, Serialize, Deserialize, sqlx::Type)]
enum TodoStatus {
    Incomplete,
    Complete,
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
struct Todo {
    id: u16,
    description: String,
    status: TodoStatus,
}

//DB connection + setup if needed
type Db = sqlx::Pool<sqlx::Sqlite>;

struct AppState {
    db: Db,
}

async fn setup_db(app: &tauri::App) -> Db {
    let mut path = app
        .path_resolver()
        .app_data_dir()
        .expect("could not get data_dir");

    // Create the directory if it doesn't exist
    std::fs::create_dir_all(path.clone()).expect("Failed to create app data directory");

    path.push("db.sqlite");
    let result = std::fs::OpenOptions::new().create_new(true).write(true).open(&path);
    match result {
        Ok(_) => println!("Database file created"),
        Err(err) => match err.kind() {
            std::io::ErrorKind::AlreadyExists => println!("Database file already exists"),
            _ => panic!("Error creating database file: {}", err),
        },
    }

    // Open SQLite connection pool
    let db = sqlx::sqlite::SqlitePoolOptions::new()
        .connect(path.to_str().unwrap())
        .await
        .expect("Failed to connect to the database");

    // Run migrations
    sqlx::migrate!("./migrations").run(&db).await.expect("Failed to run migrations");

    db
}

#[tauri::command]
async fn add_todo(
    state: tauri::State<'_, AppState>, 
    description: &str
) -> Result<(), String> {
    let db = &state.db;

    sqlx::query("INSERT INTO todos (description, status) VALUES (?1, ?2)")
        .bind(description)
        .bind(TodoStatus::Incomplete) // Default status is Incomplete
        .execute(db)
        .await
        .map_err(|e| format!("Error saving todo: {}", e))?;

    Ok(())
}

#[tauri::command]
async fn get_todos(state: tauri::State<'_, AppState>) -> Result<Vec<Todo>, String> {
    let db = &state.db;

    let todos: Vec<Todo> = sqlx::query_as::<_, Todo>("SELECT * FROM todos")
        .fetch(db)
        .try_collect()
        .await
        .map_err(|e| format!("Failed to get todos: {}", e))?;

    Ok(todos)
}

#[tauri::command]
async fn update_todo(
    state: tauri::State<'_, AppState>, 
    todo: Todo
) -> Result<(), String> {
    let db = &state.db;

    sqlx::query("UPDATE todos SET description = ?1, status = ?2 WHERE id = ?3")
        .bind(todo.description)
        .bind(todo.status)
        .bind(todo.id)
        .execute(db)
        .await
        .map_err(|e| format!("Could not update todo: {}", e))?;

    Ok(())
}

#[tauri::command]
async fn delete_todo(state: tauri::State<'_, AppState>, id: u16) -> Result<(), String> {
    let db = &state.db;

    sqlx::query("DELETE FROM todos WHERE id = ?1")
        .bind(id)
        .execute(db)
        .await
        .map_err(|e| format!("Could not delete todo: {}", e))?;

    Ok(())
}

#[tokio::main]
async fn main() {
    let app = tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            add_todo,
            get_todos,
            update_todo,
            delete_todo
        ])
        .build(tauri::generate_context!())
        .expect("error while building Tauri application");

    let db = setup_db(&app).await;
    app.manage(AppState { db });

    app.run(|_, _| {});
}
