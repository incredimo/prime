// WebSocket connection for real-time updates
let ws;
let reconnectAttempts = 0;
const maxReconnectAttempts = 5;

function connectWebSocket() {
  const protocol = window.location.protocol === 'https:' ? 'wss:' : 'ws:';
  const wsUrl = `${protocol}//${window.location.host}/ws`;
  
  ws = new WebSocket(wsUrl);
  
  ws.onopen = function() {
    console.log('WebSocket connected');
    reconnectAttempts = 0;
    const statusIndicator = document.getElementById('ws-status');
    if (statusIndicator) {
      statusIndicator.className = 'status-online';
      statusIndicator.textContent = 'Connected';
    }
  };
  
  ws.onmessage = function(event) {
    const data = JSON.parse(event.data);
    
    if (data.type === 'log') {
      appendLog(data);
    } else if (data.type === 'task_update') {
      updateTaskStatus(data);
    } else if (data.type === 'task_complete') {
      completeTask(data);
    }
  };
  
  ws.onclose = function() {
    const statusIndicator = document.getElementById('ws-status');
    if (statusIndicator) {
      statusIndicator.className = 'status-offline';
      statusIndicator.textContent = 'Disconnected';
    }
    
    // Attempt to reconnect
    reconnectAttempts++;
    if (reconnectAttempts <= maxReconnectAttempts) {
      console.log(`WebSocket closed. Reconnecting (${reconnectAttempts}/${maxReconnectAttempts})...`);
      setTimeout(connectWebSocket, 3000);
    } else {
      console.log('WebSocket connection failed after multiple attempts.');
    }
  };
  
  ws.onerror = function(error) {
    console.error('WebSocket error:', error);
  };
}

// Log streaming with Server-Sent Events
function setupLogStream() {
  const logConsole = document.getElementById('log-console');
  if (!logConsole) return;
  
  const evtSource = new EventSource('/logs/stream');
  
  evtSource.onopen = function() {
    console.log('Log stream connected');
  };
  
  evtSource.addEventListener('logs', function(event) {
    const logs = JSON.parse(event.data);
    logConsole.innerHTML = '';
    
    logs.forEach(log => {
      const logLine = document.createElement('div');
      logLine.className = 'console-line';
      logLine.textContent = `[${log.timestamp}] ${log.message}`;
      logConsole.appendChild(logLine);
    });
    
    logConsole.scrollTop = logConsole.scrollHeight;
  });
  
  evtSource.addEventListener('log', function(event) {
    const log = JSON.parse(event.data);
    
    const logLine = document.createElement('div');
    logLine.className = 'console-line';
    logLine.textContent = `[${log.timestamp}] ${log.message}`;
    logConsole.appendChild(logLine);
    
    logConsole.scrollTop = logConsole.scrollHeight;
  });
  
  evtSource.onerror = function() {
    console.error('Log stream error. Reconnecting...');
  };
}

