<script setup lang="ts">
import { ref, reactive } from "vue";
import { invoke } from "@tauri-apps/api/core";

interface ProcessInfo {
  pid: string;
  name: string;
  port: string;
}

interface ProcessDetail {
  pid: string;
  name: string;
  port: string;
  user?: string;
  command?: string;
  cpu_usage?: string;
  memory_usage?: string;
  start_time?: string;
}

interface PortCheckResult {
  is_occupied: boolean;
  processes: ProcessInfo[];
  error?: string;
}

const port = ref("");
const confirmedPort = ref(""); // Port that has been confirmed by user action
const loading = ref(false);
const result = reactive<PortCheckResult>({
  is_occupied: false,
  processes: [],
});
const message = ref("");

// Process detail modal state
const showDetailModal = ref(false);
const detailLoading = ref(false);
const selectedProcessDetail = ref<ProcessDetail | null>(null);

// Check port occupation
async function checkPort() {
  if (!port.value) {
    message.value = "Please enter a port number";
    return;
  }

  // Update confirmed port when user initiates check
  confirmedPort.value = port.value;
  
  loading.value = true;
  message.value = "";

  console.log(`Checking port ${port.value}...`);

  try {
    const response = await invoke<PortCheckResult>("check_port", { port: port.value.toString() });
    Object.assign(result, response);
    
    if (response.error) {
      message.value = `Error: ${response.error}`;
      console.error(`Port check error: ${response.error}`);
    } else if (!response.is_occupied) {
      message.value = `Port ${port.value} is available`;
      console.log(`Port ${port.value} is available`);
    } else {
      message.value = `Found ${response.processes.length} process(es) using port ${port.value}`;
      console.log(`Found ${response.processes.length} processes using port ${port.value}`, response.processes);
    }
  } catch (error) {
    message.value = `Query failed: ${error}`;
    console.error(`Port check failed: ${error}`);
  } finally {
    loading.value = false;
  }
}

// Kill process with force option
async function killProcess(pid: string, name: string, graceful: boolean = false) {
  const action = graceful ? "gracefully terminate" : "force kill";
  console.log(`Attempting to ${action} process ${name} (PID: ${pid})`);
  
  try {
    const command = graceful ? "graceful_kill_process" : "kill_process";
    await invoke<string>(command, { pid });
    message.value = `Successfully ${graceful ? "gracefully terminated" : "force killed"} process ${name} (PID: ${pid})`;
    console.log(`Successfully ${action} process ${name} (PID: ${pid})`);
    
    // Refresh port check after killing process
    await checkPort();
  } catch (error) {
    message.value = `Failed to ${action} process: ${error}`;
    console.error(`Failed to ${action} process ${name} (PID: ${pid}): ${error}`);
  }
}

// Get process detail
async function getProcessDetail(pid: string) {
  console.log(`Getting details for process PID: ${pid}`);
  detailLoading.value = true;
  
  try {
    const detail = await invoke<ProcessDetail>("get_process_detail", { pid });
    selectedProcessDetail.value = detail;
    showDetailModal.value = true;
    console.log(`Successfully retrieved details for PID: ${pid}`, detail);
  } catch (error) {
    message.value = `Failed to get process details: ${error}`;
    console.error(`Failed to get process details for PID ${pid}: ${error}`);
  } finally {
    detailLoading.value = false;
  }
}

// Close detail modal
function closeDetailModal() {
  showDetailModal.value = false;
  selectedProcessDetail.value = null;
}

// Handle Enter key press
function handleKeyPress(event: KeyboardEvent) {
  if (event.key === "Enter") {
    checkPort();
  }
}
</script>

