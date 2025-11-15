const { invoke } = window.__TAURI__.core;

let serverIpEl;
let portEl;
let statusEl;
let modeEl;
let connectBtn;
let disconnectBtn;

async function connect() {
  const serverIp = serverIpEl.value;
  const port = parseInt(portEl.value);

  try {
    const result = await invoke("connect_to_server", { serverIp, port });
    statusEl.textContent = `✅ ${result}`;
    statusEl.className = "status connected";
    connectBtn.disabled = true;
    disconnectBtn.disabled = false;
    updateState();
  } catch (error) {
    statusEl.textContent = `❌ ${error}`;
    statusEl.className = "status error";
  }
}

async function disconnect() {
  try {
    await invoke("disconnect_from_server");
    statusEl.textContent = "⭕ Disconnected";
    statusEl.className = "status disconnected";
    connectBtn.disabled = false;
    disconnectBtn.disabled = true;
    updateState();
  } catch (error) {
    statusEl.textContent = `❌ ${error}`;
    statusEl.className = "status error";
  }
}

async function toggleMode() {
  try {
    const newMode = await invoke("toggle_mode");
    updateState();
  } catch (error) {
    console.error("Failed to toggle mode:", error);
  }
}

async function updateState() {
  try {
    const state = await invoke("get_state");
    const stateObj = JSON.parse(state);

    if (stateObj.mode === "MacOS") {
      modeEl.textContent = "🖥️ macOS Mode";
      modeEl.className = "mode macos";
    } else {
      modeEl.textContent = "🪟 Windows Mode";
      modeEl.className = "mode windows";
    }
  } catch (error) {
    console.error("Failed to get state:", error);
  }
}

window.addEventListener("DOMContentLoaded", () => {
  serverIpEl = document.querySelector("#server-ip");
  portEl = document.querySelector("#port");
  statusEl = document.querySelector("#status");
  modeEl = document.querySelector("#mode");
  connectBtn = document.querySelector("#connect-btn");
  disconnectBtn = document.querySelector("#disconnect-btn");

  connectBtn.addEventListener("click", connect);
  disconnectBtn.addEventListener("click", disconnect);

  // Update state every 2 seconds
  setInterval(updateState, 2000);
  updateState();
});