// Fetch and display active tasks
function loadActiveTasks() {
  const tasksList = document.getElementById('active-tasks');
  if (!tasksList) return;
  
  fetch('/api/tasks')
    .then(response => response.json())
    .then(tasks => {
      tasksList.innerHTML = '';
      
      if (Object.keys(tasks).length === 0) {
        tasksList.innerHTML = '<p>No active tasks.</p>';
        return;
      }
      
      for (const [id, task] of Object.entries(tasks)) {
        const taskItem = document.createElement('div');
        taskItem.className = 'task-item';
        // Determine if cancel/retry should be shown
        const status = (task.status || '').toLowerCase();
        let controls = `<a href="/task/${id}" class="button" style="margin-right:8px;">View Details</a>`;
        if (status.includes('running') || status.includes('starting') || status.includes('prompting')) {
          controls += `<button class="button button-danger" data-cancel="${id}">Cancel</button>`;
        } else if (status.includes('failed') || status.includes('completed') || status.includes('cancelled')) {
          controls += `<button class="button button-secondary" data-retry="${id}">Retry</button>`;
        }
        taskItem.innerHTML = `
          <div class="task-header">
            <div class="task-goal">${task.goal}</div>
            <span class="status status-${task.status.toLowerCase().split(' ')[0]}">${task.status}</span>
          </div>
          <div class="task-meta">
            <div>Created: ${new Date(task.created).toLocaleString()}</div>
            ${controls}
          </div>
        `;
        // Add event listeners for cancel/retry
        setTimeout(() => {
          const cancelBtn = taskItem.querySelector('[data-cancel]');
          if (cancelBtn) {
            cancelBtn.onclick = function() {
              if (confirm('Cancel this task?')) {
                fetch(`/api/task/${id}/cancel`, { method: 'POST' })
                  .then(resp => resp.json())
                  .then(data => {
                    loadActiveTasks();
                  });
              }
            };
          }
          const retryBtn = taskItem.querySelector('[data-retry]');
          if (retryBtn) {
            retryBtn.onclick = function() {
              if (confirm('Retry this task?')) {
                fetch(`/api/task/${id}/retry`, { method: 'POST' })
                  .then(resp => resp.json())
                  .then(data => {
                    if (data && data.id) {
                      window.location.href = `/task/${data.id}`;
                    } else {
                      alert('Failed to retry task.');
                    }
                  });
              }
            };
          }
        }, 0);
        tasksList.appendChild(taskItem);
      }
    })
    .catch(error => {
      console.error('Error loading tasks:', error);
      tasksList.innerHTML = '<p>Error loading tasks. Please try again.</p>';
    });
}

// Load task history
function loadTaskHistory() {
  const historyList = document.getElementById('task-history');
  if (!historyList) return;
  
  fetch('/api/history')
    .then(response => response.json())
    .then(tasks => {
      historyList.innerHTML = '';
      
      if (tasks.length === 0) {
        historyList.innerHTML = '<p>No task history found.</p>';
        return;
      }
      
      tasks.forEach(task => {
        const taskItem = document.createElement('div');
        taskItem.className = 'task-item';
  
        // Format duration nicely
        let duration = 'N/A';
        if (task.duration) {
          if (task.duration < 60) {
            duration = `${task.duration} seconds`;
          } else if (task.duration < 3600) {
            duration = `${Math.floor(task.duration / 60)} minutes`;
          } else {
            duration = `${Math.floor(task.duration / 3600)} hours, ${Math.floor((task.duration % 3600) / 60)} minutes`;
          }
        }
  
        // Add retry button for completed/failed/cancelled tasks
        const status = (task.status || '').toLowerCase();
        let controls = `<button class="button" onclick="showOutput('${task.id}', \`${task.goal}\`, \`${task.output.replace(/`/g, '\\`')}\`)">Show Output</button>`;
        if (status.includes('failed') || status.includes('completed') || status.includes('cancelled')) {
          controls += ` <button class="button button-secondary" data-retry="${task.id}">Retry</button>`;
        }
  
        taskItem.innerHTML = `
          <div class="task-header">
            <div class="task-goal">${task.goal}</div>
            <span class="status status-${task.status.toLowerCase()}">${task.status}</span>
          </div>
          <div class="task-meta">
            <div>Executed: ${new Date(task.timestamp).toLocaleString()}</div>
            <div>Duration: ${duration}</div>
            ${controls}
          </div>
        `;
        setTimeout(() => {
          const retryBtn = taskItem.querySelector('[data-retry]');
          if (retryBtn) {
            retryBtn.onclick = function() {
              if (confirm('Retry this task?')) {
                fetch(`/api/task/${task.id}/retry`, { method: 'POST' })
                  .then(resp => resp.json())
                  .then(data => {
                    if (data && data.id) {
                      window.location.href = `/task/${data.id}`;
                    } else {
                      alert('Failed to retry task.');
                    }
                  });
              }
            };
          }
        }, 0);
        historyList.appendChild(taskItem);
      });
    })
    .catch(error => {
      console.error('Error loading history:', error);
      historyList.innerHTML = '<p>Error loading task history. Please try again.</p>';
    });
}