<template>
  <div class="app-container">
    <div class="main-content">
      <!-- Header -->
      <header class="app-header">
        <h1 class="app-title">⚡ Kill Process</h1>
      </header>

      <!-- Port Input Section -->
      <section class="input-section">
        <div class="input-group">
          <input
            v-model="port"
            @keypress="handleKeyPress"
            type="number"
            placeholder="Enter port number (e.g., 3000)"
            class="port-input"
            :disabled="loading"
          />
          <button
            @click="checkPort"
            :disabled="loading"
            class="check-button"
          >
            <span v-if="loading" class="loading-content">
              <span class="loading-spinner"></span>
              Scanning...
            </span>
            <span v-else>🔍 Check Port</span>
          </button>
        </div>
      </section>

      <!-- Message Display -->
      <div v-if="message" class="message-display">
        <div class="message-content" :class="{ 'error': message.includes('Error') || message.includes('failed') }">
          <span class="message-icon">{{ message.includes('Error') || message.includes('failed') ? '❌' : message.includes('available') ? '✅' : 'ℹ️' }}</span>
          {{ message }}
        </div>
      </div>

      <!-- Process Table -->
      <div v-if="result.processes.length > 0" class="process-table-container">
        <div class="table-header">
          <h2 class="table-title">🔥 Processes Using Port {{ confirmedPort }}</h2>
          <span class="process-count">{{ result.processes.length }} process(es) found</span>
        </div>
        
        <div class="table-wrapper">
          <table class="process-table">
            <thead>
              <tr>
                <th>Port</th>
                <th>PID</th>
                <th>Process Name</th>
                <th>Action</th>
              </tr>
            </thead>
            <tbody>
              <tr
                v-for="process in result.processes"
                :key="process.pid"
                class="process-row"
              >
                <td class="port-cell">
                  <span class="port-badge">{{ process.port }}</span>
                </td>
                <td class="pid-cell">{{ process.pid }}</td>
                <td class="name-cell">
                  <span class="process-name">{{ process.name }}</span>
                </td>
                <td class="action-cell">
                  <div class="action-buttons">
                    <button
                      @click="getProcessDetail(process.pid)"
                      :disabled="detailLoading"
                      class="detail-button"
                      title="View process details"
                    >
                      📋 Detail
                    </button>
                    <button
                      @click="killProcess(process.pid, process.name, true)"
                      class="graceful-kill-button"
                      title="Gracefully terminate this process (SIGTERM)"
                    >
                      🔥 Graceful
                    </button>
                    <button
                      @click="killProcess(process.pid, process.name, false)"
                      class="force-kill-button"
                      title="Force kill this process (SIGKILL)"
                    >
                      💀 Force
                    </button>
                  </div>
                </td>
              </tr>
            </tbody>
          </table>
        </div>
      </div>

      <!-- Empty State -->
      <div v-else-if="!loading && confirmedPort && !result.is_occupied" class="empty-state">
        <div class="empty-icon">✨</div>
        <h3 class="empty-title">Port Available</h3>
        <p class="empty-description">Port {{ confirmedPort }} is not being used by any process</p>
      </div>
    </div>

    <!-- Process Detail Modal -->
    <div v-if="showDetailModal" class="modal-overlay" @click="closeDetailModal">
      <div class="modal-content" @click.stop>
        <div class="modal-header">
          <h3 class="modal-title">🔍 Process Details</h3>
          <button class="modal-close" @click="closeDetailModal">✕</button>
        </div>
        
        <div v-if="selectedProcessDetail" class="modal-body">
          <div class="detail-grid">
            <div class="detail-item">
              <span class="detail-label">PID:</span>
              <span class="detail-value">{{ selectedProcessDetail.pid }}</span>
            </div>
            <div class="detail-item">
              <span class="detail-label">Process Name:</span>
              <span class="detail-value">{{ selectedProcessDetail.name }}</span>
            </div>
            <div class="detail-item">
              <span class="detail-label">Port:</span>
              <span class="detail-value">{{ selectedProcessDetail.port }}</span>
            </div>
            <div v-if="selectedProcessDetail.user" class="detail-item">
              <span class="detail-label">User:</span>
              <span class="detail-value">{{ selectedProcessDetail.user }}</span>
            </div>
            <div v-if="selectedProcessDetail.cpu_usage" class="detail-item">
              <span class="detail-label">CPU Usage:</span>
              <span class="detail-value">{{ selectedProcessDetail.cpu_usage }}</span>
            </div>
            <div v-if="selectedProcessDetail.memory_usage" class="detail-item">
              <span class="detail-label">Memory Usage:</span>
              <span class="detail-value">{{ selectedProcessDetail.memory_usage }}</span>
            </div>
            <div v-if="selectedProcessDetail.start_time" class="detail-item">
              <span class="detail-label">Start Time:</span>
              <span class="detail-value">{{ selectedProcessDetail.start_time }}</span>
            </div>
            <div v-if="selectedProcessDetail.command" class="detail-item command-item">
              <span class="detail-label">Command:</span>
              <span class="detail-value command-value">{{ selectedProcessDetail.command }}</span>
            </div>
          </div>
        </div>
        
        <div class="modal-footer">
          <button class="modal-button" @click="closeDetailModal">Close</button>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
