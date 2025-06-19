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
import { sync as globSync } from 'glob';
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
const DEFAULT_SHELL = IS_WINDOWS ? 'powershell.exe' : true;

// Initialize LLM client
const ai = new GoogleGenAI({ apiKey: process.env.GEMINI_API_KEY });
const MODEL = 'gemini-2.5-flash-preview-05-20';

// Safely read a file, return empty string if missing
function safeRead(filePath) {
  try {
    return fs.readFileSync(filePath, 'utf8');
  } catch {
    return '';
  }
}

// Ensure and return the current session directory
function ensureSessionDir() {
  if (!fs.existsSync(CONVERSATIONS_DIR)) fs.mkdirSync(CONVERSATIONS_DIR);
  const sessions = globSync(path.join(CONVERSATIONS_DIR, '*/')).sort();
  if (sessions.length) return sessions[sessions.length - 1];

  const ts = new Date().toISOString().replace(/:/g, '-') + 'Z';
  const dir = path.join(CONVERSATIONS_DIR, ts);
  fs.mkdirSync(dir);
  return dir;
}

// Log a message into the current session folder
function logMessage(role, text) {
  const sessionDir = ensureSessionDir();
  const ts = new Date().toISOString().replace(/:/g, '-') + 'Z';
  const fileName = `${ts}_${role}.md`;
  const filePath = path.join(sessionDir, fileName);
  const frontMatter = `---\ntimestamp: ${new Date().toISOString()}\nrole: ${role}\n---\n`;
  fs.writeFileSync(filePath, frontMatter + text + '\n');
}

// Retrieve the last N message files from the session
function getRecentHistory(sessionDir, n = HISTORY_COUNT) {
  const files = globSync(path.join(sessionDir, '*.md')).sort();
  return files.slice(-n).map(fp => ({ path: fp, content: fs.readFileSync(fp, 'utf8') }));
}

// Extract annotated code blocks and inline commands from markdown
function extractBlocks(markdown) {
  const blocks = [];
  // Fenced code blocks: ```name [annotation]\n...``` 
  const fencedRegex = /```(\w+)(?: +(\w+))?[\r\n]([\s\S]*?)```/g;
  let match;
  while ((match = fencedRegex.exec(markdown))) {
    const [, name, annotation, code] = match;
    blocks.push({ name, annotation, code });
  }
  // If no fenced blocks, detect inline `command` as shell execute
  if (blocks.length === 0) {
    const inlineRegex = /`([^`\n]+)`/g;
    let im;
    while ((im = inlineRegex.exec(markdown))) {
      blocks.push({ name: 'shell', annotation: 'execute', code: im[1] });
    }
  }
  return blocks;
}

// Dispatch a block using the handler registry
function dispatch(block) {
  const handlersText = safeRead(HANDLERS_FILE);
  const handlers = YAML.parse(handlersText);
  const meta = handlers[block.name];
  if (!meta) {
    return `Unknown handler: ${block.name}`;
  }
  const cmd = meta.template.replace('{code}', block.code);
  try {
    return execSync(cmd, { shell: DEFAULT_SHELL, encoding: 'utf8' });
  } catch (err) {
    return err.stdout || err.stderr || err.message;
  }
}

// Build the LLM prompt by assembling all structured inputs
function buildPrompt(newUserMessage) {
  const sessionDir = ensureSessionDir();

  // 1. Prompt template
  const promptTmpl = safeRead(PROMPT_TEMPLATE_FILE).trim();
  const tmplBlock = `#### Path: ${PROMPT_TEMPLATE_FILE}\n` +
                    `\`\`\`text\n${promptTmpl}\n\`\`\``;

  // 2. Handlers registry
  const handlersText = safeRead(HANDLERS_FILE).trim();
  const handlersBlock = `#### Path: ${HANDLERS_FILE}\n` +
                        `\`\`\`yaml\n${handlersText}\n\`\`\``;

  // 3. Memory store
  const memoryText = safeRead(MEMORY_FILE).trim();
  const memoryBlock = `#### Path: ${MEMORY_FILE}\n` +
                      `\`\`\`yaml\n${memoryText}\n\`\`\``;

  // 4. Environment info
  const envInfo = JSON.stringify({ platform: process.platform, shell: IS_WINDOWS ? 'powershell.exe' : '/bin/bash' }, null, 2);
  const envBlock = `#### Environment\n` +
                   `\`\`\`json\n${envInfo}\n\`\`\``;

  // 5. Session history
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
        config: { responseMimeType: 'text/plain', systemInstruction: [{ text: 'You are prime. A highly capable intelligent assistant.' }] },
        contents: [{ role: 'user', parts: [{ text: prompt }] }]
      });

      process.stdout.write('\nAssistant: ');
      for await (const chunk of responseStream) {
        process.stdout.write(chunk.text);
        assistantReply += chunk.text;
      }
      console.log();
    } catch (err) {
      console.error('LLM Error:', err.message);
    }

    // Log assistant reply
    logMessage('assistant', assistantReply);

    // Execute annotated blocks or inline commands
    const blocks = extractBlocks(assistantReply);
    for (const block of blocks) {
      const result = dispatch(block);
      logMessage('system', '```text\n' + result.trim() + '\n```');
      console.log('\n[System Output]\n' + result);
    }
  }
}

main().catch(console.error);