// Task output modal
function showOutput(id, goal, output) {
  const modal = document.createElement('div');
  modal.style.position = 'fixed';
  modal.style.top = '0';
  modal.style.left = '0';
  modal.style.width = '100%';
  modal.style.height = '100%';
  modal.style.backgroundColor = 'rgba(0,0,0,0.7)';
  modal.style.zIndex = '1000';
  modal.style.display = 'flex';
  modal.style.justifyContent = 'center';
  modal.style.alignItems = 'center';
  
  const content = document.createElement('div');
  content.style.backgroundColor = 'white';
  content.style.padding = '20px';
  content.style.borderRadius = '8px';
  content.style.width = '80%';
  content.style.maxWidth = '800px';
  content.style.maxHeight = '80vh';
  content.style.overflow = 'auto';
  
  content.innerHTML = `
    <h2>${goal}</h2>
    <div class="output-code">${output.replace(/\n/g, '<br>')}</div>
    <button class="button button-secondary" onclick="document.body.removeChild(document.querySelector('[data-modal]'))">Close</button>
  `;
  
  modal.appendChild(content);
  modal.setAttribute('data-modal', '');
  
  // Close on click outside
  modal.addEventListener('click', function(e) {
    if (e.target === modal) {
      document.body.removeChild(modal);
    }
  });
  
  document.body.appendChild(modal);
}

// Task detail page functionality
function loadTaskDetails(taskId) {
  if (!taskId) return;
  
  const taskOutput = document.getElementById('task-output');
  const taskStatus = document.getElementById('task-status');
  const taskGoal = document.getElementById('task-goal');
  
  if (!taskOutput || !taskStatus || !taskGoal) return;
  
  function updateTask() {
    fetch(`/api/task/${taskId}`)
      .then(response => response.json())
      .then(task => {
        if (task.error) {
          taskOutput.innerHTML = `<p>Error: ${task.error}</p>`;
          return;
        }
        
        taskGoal.textContent = task.goal;
        taskStatus.textContent = task.status;
        taskStatus.className = `status status-${task.status.toLowerCase().split(' ')[0]}`;
        
        // Format the output with proper line breaks
        taskOutput.innerHTML = task.output.replace(/\n/g, '<br>');
        
        // If task is still running, schedule another update
        if (task.status.toLowerCase().includes('running')) {
          setTimeout(updateTask, 2000);
        }
      })
      .catch(error => {
        console.error('Error loading task details:', error);
        taskOutput.innerHTML = '<p>Error loading task details. Please try again.</p>';
      });
  }
  
  updateTask();
}

// Update system status indicators
function updateSystemStatus() {
  const statusContainer = document.getElementById('system-status');
  if (!statusContainer) return;
  
  fetch('/api/status')
    .then(response => response.json())
    .then(status => {
      // Ollama status
      const ollamaStatus = document.getElementById('ollama-status');
      if (ollamaStatus) {
        ollamaStatus.textContent = status.ollama === 'running' ? 'Online' : 'Offline';
        ollamaStatus.className = status.ollama === 'running' ? 'status-online' : 'status-offline';
      }
      
      // Current model
      const currentModel = document.getElementById('current-model');
      if (currentModel) {
        currentModel.textContent = status.current_model;
      }
      
      // Active tasks
      const activeTasks = document.getElementById('active-tasks-count');
      if (activeTasks) {
        activeTasks.textContent = status.active_tasks;
      }
      
      // Available models
      const availableModels = document.getElementById('available-models');
      if (availableModels) {
        availableModels.textContent = status.models.join(', ') || 'None';
      }
    })
    .catch(error => {
      console.error('Error updating system status:', error);
    });
}

