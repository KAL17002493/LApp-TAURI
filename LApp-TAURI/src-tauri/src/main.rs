// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use serde::{Deserialize, Serialize}; // For serializing and deserializing JSON data
//use sqlx::sqlite::SqlitePoolOptions;
//use sqlx::{Pool, Sqlite}; // For database handling
use futures::TryStreamExt; // For streaming query results
use tauri::Manager; // Import Manager trait for manage function

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

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
struct Word {
    id: i32,               // Integer type for the primary key
    english_word: String,   // English word as text
    german_word: String,    // German word as text
    date_added: Option<String>,  // Date added (optional if you want)
}

#[tauri::command] //Get word count
async fn db_word_count(state: tauri::State<'_, AppState>) -> Result<i64, String> {
    let db = &state.db;

    // Query to count the total number of words
    let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM word")
        .fetch_one(db)
        .await
        .map_err(|e| format!("Failed to get word count: {}", e))?;

    Ok(count.0)  // Return the count
}

#[tauri::command] //Add a new word to the database
async fn add_word(
    state: tauri::State<'_, AppState>, 
    english_word: String,
    german_word: String,
) -> Result<(), String> {
    let db = &state.db;

    sqlx::query(
        "INSERT INTO word (english_word, german_word) VALUES (?1, ?2)"
    )
    .bind(english_word)
    .bind(german_word)
    .execute(db)
    .await
    .map_err(|e| format!("Error adding word: {}", e))?;

    Ok(())
}

#[tauri::command] //Get all words from the database, newest word added displayed first
async fn get_words(state: tauri::State<'_, AppState>) -> Result<Vec<Word>, String> {
    let db = &state.db;

    let words: Vec<Word> = sqlx::query_as::<_, Word>("SELECT * FROM word ORDER BY date_added DESC")
        .fetch_all(db) // `fetch_all` instead of `fetch`
        .await
        .map_err(|e| format!("Failed to get words: {}", e))?;
    
    Ok(words)
}

#[tauri::command] //Get single word by Id
async fn get_word_by_id(state: tauri::State<'_, AppState>) -> Result<Vec<Word>, String> {
    let db = &state.db;

    let words: Vec<Word> = sqlx::query_as::<_, Word>("SELECT * FROM word WHERE id = ?")
        .fetch_all(db) // `fetch_all` instead of `fetch`
        .await
        .map_err(|e| format!("Failed to get words: {}", e))?;
    
    Ok(words)
}

#[tauri::command]
async fn delete_word(state: tauri::State<'_, AppState>, id: u16) -> Result<(), String> {
    let db = &state.db;

    sqlx::query("DELETE FROM word WHERE id = ?1")
        .bind(id)
        .execute(db)
        .await
        .map_err(|e| format!("Could not delete word: {}", e))?;

    Ok(())
}

/*#[tauri::command]
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
}*/

#[tokio::main]
async fn main() {
    let app = tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            add_word,
            get_words,
            db_word_count,
            delete_word,
            get_word_by_id
        ])
        .build(tauri::generate_context!())
        .expect("error while building Tauri application");

    let db = setup_db(&app).await;
    app.manage(AppState { db });

    app.run(|_, _| {});
}
