// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use serde::{Deserialize, Serialize}; // For serializing and deserializing JSON data
//use sqlx::sqlite::SqlitePoolOptions;
//use sqlx::{Pool, Sqlite}; // For database handling
//use futures::TryStreamExt; // For streaming query results
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

    //let wordExists: bool = sqlx::query("SELECT * FROM word english_word WHERE english_word = english_word")

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

#[tauri::command]
async fn get_word_by_id(state: tauri::State<'_, AppState>, id: i32) -> Result<Word, String> {
    let db = &state.db;

    let word: Word = sqlx::query_as::<_, Word>("SELECT * FROM word WHERE id = ?")
        .bind(id)
        .fetch_one(db)
        .await
        .map_err(|e| format!("Failed to get word by id: {}", e))?;
    
    Ok(word)
}

#[tauri::command]
async fn delete_word(state: tauri::State<'_, AppState>, id: i32) -> Result<(), String> {
    let db = &state.db;

    sqlx::query("DELETE FROM word WHERE id = ?1")
        .bind(id)
        .execute(db)
        .await
        .map_err(|e| format!("Could not delete word: {}", e))?;

    Ok(())
}

#[tauri::command]
async fn update_word(
    state: tauri::State<'_, AppState>, 
    word: Word
) -> Result<(), String> {
    let db = &state.db;

    sqlx::query("UPDATE word SET english_word = ?1, german_word = ?2 WHERE id = ?3")
        .bind(&word.english_word)
        .bind(&word.german_word)
        .bind(word.id)  // Don't forget to bind the `id` as the 3rd parameter
        .execute(db)
        .await
        .map_err(|e| format!("Could not update word: {}", e))?;

    Ok(())
}

#[tauri::command] //Get 1 random word from DB
async fn get_random_word(state: tauri::State<'_, AppState>) -> Result<Word, String> {
    let db = &state.db;

    let word: Word = sqlx::query_as::<_, Word>("SELECT * FROM word ORDER BY RANDOM() LIMIT 1")
        .fetch_one(db)
        .await
        .map_err(|e| format!("Failed to fetch random word: {}", e))?;
    
    Ok(word)
}

#[tauri::command] //Get 1 random word from DB added in last 6 days
async fn get_random_new_word(state: tauri::State<'_, AppState>) -> Result<Word, String> {
    let db = &state.db;

    let word: Word = sqlx::query_as::<_, Word>(
        "SELECT * FROM word 
         WHERE date_added >= DATE('now', '-6 days')
         ORDER BY RANDOM() 
         LIMIT 1"
    )
    .fetch_one(db)
    .await
    .map_err(|e| format!("Failed to fetch random word: {}", e))?;
    
    Ok(word)
}

#[tauri::command] //Prevent word repetition
async fn get_next_word(state: tauri::State<'_, AppState>, recent_words: Vec<i32>) -> Result<Word, String> {
    let db = &state.db;

    let query = format!(
        "SELECT * FROM word WHERE id NOT IN ({}) ORDER BY RANDOM() LIMIT 1",
        recent_words.iter().map(|id| id.to_string()).collect::<Vec<_>>().join(", ")
    );

    let word: Word = sqlx::query_as::<_, Word>(&query)
        .fetch_one(db)
        .await
        .map_err(|e| format!("Failed to fetch random word: {}", e))?;
    
    Ok(word)
}

#[tauri::command] //Check if user guess is correct
async fn check_guess(state: tauri::State<'_, AppState>, guess: String, correct_word_id: i32, practice_type: String) -> Result<bool, String> {
    let db = &state.db;

    // Function to remove parentheses and their content
    fn remove_parentheses(text: &str) -> String {
        let re = regex::Regex::new(r"\s*\(.*?\)").unwrap(); //Find brackets
        re.replace_all(text, "").to_string() //Remove brackets if found
    }

    //Get the correct word from the database to check agains using it's id
    let word: Word = sqlx::query_as::<_, Word>("SELECT * FROM word WHERE id = ?")
        .bind(correct_word_id)
        .fetch_one(db)
        .await
        .map_err(|e| format!("Failed to fetch correct word: {}", e))?;

    //Clean the user's guess by removing content inside parentheses and trimming
    let cleaned_guess = remove_parentheses(&guess).trim().to_lowercase();

    if practice_type == "practice-english" {
        //Clean the German word before comparing
        let cleaned_german_word = remove_parentheses(&word.german_word).trim().to_lowercase();

        //Compare the cleaned guess with the cleaned German word
        if cleaned_guess == cleaned_german_word {
            Ok(true)  //Guess is correct
        } else {
            Ok(false)  //Guess is incorrect
        }
    }
    else if practice_type == "practice-german"{
        // Handle multiple correct answers for English to German
        let correct_answers: Vec<String> = word.english_word.split('/')
            .map(|part| remove_parentheses(part).trim().to_lowercase()) // Clean each answer
            .collect();

        // Check if the cleaned guess matches any of the cleaned correct answers
        if correct_answers.contains(&cleaned_guess) {
            Ok(true)  // Guess is correct
        } else {
            Ok(false)  // Guess is incorrect
        }
    }
    else{
        Ok(false)
    }
}


#[tauri::command] //Sends correct or incorrect guess to frontend
async fn process_guess(
    state: tauri::State<'_, AppState>,
    guess: String,
    correct_word_id: i32,
    practice_type: String  //It does nothing here but it it's not here the app will creash since check guess is sending 4 arguments
) -> Result<String, String> {
    let is_correct = check_guess(state.clone(), guess, correct_word_id, practice_type).await?;

    if is_correct {
        Ok("Correct!".to_string())
    } else {
        Ok("Incorrect!".to_string())
    }
}

#[tokio::main]
async fn main() {
    let app = tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            add_word,
            get_words,
            db_word_count,
            delete_word,
            get_word_by_id,
            update_word,
            get_random_word,
            get_next_word,
            check_guess,
            process_guess,
            get_random_new_word
        ])
        .build(tauri::generate_context!())
        .expect("error while building Tauri application");

    let db = setup_db(&app).await;
    app.manage(AppState { db });

    app.run(|_, _| {});
}