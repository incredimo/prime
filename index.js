/*
PRIME
-----
This is the origin of the OPTIMUS PRIME.
The ultimate AI Assistant with limitless capabilities and self evolving intelligence.
*/

import { GoogleGenAI } from '@google/genai';
import readline from 'readline';
import fs from 'fs';
import path from 'path';
import { sync as globSync } from 'glob'; // globSync is for directory finding
import YAML from 'yaml';
import { execSync } from 'child_process';

// Configuration constants
const CONVERSATIONS_DIR = 'conversations';
const HANDLERS_FILE = 'handlers.yaml';
const MEMORY_FILE = 'memory.yaml';
const PROMPT_TEMPLATE_FILE = 'prompt_template.md';
const HISTORY_COUNT = 5;

// Detect platform and choose shell
const IS_WINDOWS = process.platform === 'win32';
const DEFAULT_SHELL = IS_WINDOWS ? 'powershell.exe' : '/bin/bash'; // Explicitly specify /bin/bash for clarity

// Initialize LLM client
const ai = new GoogleGenAI({ apiKey: process.env.GEMINI_API_KEY });
const MODEL = 'gemini-2.5-flash-preview-05-20'; // Or 'gemini-1.5-flash-latest' for the newest flash model

// Safely read a file, return empty string if missing
function safeRead(filePath) {
  try {
    return fs.readFileSync(filePath, 'utf8');
  } catch (e) {
    if (e.code === 'ENOENT') {
      return ''; // File not found, return empty string
    }
    throw e; // Re-throw other errors
  }
}

// Ensure and return the current session directory
function ensureSessionDir() {
  if (!fs.existsSync(CONVERSATIONS_DIR)) fs.mkdirSync(CONVERSATIONS_DIR);

  // Find existing sessions, sort by name (which is ISO timestamp) to get the latest
  const sessions = fs.readdirSync(CONVERSATIONS_DIR, { withFileTypes: true })
                       .filter(dirent => dirent.isDirectory())
                       .map(dirent => dirent.name)
                       .sort();

  if (sessions.length) return path.join(CONVERSATIONS_DIR, sessions[sessions.length - 1]);

  const ts = new Date().toISOString().replace(/:/g, '-') + 'Z';
  const dir = path.join(CONVERSATIONS_DIR, ts);
  fs.mkdirSync(dir);
  return dir;
}

// Log a message into the current session folder
function logMessage(role, text) {
  const sessionDir = ensureSessionDir();
  // Ensure timestamp has milliseconds for uniqueness within a second
  const ts = new Date().toISOString().replace(/:/g, '-') + '-' + new Date().getMilliseconds() + 'Z';
  const fileName = `${ts}_${role}.md`;
  const filePath = path.join(sessionDir, fileName);
  const frontMatter = `---\ntimestamp: ${new Date().toISOString()}\nrole: ${role}\n---\n`;
  fs.writeFileSync(filePath, frontMatter + text + '\n');
}

// Retrieve the last N message files from the session
function getRecentHistory(sessionDir, n = HISTORY_COUNT) {
  const files = fs.readdirSync(sessionDir)
                   .filter(name => name.endsWith('.md'))
                   .map(name => path.join(sessionDir, name))
                   .sort(); // Sort by name (timestamp) for chronological order

  return files.slice(-n).map(fp => ({ path: fp, content: fs.readFileSync(fp, 'utf8') }));
}

