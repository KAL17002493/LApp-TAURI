const { invoke } = window.__TAURI__.tauri;

// Add a word to DB
async function addWord(english_word, german_word) {
    //console.log("Adding word:", { english_word, german_word });  // Debugging line
    return await invoke("add_word", { englishWord: english_word, germanWord: german_word });
}

// Delete word from DB
async function deleteWord(id){ // Function to delete word by id
    await invoke("delete_word", {id});
    await Promise.all([fetchWords(), fetchWordCount()]);
}

// Function to update the word in the database
async function updateWord(id, englishWord, germanWord, listItem) {
    await invoke('update_word', {
        word: {
            id: id,
            english_word: englishWord,
            german_word: germanWord}
    }).then(() => {
      console.log("Word updated successfully");

      revertToViewMode(listItem, {
        id: id,
        english_word: englishWord,
        german_word: germanWord
      });
    })
    .catch((error) => {
      console.error("Error updating word:", error);
      alert("Failed to update word");
    });
  }
  

// Fetch a word and run the initial displayWord function
async function fetchWords() {
    try {
        const words = await invoke('get_words');  // Call the Tauri command to get words
        displayWords(words);  //Pass word to display function
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

/* -=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=- */
/* -=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=- */
if (document.getElementsByClassName("whole-content-container-practice")[0])
{
let correctWordId = null;
let randomWord = "";

async function fetchRandomWord() {
    try {
        randomWord = await invoke('get_random_word');  // Call the Tauri command to get words
        correctWordId = randomWord.id;
        displayRandomWord(randomWord.english_word);  // Display the fetched words
    } catch (error) {
        console.error('Error fetching words:', error);
    }
}

function displayRandomWord(randomWord){
    document.getElementsByClassName("word-to-guess")[0].innerHTML = randomWord;
}

// Handle form submission and check the user's guess
document.getElementsByClassName('word-practice-form')[0].addEventListener('submit', async function(event) {
    event.preventDefault();  // Prevent form submission and page reload
    
    // Get the user's guess from the input field
    const guess = document.getElementsByClassName('users-guess')[0].value.trim();

    // Call the submitGuess function with the user's guess and the correct word ID
    try {
        const response = await submitGuess(guess, correctWordId);  // Send guess to backend
        document.getElementsByClassName('users-guess')[0].value = "";  // Clear input field

        if (response === "Correct!") {
            // Show success feedback
            document.getElementsByClassName('word-guess-response')[0].innerHTML = "Correct! Well done!";
            borderColourChange("#08ff291f");
        } else {
            // Show incorrect feedback
            document.getElementsByClassName('word-guess-response')[0].innerHTML = `Guess: ${guess}<br>Answer: ${randomWord.german_word}`;
            console.log(randomWord.german_word)

            borderColourChange("#88111141");
        }

        // Optionally fetch and display a new word after each guess
        await fetchRandomWord();
    } catch (error) {
        console.error("Error submitting guess:", error);
    }
});


function borderColourChange(hexColour) { // Checks colour of the border when a guess is made
    const element = document.getElementsByClassName("whole-content-container-practice")[0];
    
    //Apply the border color change
    element.style.boxShadow = `inset 0 0 0 10px ${hexColour}`;
    
    //Clear any previous timeout to prevent multiple timeouts from executing
    clearTimeout(element.timeoutId);
    
    //Set a new timeout to remove the box shadow after some seconds
    element.timeoutId = setTimeout(() => {
        element.style.boxShadow = null;
    }, 2000);
}

// Function to submit guess to the backend
async function submitGuess(guess, correctWordId) {
    const response = await invoke("process_guess", { guess, correctWordId });
    return response;
}

//Run the fetchRandomWord function
document.addEventListener('DOMContentLoaded', async () => {
    document.getElementsByClassName("users-guess")[0].focus(); //Auto click on the input field
    await fetchRandomWord();
});
}
/* -=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=- */
/* -=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=- */

// Display the fetched words in the DOM for initially item display (revert function further down reloads individual items)
function displayWords(words) {
    const wordsContainer = document.querySelector('#words');
    wordsContainer.innerHTML = '';  // Clear any existing content

    words.forEach(word => {
        // Create a list item for each word
        const listItem = document.createElement('li');
        listItem.classList.add('word-item');
        listItem.id = word.id;  // Set the id for the list item

        // Create the English word paragraph
        const englishWordElement = document.createElement('p');
        englishWordElement.classList.add('english-word');
        englishWordElement.textContent = word.english_word;

        // Create the German word paragraph
        const germanWordElement = document.createElement('p');
        germanWordElement.classList.add('german-word');
        germanWordElement.textContent = word.german_word;

        // Create a link element (which will act as the "edit" trigger)
        const linkElement = document.createElement('a');
        linkElement.classList.add('word-link');
        linkElement.href = '#'; // Use '#' to avoid navigation

        // Add click event listener to the link
        linkElement.addEventListener('click', function (event) {
            event.preventDefault();  // Prevent navigation or default behavior
            // Switch to update mode (edit the current list item)
            switchToUpdateMode(listItem, word);
        });

        // Append the word details to the link
        linkElement.appendChild(englishWordElement);
        linkElement.appendChild(germanWordElement);

        // Create the delete button
        const deleteButton = document.createElement('button');
        deleteButton.textContent = 'Delete';
        deleteButton.onclick = function () {
            deleteWord(word.id, this);  // Handle deletion
        };

        // Append the link and delete button to the list item
        listItem.appendChild(linkElement);
        listItem.appendChild(deleteButton);

        // Append the list item to the container (ul element)
        wordsContainer.appendChild(listItem);
    });
}

// Switch the clicked list item to update mode
function switchToUpdateMode(listItem, word) {
    // Clear the list item's current content
    listItem.innerHTML = '';

    // Create input fields for editing
    const englishWordInput = document.createElement('input');
    englishWordInput.type = 'text';
    englishWordInput.value = word.english_word;
    englishWordInput.classList.add('english-word-update');

    const germanWordInput = document.createElement('input');
    germanWordInput.type = 'text';
    germanWordInput.value = word.german_word;
    germanWordInput.classList.add('german-word-update');

    // Create the update button
    const updateButton = document.createElement('button');
    updateButton.textContent = 'Update';
    updateButton.onclick = function () {
        clearCountdown(listItem); // Clear the timer when the update happens
        updateWord(word.id, englishWordInput.value, germanWordInput.value, listItem);  // Handle update
    };
    updateButton.classList.add("update-word-button");

    // Append inputs and buttons to the list item
    listItem.appendChild(englishWordInput);
    listItem.appendChild(germanWordInput);
    listItem.appendChild(updateButton);

    // Clear any existing timer for this list item
    clearCountdown(listItem);

    // Start the countdown for this specific list item
    startCountdown(listItem, word);
}

// Start a countdown timer for a specific list item
function startCountdown(listItem, word) {
    let countdown = 10;
    
    // Store the timer reference in the list item for future access
    const timer = setInterval(() => {
        countdown--;
        if (countdown <= 0) {
            clearInterval(timer);
            console.log(`Time's up for item ${word.id}!`);

            // Revert the list item back to view mode (i.e., exit edit mode)
            revertToViewMode(listItem, word);
        }
    }, 1000);

    // Save the timer reference in the listItem's dataset
    listItem.dataset.timer = timer;
}

// Clear any existing countdown timer for the list item
function clearCountdown(listItem) {
    const timer = listItem.dataset.timer;
    if (timer) {
        clearInterval(timer);  // Clear the specific timer
        delete listItem.dataset.timer;  // Remove the reference
    }
}

// Revert the list item back to view mode
function revertToViewMode(listItem, word) {
    // Clear the list item's content
    listItem.innerHTML = '';

    // Create the English word paragraph
    const englishWordElement = document.createElement('p');
    englishWordElement.classList.add('english-word');
    englishWordElement.textContent = word.english_word;

    // Create the German word paragraph
    const germanWordElement = document.createElement('p');
    germanWordElement.classList.add('german-word');
    germanWordElement.textContent = word.german_word;

    // Create a link element (which will act as the "edit" trigger)
    const linkElement = document.createElement('a');
    linkElement.classList.add('word-link');
    linkElement.href = '#'; // Use '#' to avoid navigation

    // Add click event listener to the link
    linkElement.addEventListener('click', function (event) {
        event.preventDefault();  // Prevent navigation or default behavior
        // Switch to update mode (edit the current list item)
        switchToUpdateMode(listItem, word);
    });

    // Append the word details to the link
    linkElement.appendChild(englishWordElement);
    linkElement.appendChild(germanWordElement);

    // Create the delete button
    const deleteButton = document.createElement('button');
    deleteButton.textContent = 'Delete';
    deleteButton.onclick = function () {
        deleteWord(word.id, this);  // Handle deletion
    };

    // Append the link and delete button to the list item
    listItem.appendChild(linkElement);
    listItem.appendChild(deleteButton);
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

const searchInput = document.getElementById('searchWord');
if (searchInput) {
    searchInput.addEventListener('input', function() {
        const searchValue = this.value.toLowerCase();
        const wordItems = document.querySelectorAll('.word-item');
        
        wordItems.forEach(function(item) {
            const englishWord = item.querySelector('.english-word').textContent.toLowerCase();
            const germanWord = item.querySelector('.german-word').textContent.toLowerCase();
            
            if (englishWord.includes(searchValue) || germanWord.includes(searchValue)) {
                item.style.display = '';
            } else {
                item.style.display = 'none';
            }
        });
    });

    document.getElementsByClassName("clear-search")[0].addEventListener("click", function() {
        //Clear the input field
        searchInput.value = '';
        //Create a new event and force trigger it (This unusual method of clearing the search field is needed because of how the search feature itself has been implemented)
        const event = new Event('input'); 
        searchInput.dispatchEvent(event);
    });
}