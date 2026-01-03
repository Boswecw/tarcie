import { invoke } from "@tauri-apps/api/core";
import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow";

const win = getCurrentWebviewWindow();

const input = document.querySelector<HTMLInputElement>("#tarcie-input")!;
const markerBtn = document.querySelector<HTMLButtonElement>("#tarcie-marker")!;
const statusEl = document.querySelector<HTMLDivElement>("#tarcie-status")!;

function flashCaptured() {
  document.body.classList.add("captured");
  statusEl.textContent = "Captured";
  setTimeout(async () => {
    document.body.classList.remove("captured");
    statusEl.textContent = "";
    await win.hide();
    input.value = "";
  }, 200);
}

async function captureNote() {
  const content = input.value || "";
  await invoke("capture_note", { content });
  flashCaptured();
}

async function captureMarker() {
  await invoke("capture_marker", { reason: null });
  flashCaptured();
}

input.addEventListener("keydown", (e) => {
  if (e.key === "Enter") {
    e.preventDefault();
    captureNote().catch(console.error);
  } else if (e.key === "Escape") {
    win.hide().catch(console.error);
  }
});

markerBtn.addEventListener("click", () => {
  captureMarker().catch(console.error);
});

input.focus();