// Extract annotated code blocks and inline commands from markdown
function extractBlocks(markdown) {
  const blocks = [];
  // Fenced code blocks: ```name [annotation]\n...``` 
  // Captures: 1=handler name, 2=annotation (optional), 3=code content
  const fencedRegex = /```(\w+)(?: +(\w+))?[\r\n]([\s\S]*?)```/g;
  let match;
  while ((match = fencedRegex.exec(markdown))) {
    const [, name, annotation, code] = match;
    // Default annotation to 'execute' if not provided
    blocks.push({ name, annotation: annotation || 'execute', code: code.trim() });
  }
  // If no fenced blocks, detect inline `command` as shell execute
  if (blocks.length === 0) {
    const inlineRegex = /`([^`\n]+)`/g;
    let im;
    while ((im = inlineRegex.exec(markdown))) {
      // Inline commands are always treated as 'shell execute'
      blocks.push({ name: 'shell', annotation: 'execute', code: im[1].trim() });
    }
  }
  return blocks;
}

// Dispatch a block using the handler registry
function dispatch(block) {
  const handlersText = safeRead(HANDLERS_FILE);
  const handlers = YAML.parse(handlersText);

  const handlerEntry = handlers[block.name];

  if (!handlerEntry) {
    return { status: 'failure', error: `Unknown handler: "${block.name}"` };
  }

  // `block.annotation` is already defaulted to 'execute' by `extractBlocks` if not present
  const actionKey = block.annotation; 

  let template;
  // Handle backward compatibility: if handlerEntry is a simple string, treat it as 'execute' template
  if (typeof handlerEntry === 'string') {
    if (actionKey !== 'execute') {
        return { status: 'failure', error: `Handler "${block.name}" is defined as a simple string, but action "${actionKey}" was requested. Only 'execute' is supported in this format.` };
    }
    template = handlerEntry;
  } else if (typeof handlerEntry === 'object' && handlerEntry !== null) {
    template = handlerEntry[actionKey];
    if (typeof template === 'undefined') {
      return { status: 'failure', error: `Handler "${block.name}" does not define action "${actionKey}".` };
    }
  } else {
    return { status: 'failure', error: `Invalid handler definition for "${block.name}". Handler entry should be a string or an object.` };
  }

  let cmd = template.replace('{code}', block.code); // Default replacement

  // --- Special handling for 'file' write/append actions ---
  // If the target is memory.yaml or handlers.yaml, use Node.js `fs` directly for safety & reliability.
  // For other files, construct a shell command.
  if (block.name === 'file' && (actionKey === 'write' || actionKey === 'append')) {
    const contentParts = block.code.split('\n');
    if (contentParts.length === 0) {
        return { status: 'failure', error: `File operation "${actionKey}" requires at least a path and content in the code block (path on first line).` };
    }
    const filePath = contentParts[0].trim();
    const fileContent = contentParts.slice(1).join('\n'); // All subsequent lines are content

    // Direct write for critical configuration files (bypasses shell for safety)
    if (filePath === MEMORY_FILE || filePath === HANDLERS_FILE) {
        try {
            fs.mkdirSync(path.dirname(filePath), { recursive: true }); // Ensure directory exists
            if (actionKey === 'write') {
                fs.writeFileSync(filePath, fileContent, 'utf8');
            } else { // 'append'
                fs.appendFileSync(filePath, fileContent, 'utf8');
            }
            return { status: 'success', output: `Successfully ${actionKey}d critical file: ${filePath}` };
        } catch (err) {
            return { status: 'failure', error: `Failed to ${actionKey} critical file "${filePath}": ${err.message}` };
        }
    } else {
      // For other files, use shell command as defined in handler, replacing {file_path} and {file_content}
      // Note: Escaping fileContent for shell's `printf %s` is crucial. The ' character must be handled.
      // Basic escaping for content for shell `printf %s`. This covers single quotes inside the content.
      const escapedFileContent = fileContent.replace(/'/g, "'\\''");

      cmd = cmd
        .replace('{file_path}', filePath)
        .replace('{file_content}', escapedFileContent);
    }
  }

  // --- Execute the command ---
  try {
    const result = execSync(cmd, { shell: DEFAULT_SHELL, encoding: 'utf8', stdio: ['pipe', 'pipe', 'pipe'] }); // Capture all stdio
    return { status: 'success', output: result.trim() };
  } catch (err) {
    // Return structured error information for LLM to debug
    return {
      status: 'failure',
      error: (err.message || 'Unknown error').trim(),
      stdout: (err.stdout || '').trim(),
      stderr: (err.stderr || '').trim()
    };
  }
}

// Build the LLM prompt by assembling all structured inputs
function buildPrompt(newUserMessage) {
  const sessionDir = ensureSessionDir();

  // 1. Prompt template (markdown for syntax highlighting)
  const promptTmpl = safeRead(PROMPT_TEMPLATE_FILE).trim();
  const tmplBlock = `#### Path: ${PROMPT_TEMPLATE_FILE}\n` +
                    `\`\`\`markdown\n${promptTmpl}\n\`\`\``;

  // 2. Handlers registry
  const handlersText = safeRead(HANDLERS_FILE).trim();
  const handlersBlock = `#### Path: ${HANDLERS_FILE}\n` +
                        `\`\`\`yaml\n${handlersText}\n\`\`\``;

  // 3. Memory store
  const memoryText = safeRead(MEMORY_FILE).trim();
  const memoryBlock = `#### Path: ${MEMORY_FILE}\n` +
                      `\`\`\`yaml\n${memoryText}\n\`\`\``;

  // 4. Environment info (add Current Working Directory)
  const envInfo = {
      platform: process.platform,
      shell: DEFAULT_SHELL,
      cwd: process.cwd()
  };
  const envBlock = `#### Environment\n` +
                   `\`\`\`json\n${JSON.stringify(envInfo, null, 2)}\n\`\`\``;

  // 5. Session history (last N messages)
  const history = getRecentHistory(sessionDir);
  const historyBlocks = history.map(h =>
    `#### Path: ${h.path}\n` +
    `\`\`\`markdown\n${h.content.trim()}\n\`\`\``
  ).join('\n\n');

  // 6. New user message
  const userBlock = `#### Path: [stdin]\n` +
                    `\`\`\`text\n${newUserMessage.trim()}\n\`\`\``;

  return [tmplBlock, handlersBlock, memoryBlock, envBlock, historyBlocks, userBlock].join('\n\n');
}

