const { invoke } = window.__TAURI__.tauri;

// Add a word
async function addWord(english_word, german_word) {
    console.log("Adding word:", { english_word, german_word }); // Debugging line
    return await invoke("add_word", { englishWord: english_word, germanWord: german_word });
}

// Fetch and display words
async function fetchWords() {
    try {
        const words = await invoke('get_words');  // Call the Tauri command
        displayWords(words);  // Display the fetched words
    } catch (error) {
        console.error('Error fetching words:', error);
    }
}

// Display the fetched words in the DOM
function displayWords(words) {
    const wordsContainer = document.querySelector('#words');
    wordsContainer.innerHTML = '';  // Clear any existing content

    words.forEach(word => {
        const wordElement = document.createElement('div');
        wordElement.classList.add('word-item');  // Add class for styling
        wordElement.textContent = `${word.english_word} - ${word.german_word} (Added on: ${word.date_added})`;
        wordsContainer.appendChild(wordElement);
    });
}

// Handle form submission
document.querySelector("#word-form").addEventListener("submit", async (event) => {
    event.preventDefault();
    let englishInput = document.querySelector("#english-word-input");
    let germanInput = document.querySelector("#german-word-input");

    try {
        await addWord(englishInput.value, germanInput.value);
        englishInput.value = "";
        germanInput.value = "";

        // Fetch and display the updated list of words after adding
        await fetchWords();
    } catch (error) {
        console.error("Error adding word:", error);
    }
});

// Fetch words when the page loads
document.addEventListener('DOMContentLoaded', fetchWords);
















/*
async function addTodo(description) {
    return await invoke("add_todo", { description });
}

async function getTodos() {
    return await invoke("get_todos");
}

async function updateTodo(todo) {
    return await invoke("update_todo", { todo });
}

async function deleteTodo(id) {
    return await invoke("delete_todo", { id });
}

document.querySelector("#todo-form").addEventListener("submit", (event) => {
  event.preventDefault();
  let input = document.querySelector("#todo-input");
  addTodo(input.value).then(() => {
      buildTodoList();
  });
  input.value = "";
});

async function buildTodoList() {
  let todos = await getTodos();
  let tasksContainer = document.querySelector("#tasks");
  tasksContainer.innerHTML = "";

  todos.forEach((todo) => {
      let div = document.createElement("div");
      div.innerHTML = `
          <label>
              <input type="checkbox" data-id="${todo.id}" ${todo.status === 'Complete' ? 'checked' : ''}>
              ${todo.description}
          </label>
          <button class="delete" data-id="${todo.id}">Delete</button>
      `;
      tasksContainer.appendChild(div);
  });

  // Handle checkbox updates for status
  document.querySelectorAll("input[type=checkbox]").forEach((checkbox) => {
      checkbox.addEventListener("change", async (event) => {
          let todoId = event.target.getAttribute("data-id");
          let todo = todos.find(t => t.id == todoId);
          todo.status = event.target.checked ? "Complete" : "Incomplete";
          await updateTodo(todo);
          buildTodoList();
      });
  });

  // Handle delete functionality
  document.querySelectorAll(".delete").forEach((button) => {
      button.addEventListener("click", async (event) => {
          let id = event.target.getAttribute("data-id");
          await deleteTodo(parseInt(id));
          buildTodoList();
      });
  });
}

buildTodoList();
*/