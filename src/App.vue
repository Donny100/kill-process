<script setup lang="ts">
import { ref, reactive } from "vue";
import { invoke } from "@tauri-apps/api/core";

interface ProcessInfo {
  pid: string;
  name: string;
  port: string;
}

interface PortCheckResult {
  is_occupied: boolean;
  processes: ProcessInfo[];
  error?: string;
}

const port = ref("");
const loading = ref(false);
const result = reactive<PortCheckResult>({
  is_occupied: false,
  processes: [],
});
const message = ref("");

// Check port occupation
async function checkPort() {
  if (!port.value) {
    message.value = "Please enter a port number";
    return;
  }

  loading.value = true;
  message.value = "";

  try {
    const response = await invoke<PortCheckResult>("check_port", { port: port.value.toString() });
    Object.assign(result, response);
    
    if (response.error) {
      message.value = `Error: ${response.error}`;
    } else if (!response.is_occupied) {
      message.value = `Port ${port.value} is not occupied`;
    } else {
      message.value = `Port ${port.value} is occupied by ${response.processes.length} process(es)`;
    }
  } catch (error) {
    message.value = `Query failed: ${error}`;
  } finally {
    loading.value = false;
  }
}

// Kill process
async function killProcess(pid: string, name: string) {
  try {
    const response = await invoke<string>("kill_process", { pid });
    message.value = `Successfully killed process ${name} (PID: ${pid})`;
    
    // Refresh port check after killing process
    await checkPort();
  } catch (error) {
    message.value = `Failed to kill process: ${error}`;
  }
}

// Handle Enter key press
function handleKeyPress(event: KeyboardEvent) {
  if (event.key === "Enter") {
    checkPort();
  }
}
</script>

<template>
  <div class="min-h-screen bg-gradient-to-br from-blue-50 to-indigo-100 p-4">
    <div class="max-w-2xl mx-auto">
      <!-- Header -->
      <div class="text-center mb-8">
        <h1 class="text-4xl font-bold text-gray-800 mb-2">KillProcess</h1>
        <p class="text-gray-600">Detect and kill processes occupying ports</p>
      </div>

      <!-- Port Input Section -->
      <div class="bg-white rounded-xl shadow-lg p-6 mb-6">
        <div class="flex gap-3">
          <input
            v-model="port"
            @keypress="handleKeyPress"
            type="number"
            placeholder="Enter port number (e.g., 3000)"
            class="flex-1 px-4 py-3 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-transparent outline-none transition-all"
          />
          <button
            @click="checkPort"
            :disabled="loading"
            class="px-6 py-3 bg-blue-600 text-white rounded-lg hover:bg-blue-700 disabled:opacity-50 disabled:cursor-not-allowed transition-colors font-medium"
          >
            <span v-if="loading" class="flex items-center gap-2">
              <svg class="animate-spin h-4 w-4" viewBox="0 0 24 24">
                <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4" fill="none"></circle>
                <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
              </svg>
              Querying...
            </span>
            <span v-else>Check Port</span>
          </button>
        </div>
      </div>

      <!-- Message Display -->
      <div v-if="message" class="mb-6">
        <div class="bg-white rounded-lg p-4 shadow-md">
          <p class="text-gray-700">{{ message }}</p>
        </div>
      </div>

      <!-- Process List -->
      <div v-if="result.processes.length > 0" class="bg-white rounded-xl shadow-lg overflow-hidden">
        <div class="bg-gray-50 px-6 py-4 border-b">
          <h2 class="text-lg font-semibold text-gray-800">Occupying Processes</h2>
        </div>
        
        <div class="divide-y divide-gray-200">
          <div
            v-for="process in result.processes"
            :key="process.pid"
            class="px-6 py-4 hover:bg-gray-50 transition-colors"
          >
            <div class="flex items-center justify-between">
              <div class="flex-1">
                <div class="flex items-center gap-3">
                  <div class="w-2 h-2 bg-red-500 rounded-full"></div>
                  <div>
                    <p class="font-medium text-gray-900">{{ process.name }}</p>
                    <p class="text-sm text-gray-500">PID: {{ process.pid }} | 端口: {{ process.port }}</p>
                  </div>
                </div>
              </div>
              
                              <button
                  @click="killProcess(process.pid, process.name)"
                  class="px-4 py-2 bg-red-600 text-white rounded-lg hover:bg-red-700 transition-colors text-sm font-medium"
                >
                  Kill Process
                </button>
            </div>
          </div>
        </div>
      </div>

      <!-- Empty State -->
      <div v-else-if="!loading && port && !result.is_occupied" class="bg-white rounded-xl shadow-lg p-8 text-center">
        <div class="w-16 h-16 bg-green-100 rounded-full flex items-center justify-center mx-auto mb-4">
          <svg class="w-8 h-8 text-green-600" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 13l4 4L19 7"></path>
          </svg>
        </div>
        <h3 class="text-lg font-semibold text-gray-800 mb-2">Port Available</h3>
        <p class="text-gray-600">Port {{ port }} is not occupied by any process</p>
      </div>
    </div>
  </div>
</template>