async function main() {
  const rl = readline.createInterface({ input: process.stdin, output: process.stdout });

  console.log('Welcome to the AI Orchestrator CLI. Type "exit" to quit.');

  // Ensure essential files exist or are created if missing (with default content)
  if (safeRead(PROMPT_TEMPLATE_FILE) === '') fs.writeFileSync(PROMPT_TEMPLATE_FILE, "# SYSTEM PROMPT\nYou are Prime, the orchestration engine...\n", 'utf8'); // Placeholder
  if (safeRead(HANDLERS_FILE) === '') fs.writeFileSync(HANDLERS_FILE, YAML.stringify({ shell: { execute: "{code}" } }), 'utf8'); // Basic shell handler
  if (safeRead(MEMORY_FILE) === '') fs.writeFileSync(MEMORY_FILE, '# LLM’s persistent memory store\n', 'utf8');

  while (true) {
    const userMessage = await new Promise(res => rl.question('\nYou: ', res));
    if (userMessage.toLowerCase() === 'exit') {
      console.log('Goodbye!');
      rl.close();
      process.exit(0);
    }

    // Log the user message
    logMessage('user', userMessage);

    // Build prompt and invoke the LLM
    const prompt = buildPrompt(userMessage);
    let assistantReply = '';
    try {
      const responseStream = await ai.models.generateContentStream({
        model: MODEL,
        // System instructions are crucial for shaping AI's general behavior
        systemInstruction: { text: 'You are Prime, a highly capable and intelligent AI assistant. Follow the prompt instructions precisely, prioritize task completion through action, and respond clearly with a focus on executable code and actionable steps. Your output will be parsed for commands. Only output commands and text required by the prompt, no conversational filler or salutations when asked to execute unless explicitly prompted.' },
        contents: [{ role: 'user', parts: [{ text: prompt }] }]
      });

      process.stdout.write('\nAssistant: ');
      for await (const chunk of responseStream) {
        process.stdout.write(chunk.text);
        assistantReply += chunk.text;
      }
      console.log(); // Newline after stream finishes
    } catch (err) {
      console.error('LLM Error:', err.message);
      // Log LLM error as a structured system message for historical context for the AI
      logMessage('system', '```json system_error\n' + JSON.stringify({ status: 'failure', type: 'LLM_ERROR', message: err.message }, null, 2) + '\n```');
      continue; // Skip processing blocks if LLM call failed
    }

    // Log assistant reply
    logMessage('assistant', assistantReply);

    // Execute annotated blocks or inline commands from assistant's reply
    const blocks = extractBlocks(assistantReply);
    if (blocks.length === 0) {
        console.log('[System Info] No executable blocks detected in assistant\'s reply. Assistant may provide conversational output or ask questions.');
        logMessage('system', '```text\nNo executable blocks detected.\n```');
    }

    for (const block of blocks) {
      console.log(`\n[Executing: ${block.name}${block.annotation !== 'execute' ? ` (${block.annotation})` : ''}]`);
      // Display first line of code or a short snippet
      console.log(`Code: "${block.code.split('\n')[0].substring(0, 100)}${block.code.split('\n')[0].length > 100 ? '...' : ''}"`);

      const result = dispatch(block);
      
      // Log the structured result for the AI's future context
      logMessage('system', '```json system_output\n' + JSON.stringify(result, null, 2) + '\n```');
      
      // Print the output to the human user for immediate feedback
      console.log('\n[System Output]:');
      if (result.status === 'success') {
          console.log(`Status: SUCCESS`);
          if (result.output) {
              console.log('Output:\n' + result.output);
          }
      } else { // result.status === 'failure'
          console.error(`Status: FAILED`);
          console.error(`Error: ${result.error}`);
          if (result.stdout) {
              console.log('Stdout:\n' + result.stdout);
          }
          if (result.stderr) {
              console.error('Stderr:\n' + result.stderr);
          }
      }
    }
  }
}

main().catch(console.error);