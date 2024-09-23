const { invoke } = window.__TAURI__.tauri;

// Add a word
async function addWord(english_word, german_word) {
    //console.log("Adding word:", { english_word, german_word });  // Debugging line
    return await invoke("add_word", { englishWord: english_word, germanWord: german_word });
}

async function deleteWord(id){ // Function to delete word by id
    await invoke("delete_word", {id});

    await fetchWords();  // Fetch and display the updated list of words
    await fetchWordCount();  // Fetch and display the updated word count
}

/*async function fetchWordById(id){
    try {
        const word = await invoke('get_word_by_id', {id});
        console.log(word);  // Display the fetched words
    } catch (error) {
        console.error('Error fetching word by id:', error);
    }
}*/

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
        updateWord(word.id, englishWordInput.value, germanWordInput.value, listItem);  // Handle update
    };
    updateButton.classList.add("update-word-button");
    
    // Append inputs and buttons to the list item
    listItem.appendChild(englishWordInput);
    listItem.appendChild(germanWordInput);
    listItem.appendChild(updateButton);
}

/*document.getElementsByClassName("right-container")[0].addEventListener(
    "click", () => {
        // Accessing the first element in the HTMLCollection
        document.getElementsByClassName("word-list")[0].hidden = true;
        document.getElementsByClassName("word-update")[0].hidden = false;
    }, false);*/

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

const searchWordErrorRemove = document.getElementById('searchWord'); //Checks if the input field exists on the page (Removes errors from devtool screen)
if (searchWordErrorRemove)
{
    document.getElementById('searchWord').addEventListener('input', function() {
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
}