// Initialize page functionality
document.addEventListener('DOMContentLoaded', function() {
  // Connect WebSocket
  connectWebSocket();

  // Setup log streaming
  setupLogStream();

  // Load active tasks
  loadActiveTasks();

  // Load task history
  loadTaskHistory();

  // Update system status
  updateSystemStatus();

  // Batch/Template Task Submission
  const goalForm = document.querySelector('.goal-form');
  const goalTextarea = document.getElementById('goal');
  if (goalForm && goalTextarea) {
    // Add template buttons
    const templates = [
      "Install Docker",
      "Create a Python script to analyze logs",
      "Show disk usage and free memory",
      "List all running processes",
      "Update all system packages"
    ];
    const templateBar = document.createElement('div');
    templateBar.style.marginBottom = '8px';
    templateBar.innerHTML = templates.map(t =>
      `<button type="button" class="button button-secondary" style="margin-right:6px; margin-bottom:6px;" data-template="${t}">${t}</button>`
    ).join('');
    goalForm.insertBefore(templateBar, goalForm.firstChild);
    templateBar.querySelectorAll('button[data-template]').forEach(btn => {
      btn.addEventListener('click', function() {
        if (goalTextarea.value && !goalTextarea.value.endsWith('\n')) goalTextarea.value += '\n';
        goalTextarea.value += this.getAttribute('data-template');
        goalTextarea.focus();
      });
    });

    // Add feedback area
    let feedback = document.createElement('div');
    feedback.id = 'goal-feedback';
    feedback.style.margin = '10px 0';
    feedback.style.color = '#dc3545';
    goalForm.appendChild(feedback);

    goalForm.addEventListener('submit', function(e) {
      e.preventDefault();
      feedback.textContent = '';
      const goalsRaw = goalTextarea.value.trim();
      if (!goalsRaw) {
        feedback.textContent = 'Please enter at least one goal.';
        return;
      }
      const goals = goalsRaw.split('\n').map(g => g.trim()).filter(Boolean);
      if (goals.length === 0) {
        feedback.textContent = 'Please enter at least one goal.';
        return;
      }
      feedback.style.color = '#0066cc';
      feedback.textContent = 'Submitting...';
      let createdTasks = [];
      let errors = [];
      let completed = 0;

      // Submit each goal as a separate task
      goals.forEach((goal, idx) => {
        fetch('/api/goal', {
          method: 'POST',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify({ text: goal })
        })
        .then(resp => resp.json())
        .then(data => {
          if (data && data.id) {
            createdTasks.push(data.id);
          } else {
            errors.push(data && data.error ? data.error : 'Unknown error');
          }
        })
        .catch(err => {
          errors.push(err.message || 'Network error');
        })
        .finally(() => {
          completed++;
          if (completed === goals.length) {
            if (createdTasks.length > 0) {
              feedback.style.color = '#28a745';
              if (createdTasks.length === 1) {
                feedback.innerHTML = 'Task created! Redirecting...';
                setTimeout(() => {
                  window.location.href = `/task/${createdTasks[0]}`;
                }, 800);
              } else {
                feedback.innerHTML = 'Tasks created:<br>' +
                  createdTasks.map(id => `<a href="/task/${id}" class="button" style="margin:4px 0;display:inline-block;">View Task ${id}</a>`).join(' ');
              }
              goalTextarea.value = '';
              loadActiveTasks();
            }
            if (errors.length > 0) {
              feedback.style.color = '#dc3545';
              feedback.innerHTML += '<br>' + errors.map(e => `<div>${e}</div>`).join('');
            }
          }
        });
      });
    });
  }

  // Check if we're on a task detail page
  const taskId = document.getElementById('task-id')?.value;
  if (taskId) {
    loadTaskDetails(taskId);
  }

  // Refresh data periodically
  setInterval(function() {
    loadActiveTasks();
    updateSystemStatus();
  }, 10000);
});
