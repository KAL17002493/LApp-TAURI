// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use serde::{Deserialize, Serialize}; // For serializing and deserializing JSON data
use tauri::Manager; // Import Manager trait for manage function
use sqlx::FromRow;
use std::collections::VecDeque;
use tokio::sync::Mutex;

//DB connection + setup if needed
type Db = sqlx::Pool<sqlx::Sqlite>;

struct AppState {
    db: Db,
    recent_words_displayed: Mutex<VecDeque<Word>>, // Recent words
}

#[tokio::main]
async fn main() {
    let app = tauri::Builder::default()
        .setup(|app| {
            // Setup database and recent words list
            let db = futures::executor::block_on(setup_db(app));
            app.manage(AppState {
                db,
                recent_words_displayed: Mutex::new(VecDeque::new()), // Empty VecDeque initialized
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            add_word,
            get_words,
            db_word_count,
            delete_word,
            get_word_by_id,
            update_word,
            get_random_word,
            check_guess,
            process_guess,
            get_random_new_word,
            get_random_terribleat_word
        ])
        .build(tauri::generate_context!())
        .expect("error while building Tauri application");

    app.run(|_, _| {});
}

async fn setup_db(app: &tauri::App) -> Db {
    let mut path = app
        .path_resolver()
        .app_data_dir()
        .expect("could not get data_dir");

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

    let db = sqlx::sqlite::SqlitePoolOptions::new()
        .connect(path.to_str().unwrap())
        .await
        .expect("Failed to connect to the database");

    sqlx::migrate!("./migrations").run(&db).await.expect("Failed to run migrations");

    db
}

#[derive(Debug, Serialize, Deserialize, Clone, FromRow)]
struct Word {
    id: i32,               // Integer type for the primary key
    english_word: String,   // English word as text
    german_word: String,    // German word as text
    date_added: Option<String>,  // Date added (optional if you want)
}

#[derive(FromRow)]
struct UserWordPerformance {
    id: i32,
    fail_count: i32,
    word_id: i32,
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

#[tauri::command]
async fn get_random_word(state: tauri::State<'_, AppState>) -> Result<Word, String> {
    let db = &state.db;
    let recent_words_displayed = &state.recent_words_displayed;

    // Loop to keep fetching words until we find a new one
    let word = loop {
        let word: Word = sqlx::query_as::<_, Word>("SELECT * FROM word ORDER BY RANDOM() LIMIT 1")
            .fetch_one(db)
            .await
            .map_err(|e| format!("Failed to fetch random word: {}", e))?;

        // Lock the mutex to access recent words safely
        let recent_words = recent_words_displayed.lock().await;

        // Check if the word is already in the recent words list
        if !recent_words.iter().any(|w| w.id == word.id) {
            // Word is not recent, so break the loop and return this word
            break word;
        }
    };

    // Add the new word to the recent words list, ensuring the list size stays at 5
    let mut recent_words = recent_words_displayed.lock().await;
    recent_words.push_back(word.clone()); // Add to the back of the list
    if recent_words.len() > 5 {
        recent_words.pop_front(); // Remove the oldest word if list exceeds 5
    }

    Ok(word)
}

#[tauri::command]
async fn get_random_new_word(state: tauri::State<'_, AppState>) -> Result<Word, String> {
    let db = &state.db;
    let recent_words_displayed = &state.recent_words_displayed;

    let word = loop {
        let word: Word = sqlx::query_as::<_, Word>(
            "SELECT * FROM word 
            WHERE date_added >= DATE('now', '-6 days') 
            ORDER BY RANDOM() 
            LIMIT 1"
        )
        .fetch_one(db)
        .await
        .map_err(|e| format!("Failed to fetch random word: {}", e))?;

        // Lock the mutex and check against recent words
        let recent_words = recent_words_displayed.lock().await;
        if !recent_words.iter().any(|w| w.id == word.id) {
            break word;
        }
    };

    // Add the new word to the recent words list
    let mut recent_words = recent_words_displayed.lock().await;
    recent_words.push_back(word.clone());
    if recent_words.len() > 5 {
        recent_words.pop_front(); // Keep only the last 5 words
    }

    Ok(word)
}
/**/
#[tauri::command]
async fn get_random_terribleat_word(state: tauri::State<'_, AppState>) -> Result<Word, String> {
    let db = &state.db;
    let recent_words_displayed = &state.recent_words_displayed;

    // Loop to keep fetching words until we find a new one
    let word = loop {
        // Updated SQL query to join word and user_word_performance tables
        let word: Word = sqlx::query_as::<_, Word>(
            "
            SELECT w.* FROM word w
            JOIN user_word_performance u ON w.id = u.word_id
            WHERE u.fail_count > 0
            ORDER BY RANDOM() LIMIT 1
            "
        )
        .fetch_one(db)
        .await
        .map_err(|e| format!("Failed to fetch random word: {}", e))?;

        // Lock the mutex to access recent words safely
        let recent_words = recent_words_displayed.lock().await;

        // Check if the word is already in the recent words list
        if !recent_words.iter().any(|w| w.id == word.id) {
            // Word is not recent, so break the loop and return this word
            break word;
        }
    };

    // Add the new word to the recent words list, ensuring the list size stays at 5
    let mut recent_words = recent_words_displayed.lock().await;
    recent_words.push_back(word.clone()); // Add to the back of the list
    if recent_words.len() > 5 {
        recent_words.pop_front(); // Remove the oldest word if list exceeds 5
    }

    Ok(word)
}

#[tauri::command] // Check if user guess is correct
async fn check_guess(state: tauri::State<'_, AppState>, guess: String, correct_word_id: i32, practice_type: String, lan_displayed: String) -> Result<bool, String> {
    let db = &state.db;

    //Get the word from the database using its ID
    let word: Word = sqlx::query_as::<_, Word>("SELECT * FROM word WHERE id = ?")
        .bind(correct_word_id)
        .fetch_one(db)
        .await
        .map_err(|e| format!("Failed to fetch correct word: {}", e))?;

    //Remove brackets and their content (for English words)
    fn remove_parentheses(text: &str) -> String {
        let re = regex::Regex::new(r"\s*\(.*?\)").unwrap(); //Find brackets
        re.replace_all(text, "").to_string() //Remove brackets if found
    }

    //Remote white spaces from start and end + conver text to lowercase
    fn clean_word(word: &str) -> String {
        remove_parentheses(word).trim().to_lowercase() 
    }

    //Call clean_word function
    let cleaned_guess = clean_word(&guess);

    // Function to handle English-to-German comparison
    fn check_multiple_answers(answers: &str, guess: &str) -> bool {
        answers.split('/')
            .map(|part| clean_word(part))
            .any(|cleaned_answer| cleaned_answer == guess)
    }

    // Depending on the practice type or language displayed, check the answer
    match practice_type.as_str() {
        // Handle English to German practice
        "practice-english" => {
            let cleaned_german_word = clean_word(&word.german_word);
            Ok(cleaned_guess == cleaned_german_word)
        }
        // Handle German to English practice
        "practice-german" => Ok(check_multiple_answers(&word.english_word, &cleaned_guess)),
        
        // Handle mixed, new, and "terribleAt" where either German or English can be displayed
        "practice-mix" | "practice-new" | "practice-suckAt" => {
            if lan_displayed == "german" {
                // If German is displayed, compare with English guesses
                Ok(check_multiple_answers(&word.english_word, &cleaned_guess))
            } else {
                // If English is displayed, compare with German guesses
                let cleaned_german_word = clean_word(&word.german_word);
                Ok(cleaned_guess == cleaned_german_word)
            }
        }
        _ => Err("Invalid practice type.".to_string()),
    }
}

#[tauri::command] //Sends correct or incorrect guess response to frontend
async fn process_guess(state: tauri::State<'_, AppState>, guess: String, correct_word_id: i32, practice_type: String, lan_displayed: String) -> Result<String, String> {
    let is_correct = check_guess(state.clone(), guess, correct_word_id, practice_type, lan_displayed).await?;

    if is_correct {
        update_fail_count(state.clone(), correct_word_id, true).await?; //Call function to decrease fail_count
        Ok("Correct!".to_string())
    } else {
        update_fail_count(state.clone(), correct_word_id, false).await?; //Call function to increase fail_count
        Ok("Incorrect!".to_string())
    }
}

async fn update_fail_count(state: tauri::State<'_, AppState>, word_id: i32, is_correct: bool) -> Result<(), String> {
    let db = &state.db;

    // Fetch user performance in one go
    let mut performance = sqlx::query_as::<_, UserWordPerformance>("SELECT * FROM user_word_performance WHERE word_id = ?")
        .bind(word_id)
        .fetch_optional(db)
        .await
        .map_err(|e| format!("Failed to fetch user performance: {}", e))?;

    // Handle fail_count increment/decrement
    if is_correct {
        if let Some(ref mut performance) = performance {
            if performance.fail_count > 0 {
                performance.fail_count -= 1;
                // Update the fail_count in the database
                sqlx::query("UPDATE user_word_performance SET fail_count = ? WHERE word_id = ?")
                    .bind(performance.fail_count)
                    .bind(word_id)
                    .execute(db)
                    .await
                    .map_err(|e| format!("Failed to update fail count: {}", e))?;
            }
        }
    } else {
        if let Some(ref mut performance) = performance {
            if performance.fail_count < 10 {
                performance.fail_count += 1;
                // Update the fail_count in the database
                sqlx::query("UPDATE user_word_performance SET fail_count = ? WHERE word_id = ?")
                    .bind(performance.fail_count)
                    .bind(word_id)
                    .execute(db)
                    .await
                    .map_err(|e| format!("Failed to update fail count: {}", e))?;
            }
        } else {
            // If no entry exists, create a new one with fail_count = 1
            sqlx::query("INSERT INTO user_word_performance (fail_count, word_id) VALUES (?, ?)")
                .bind(1)
                .bind(word_id)
                .execute(db)
                .await
                .map_err(|e| format!("Failed to insert new user performance entry: {}", e))?;
        }
    }

    Ok(())
}