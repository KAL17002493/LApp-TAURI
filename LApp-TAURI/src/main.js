const { invoke } = window.__TAURI__.tauri;

// Add a word
async function addWord(english_word, german_word) {
    //console.log("Adding word:", { english_word, german_word });  // Debugging line
    return await invoke("add_word", { englishWord: english_word, germanWord: german_word });
}

// Fetch and display words
async function fetchWords() {
    try {
        const words = await invoke('get_words');  // Call the Tauri command to get words
        displayWords(words);  // Display the fetched words
    } catch (error) {
        console.error('Error fetching words:', error);
    }
}

// Fetch the total word count
async function fetchWordCount() {
    try {
        const count = await invoke('db_word_count');  // Call the Tauri command for word count
        displayWordCount(count);  // Display the total count
    } catch (error) {
        console.error('Error fetching word count:', error);
    }
}

// Display the fetched words in the DOM
function displayWords(words) {
    const wordsContainer = document.querySelector('#words');
    wordsContainer.innerHTML = '';  // Clear any existing content

    words.forEach(word => {
        // Create a list item for each word
        const listItem = document.createElement('li');
        listItem.classList.add('word-item');
        listItem.id = word.id;  // Set the id for the list item

        // Create a link element
        const linkElement = document.createElement('a');
        linkElement.classList.add('word-link');
        linkElement.href = `/edit-word/${word.id}`;  // Link to edit the word

        // Create the English word paragraph
        const englishWordElement = document.createElement('p');
        englishWordElement.classList.add('english-word');
        englishWordElement.textContent = word.english_word;

        // Create the German word paragraph
        const germanWordElement = document.createElement('p');
        germanWordElement.classList.add('german-word');
        germanWordElement.textContent = word.german_word;

        // Append the word details to the link
        linkElement.appendChild(englishWordElement);
        linkElement.appendChild(germanWordElement);

        // Create the delete button
        const deleteButton = document.createElement('button');
        deleteButton.textContent = 'Delete';
        deleteButton.onclick = function () {
            markForDeletion(word.id, this);  // Handle deletion
        };

        // Append the link and delete button to the list item
        listItem.appendChild(linkElement);
        listItem.appendChild(deleteButton);

        // Append the list item to the container (ul element)
        wordsContainer.appendChild(listItem);
    });
}

// Display the word count in the DOM
function displayWordCount(count) {
    const wordCountElement = document.querySelector('#word-count');
    wordCountElement.textContent = `${count}`;
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

        await fetchWords();  // Fetch and display the updated list of words
        await fetchWordCount();  // Fetch and display the updated word count
    } catch (error) {
        console.error("Error adding word:", error);
    }
});

// Fetch words and word count when the page loads
document.addEventListener('DOMContentLoaded', async () => {
    await fetchWords();  // Fetch and display the words
    await fetchWordCount();  // Fetch and display the total word count
});
















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