const { invoke } = window.__TAURI__.tauri;

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