/* Apple-inspired design system */
* {
  box-sizing: border-box;
}

.app-container {
  min-height: 100vh;
  background: linear-gradient(135deg, #f5f7fa 0%, #c3cfe2 100%);
  font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, Oxygen, Ubuntu, Cantarell, sans-serif;
  padding: 20px;
}

.main-content {
  max-width: 900px;
  margin: 0 auto;
}

/* Header Styles */
.app-header {
  text-align: center;
  margin-bottom: 40px;
}

.app-title {
  font-size: 2.5rem;
  font-weight: 700;
  color: #1d1d1f;
  margin: 0 0 8px 0;
  letter-spacing: -0.02em;
}

.app-subtitle {
  font-size: 1.1rem;
  color: #6e6e73;
  margin: 0;
  font-weight: 400;
}

/* Input Section */
.input-section {
  background: rgba(255, 255, 255, 0.8);
  backdrop-filter: blur(20px);
  border-radius: 16px;
  padding: 24px;
  margin-bottom: 24px;
  box-shadow: 0 8px 32px rgba(0, 0, 0, 0.1);
  border: 1px solid rgba(255, 255, 255, 0.2);
}

.input-group {
  display: flex;
  gap: 12px;
  align-items: stretch;
}

.port-input {
  flex: 1;
  padding: 14px 16px;
  border: 2px solid #e5e5e7;
  border-radius: 12px;
  font-size: 16px;
  transition: all 0.2s ease;
  background: white;
  outline: none;
}

.port-input:focus {
  border-color: #007aff;
  box-shadow: 0 0 0 4px rgba(0, 122, 255, 0.1);
  transform: translateY(-1px);
}

.port-input:disabled {
  background: #f2f2f7;
  color: #8e8e93;
  cursor: not-allowed;
}

.check-button {
  padding: 14px 24px;
  background: linear-gradient(135deg, #007aff 0%, #5856d6 100%);
  color: white;
  border: none;
  border-radius: 12px;
  font-size: 16px;
  font-weight: 600;
  cursor: pointer;
  transition: all 0.2s ease;
  box-shadow: 0 4px 16px rgba(0, 122, 255, 0.3);
  outline: none;
}

.check-button:hover:not(:disabled) {
  transform: translateY(-2px);
  box-shadow: 0 8px 24px rgba(0, 122, 255, 0.4);
}

.check-button:active {
  transform: translateY(0);
}

.check-button:disabled {
  opacity: 0.6;
  cursor: not-allowed;
  transform: none;
}

.loading-content {
  display: flex;
  align-items: center;
  gap: 8px;
}

.loading-spinner {
  width: 16px;
  height: 16px;
  border: 2px solid rgba(255, 255, 255, 0.3);
  border-top: 2px solid white;
  border-radius: 50%;
  animation: spin 1s linear infinite;
}

@keyframes spin {
  0% { transform: rotate(0deg); }
  100% { transform: rotate(360deg); }
}

/* Message Display */
.message-display {
  margin-bottom: 24px;
}

.message-content {
  background: rgba(255, 255, 255, 0.9);
  border-radius: 12px;
  padding: 16px 20px;
  box-shadow: 0 4px 16px rgba(0, 0, 0, 0.1);
  border-left: 4px solid #007aff;
  display: flex;
  align-items: center;
  gap: 12px;
  font-weight: 500;
  color: #1d1d1f;
}

.message-content.error {
  border-left-color: #ff3b30;
  background: rgba(255, 59, 48, 0.05);
}

.message-icon {
  font-size: 18px;
}

/* Process Table */
.process-table-container {
  background: rgba(255, 255, 255, 0.9);
  backdrop-filter: blur(20px);
  border-radius: 16px;
  overflow: hidden;
  box-shadow: 0 8px 32px rgba(0, 0, 0, 0.1);
  border: 1px solid rgba(255, 255, 255, 0.2);
}

.table-header {
  background: linear-gradient(135deg, #f2f2f7 0%, #e5e5ea 100%);
  padding: 20px 24px;
  border-bottom: 1px solid rgba(0, 0, 0, 0.05);
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.table-title {
  font-size: 1.3rem;
  font-weight: 600;
  color: #1d1d1f;
  margin: 0;
}

.process-count {
  font-size: 0.9rem;
  color: #6e6e73;
  font-weight: 500;
}

.table-wrapper {
  overflow-x: auto;
}

.process-table {
  width: 100%;
  border-collapse: collapse;
  font-size: 14px;
}

.process-table th {
  background: #f2f2f7;
  padding: 16px 20px;
  text-align: left;
  font-weight: 600;
  color: #1d1d1f;
  border-bottom: 1px solid #e5e5ea;
  white-space: nowrap;
}

.process-row {
  transition: background-color 0.2s ease;
}

.process-row:hover {
  background: rgba(0, 122, 255, 0.05);
}

.process-table td {
  padding: 16px 20px;
  border-bottom: 1px solid rgba(0, 0, 0, 0.05);
  vertical-align: middle;
}

.port-badge {
  background: linear-gradient(135deg, #007aff 0%, #5856d6 100%);
  color: white;
  padding: 4px 12px;
  border-radius: 20px;
  font-size: 12px;
  font-weight: 600;
  display: inline-block;
}

.pid-cell {
  font-family: 'SF Mono', Monaco, 'Cascadia Code', 'Roboto Mono', Consolas, 'Courier New', monospace;
  font-weight: 500;
  color: #6e6e73;
}

.process-name {
  font-weight: 500;
  color: #1d1d1f;
}

/* Action Buttons */
.action-buttons {
  display: flex;
  gap: 8px;
  flex-wrap: wrap;
}

.detail-button {
  background: linear-gradient(135deg, #007aff 0%, #5856d6 100%);
  color: white;
  border: none;
  border-radius: 6px;
  padding: 6px 12px;
  font-size: 11px;
  font-weight: 600;
  cursor: pointer;
  transition: all 0.2s ease;
  box-shadow: 0 2px 6px rgba(0, 122, 255, 0.3);
}

.detail-button:hover:not(:disabled) {
  transform: translateY(-1px);
  box-shadow: 0 4px 10px rgba(0, 122, 255, 0.4);
}

.detail-button:disabled {
  opacity: 0.6;
  cursor: not-allowed;
  transform: none;
}

.graceful-kill-button {
  background: linear-gradient(135deg, #ff9500 0%, #ff6d00 100%);
  color: white;
  border: none;
  border-radius: 6px;
  padding: 6px 12px;
  font-size: 11px;
  font-weight: 600;
  cursor: pointer;
  transition: all 0.2s ease;
  box-shadow: 0 2px 6px rgba(255, 149, 0, 0.3);
}

.graceful-kill-button:hover {
  transform: translateY(-1px);
  box-shadow: 0 4px 10px rgba(255, 149, 0, 0.4);
}

.force-kill-button {
  background: linear-gradient(135deg, #ff3b30 0%, #ff2d92 100%);
  color: white;
  border: none;
  border-radius: 6px;
  padding: 6px 12px;
  font-size: 11px;
  font-weight: 600;
  cursor: pointer;
  transition: all 0.2s ease;
  box-shadow: 0 2px 6px rgba(255, 59, 48, 0.3);
}

.force-kill-button:hover {
  transform: translateY(-1px);
  box-shadow: 0 4px 10px rgba(255, 59, 48, 0.4);
}

.detail-button:active,
.graceful-kill-button:active,
.force-kill-button:active {
  transform: translateY(0);
}

/* Empty State */
.empty-state {
  background: rgba(255, 255, 255, 0.8);
  backdrop-filter: blur(20px);
  border-radius: 16px;
  padding: 48px 24px;
  text-align: center;
  box-shadow: 0 8px 32px rgba(0, 0, 0, 0.1);
  border: 1px solid rgba(255, 255, 255, 0.2);
}

.empty-icon {
  font-size: 4rem;
  margin-bottom: 16px;
}

.empty-title {
  font-size: 1.5rem;
  font-weight: 600;
  color: #1d1d1f;
  margin: 0 0 8px 0;
}

.empty-description {
  color: #6e6e73;
  font-size: 1rem;
  margin: 0;
}

/* Responsive Design */
@media (max-width: 768px) {
  .app-container {
    padding: 16px;
  }
  
  .input-group {
    flex-direction: column;
  }
  
  .check-button {
    width: 100%;
  }
  
  .table-header {
    flex-direction: column;
    gap: 8px;
    align-items: flex-start;
  }
  
  .process-table {
    font-size: 12px;
  }
  
  .process-table th,
  .process-table td {
    padding: 12px 16px;
  }
  
  .action-buttons {
    flex-direction: column;
    gap: 4px;
  }
}

/* Modal Styles */
.modal-overlay {
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background: rgba(0, 0, 0, 0.5);
  backdrop-filter: blur(4px);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 1000;
}

.modal-content {
  background: white;
  border-radius: 16px;
  box-shadow: 0 20px 60px rgba(0, 0, 0, 0.3);
  max-width: 600px;
  width: 90%;
  max-height: 80vh;
  overflow: hidden;
  animation: modalSlideIn 0.3s ease;
}

@keyframes modalSlideIn {
  from {
    opacity: 0;
    transform: scale(0.9) translateY(-20px);
  }
  to {
    opacity: 1;
    transform: scale(1) translateY(0);
  }
}

.modal-header {
  background: linear-gradient(135deg, #f2f2f7 0%, #e5e5ea 100%);
  padding: 20px 24px;
  border-bottom: 1px solid rgba(0, 0, 0, 0.05);
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.modal-title {
  font-size: 1.3rem;
  font-weight: 600;
  color: #1d1d1f;
  margin: 0;
}

.modal-close {
  background: none;
  border: none;
  font-size: 18px;
  color: #6e6e73;
  cursor: pointer;
  padding: 4px;
  border-radius: 4px;
  transition: all 0.2s ease;
}

.modal-close:hover {
  background: rgba(0, 0, 0, 0.1);
  color: #1d1d1f;
}

.modal-body {
  padding: 24px;
  max-height: 60vh;
  overflow-y: auto;
}

.detail-grid {
  display: grid;
  gap: 16px;
}

.detail-item {
  display: grid;
  grid-template-columns: 120px 1fr;
  gap: 12px;
  align-items: start;
  padding: 12px;
  background: #f8f9fa;
  border-radius: 8px;
  border-left: 3px solid #007aff;
}

.detail-item.command-item {
  grid-template-columns: 120px 1fr;
}

.detail-label {
  font-weight: 600;
  color: #1d1d1f;
  font-size: 14px;
}

.detail-value {
  color: #6e6e73;
  font-size: 14px;
  word-break: break-word;
}

.command-value {
  font-family: 'SF Mono', Monaco, 'Cascadia Code', 'Roboto Mono', Consolas, 'Courier New', monospace;
  background: white;
  padding: 8px 12px;
  border-radius: 6px;
  border: 1px solid #e5e5ea;
  font-size: 13px;
  line-height: 1.4;
}

.modal-footer {
  background: #f8f9fa;
  padding: 16px 24px;
  border-top: 1px solid rgba(0, 0, 0, 0.05);
  display: flex;
  justify-content: flex-end;
}

.modal-button {
  background: linear-gradient(135deg, #007aff 0%, #5856d6 100%);
  color: white;
  border: none;
  border-radius: 8px;
  padding: 10px 20px;
  font-size: 14px;
  font-weight: 600;
  cursor: pointer;
  transition: all 0.2s ease;
  box-shadow: 0 2px 8px rgba(0, 122, 255, 0.3);
}

.modal-button:hover {
  transform: translateY(-1px);
  box-shadow: 0 4px 12px rgba(0, 122, 255, 0.4);
}

.modal-button:active {
  transform: translateY(0);
}
</style>
