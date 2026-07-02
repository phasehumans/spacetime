import 'dotenv/config';
import OpenAI from 'openai';
import { GoogleGenerativeAI } from '@google/generative-ai';
import fs from 'fs';
import path from 'path';
import yaml from 'js-yaml';
import { BenchmarkTask } from './types';
import { TaskRunner } from './taskRunner';
import chalk from 'chalk';
import boxen from 'boxen';
import ora from 'ora';
import figlet from 'figlet';
import { select } from '@inquirer/prompts';

const delay = (ms: number) => new Promise(res => setTimeout(res, ms));

async function withRetry<T>(fn: () => Promise<T>, maxRetries = 5): Promise<T> {
  let attempt = 0;
  while (true) {
    try {
      return await fn();
    } catch (err: any) {
      if (attempt >= maxRetries) throw err;
      if (err.message && (err.message.includes('429') || err.status === 429)) {
        attempt++;
        
        let waitTime = attempt * 15000;
        // Parse the required retry time from Gemini error if it exists
        const match = err.message.match(/retry in ([\d\.]+)s/);
        if (match && match[1]) {
          waitTime = Math.ceil(parseFloat(match[1])) * 1000 + 2000; // Exact wait + 2s buffer
        }
        
        console.log(chalk.yellow(`\n[Warning] Rate limit hit. Waiting ${waitTime/1000}s before retrying (Attempt ${attempt}/${maxRetries})...`));
        await delay(waitTime);
      } else {
        throw err;
      }
    }
  }
}

async function runTask(taskFile: string, isBatch: boolean = false): Promise<boolean> {
  const useGemini = !!process.env.GEMINI_API_KEY;
  const useOpenAI = !!process.env.OPENAI_API_KEY;

  if (!useGemini && !useOpenAI) {
    console.error(chalk.grey('[ERROR] Please set either GEMINI_API_KEY or OPENAI_API_KEY environment variable.'));
    process.exit(1);
  }

  const providerName = useGemini ? 'Gemini 3.5 Flash' : 'OpenAI GPT-4o';
  
  if (!isBatch) {
    console.log(chalk.dim(`[Provider] `) + chalk.white(providerName));
  }

  const fileContents = fs.readFileSync(taskFile, 'utf8');
  const task = yaml.load(fileContents) as BenchmarkTask;
  const runner = new TaskRunner(task);

  console.log(chalk.dim(`\n[Task]     `) + chalk.white(task.name));
  
  const initSpinner = ora({
    text: chalk.dim('Initializing environment...'),
    color: 'gray'
  }).start();
  
  const startTime = Date.now();
  let passed = false;
  
  try {
    await runner.setup();
    initSpinner.succeed(chalk.dim(`Environment initialized: `) + chalk.white(task.id));
    
    const systemPrompt = `You are an AI agent evaluating a software engineering task in a remote terminal. 
You can execute commands by wrapping them in an <execute> block. 
Example:
<execute>ls -la</execute>
When you are completely finished with the task and ready for evaluation, output a <submit></submit> block.`;

    let openaiMessages: any[] = [];
    let geminiMessages: any[] = [];

    if (useGemini) {
      geminiMessages.push({
        role: 'user',
        parts: [{ text: `${systemPrompt}\n\nTask:\n${runner.getPrompt()}` }]
      });
    } else {
      openaiMessages.push({ role: 'system', content: systemPrompt });
      openaiMessages.push({ role: 'user', content: runner.getPrompt() });
    }

    console.log('\n' + boxen(chalk.white(runner.getPrompt()), { 
      padding: 1, 
      title: chalk.grey(' Target Objective '), 
      titleAlignment: 'center',
      borderStyle: 'single', 
      borderColor: 'gray'
    }));

    let turns = 0;
    const MAX_TURNS = 15;

    let openaiClient: OpenAI | null = null;
    let geminiClient: GoogleGenerativeAI | null = null;
    let geminiModel: any = null;

    if (useGemini) {
      geminiClient = new GoogleGenerativeAI(process.env.GEMINI_API_KEY!);
      geminiModel = geminiClient.getGenerativeModel({ model: 'gemini-3.5-flash' });
    } else {
      openaiClient = new OpenAI();
    }

    while (turns < MAX_TURNS) {
      turns++;
      
      const thinkSpinner = ora({
        text: chalk.dim(`[Turn ${turns}/${MAX_TURNS}] Agent is thinking...`),
        color: 'gray'
      }).start();
      let content = '';

      try {
        if (useGemini) {
          const chat = geminiModel.startChat({ history: geminiMessages.slice(0, -1) });
          const lastMsg = geminiMessages[geminiMessages.length - 1];
          const response = await withRetry(() => chat.sendMessage(lastMsg.parts[0].text));
          content = response.response.text() || '';
          geminiMessages.push({ role: 'model', parts: [{ text: content }] });
        } else {
          const response = await withRetry(() => openaiClient!.chat.completions.create({
            model: 'gpt-4o',
            messages: openaiMessages,
          }));
          content = response.choices[0].message.content || '';
          openaiMessages.push({ role: 'assistant', content });
        }
        thinkSpinner.stop();
      } catch (err) {
        thinkSpinner.fail(chalk.grey('[Error] Agent connection failed.'));
        throw err;
      }

      console.log(boxen(chalk.grey(content), { 
        padding: 1, 
        title: chalk.grey(` Agent Thought (Turn ${turns}) `),
        borderStyle: 'single',
        borderColor: 'gray'
      }));

      if (content.includes('<submit>')) {
        console.log(chalk.white('\n[Notice] Agent initiated evaluation sequence...'));
        break;
      }

      const executeMatch = content.match(/<execute>([\s\S]*?)<\/execute>/);
      let userReply = '';
      if (executeMatch && executeMatch[1]) {
        const command = executeMatch[1].trim();
        const execSpinner = ora({
          text: chalk.dim(`Executing: `) + chalk.grey(command),
          color: 'gray'
        }).start();
        const result = await runner.runAgentCommand(command);
        execSpinner.stop();
        
        let outText = '';
        if (result.stdout) outText += `${chalk.dim('stdout:')}\n${chalk.white(result.stdout)}\n`;
        if (result.stderr) outText += `${chalk.dim('stderr:')}\n${chalk.grey(result.stderr)}\n`;
        outText += `\n${chalk.dim('exit code:')} ${chalk.white(result.exitCode)}`;
                          
        console.log(boxen(outText, { 
          padding: 1, 
          title: chalk.grey(` > ${command} `), 
          borderStyle: 'single', 
          borderColor: 'gray'
        }));
        
        userReply = `Command Result:\nExit Code: ${result.exitCode}\nStdout:\n${result.stdout}\nStderr:\n${result.stderr}`;
      } else {
        console.log(chalk.dim('[Warning] No command provided. Nudging agent...'));
        userReply = 'You must provide an <execute>...</execute> block to run a command, or <submit></submit> to finish.';
      }

      if (useGemini) {
        geminiMessages.push({ role: 'user', parts: [{ text: userReply }] });
      } else {
        openaiMessages.push({ role: 'user', content: userReply });
      }
    }

    if (turns >= MAX_TURNS) {
      console.log(chalk.grey(`\n[Warning] Reached maximum turns (${MAX_TURNS}). Forcing evaluation.`));
    }

    const evalSpinner = ora({ text: chalk.dim('Evaluating final state...'), color: 'gray' }).start();
    passed = await runner.evaluate();
    evalSpinner.stop();
    
    const duration = ((Date.now() - startTime) / 1000).toFixed(1);
    
    const summaryHeader = chalk.white(' SPACETIME LOG ');
    const summaryContent = 
      `${chalk.dim('Task:')}        ${chalk.white(task.name)}\n` +
      `${chalk.dim('Provider:')}    ${chalk.white(providerName)}\n` +
      `${chalk.dim('Turns:')}       ${chalk.white(turns)}\n` +
      `${chalk.dim('Duration:')}    ${chalk.white(duration + 's')}\n\n` +
      (passed 
        ? chalk.white('   [ PASS ]') 
        : chalk.grey('   [ FAIL ]'));
        
    console.log('\n' + boxen(summaryContent, { 
      padding: { top: 1, bottom: 1, left: 3, right: 3 }, 
      title: summaryHeader,
      titleAlignment: 'center',
      borderStyle: 'single', 
      borderColor: 'gray'
    }));

  } catch (err: any) {
    console.error(chalk.grey('\n[Critical Failure]'), chalk.dim(err.message));
  } finally {
    const teardownSpinner = ora({ text: chalk.dim('Tearing down environment...'), color: 'gray' }).start();
    await runner.teardown();
    teardownSpinner.succeed(chalk.dim('Environment destroyed.'));
  }
  
  return passed;
}

async function main() {
  // Generate Spacetime logo
  const logo = figlet.textSync('SPACETIME', { font: 'Standard' });
  console.log(chalk.grey(logo));
  console.log(chalk.grey('A benchmark for evaluating AI agents on interactive terminal tasks.\n'));

  const taskFile = process.argv[2];
  if (taskFile) {
    await runTask(taskFile);
    return;
  }

  // Interactive selection
  const tasksDir = path.join(__dirname, '../tasks');
  const files = fs.readdirSync(tasksDir).filter(f => f.endsWith('.yaml')).sort();
  
  const choices = files.map(f => ({ name: f, value: path.join(tasksDir, f) }));
  choices.push({ name: 'Run All Tests', value: 'ALL' });

  let selectedTask;
  try {
    selectedTask = await select({
      message: 'Select a task to run:',
      choices: choices,
      theme: {
        prefix: '>',
        style: {
          highlight: (text: string) => chalk.bold.white(text),
          answer: (text: string) => chalk.grey(text),
          message: (text: string) => chalk.grey(text)
        }
      }
    });
  } catch (err: any) {
    if (err.name === 'ExitPromptError') {
      console.log(chalk.grey('\nExiting...'));
      process.exit(0);
    }
    throw err;
  }

  if (!selectedTask) {
    console.log(chalk.grey('Exiting...'));
    return;
  }

  if (selectedTask === 'ALL') {
    console.log(chalk.white(`\n[Info] Running all ${files.length} tests sequentially...`));
    let passedCount = 0;
    
    for (const file of files) {
      console.log(chalk.grey(`\n-----------------------------------------------------`));
      const passed = await runTask(path.join(tasksDir, file), true);
      if (passed) passedCount++;
    }
    
    console.log(chalk.white('\n[Final Results]'));
    console.log(chalk.grey(`Passed: ${passedCount}/${files.length}`));
    
  } else {
    await runTask(selectedTask);
  }
}

main();
