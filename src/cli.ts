import OpenAI from 'openai';
import { GoogleGenerativeAI } from '@google/generative-ai';
import fs from 'fs';
import yaml from 'js-yaml';
import { BenchmarkTask } from './types';
import { TaskRunner } from './taskRunner';
import chalk from 'chalk';
import boxen from 'boxen';
import ora from 'ora';

async function runCli() {
  console.clear();
  console.log(chalk.bold.blue('\n🚀 Spacetime Agent Benchmark Runner\n'));

  const taskFile = process.argv[2];
  if (!taskFile) {
    console.error(chalk.red('✖ Please provide a task file. Usage: npm run evaluate <path-to-task-file>'));
    process.exit(1);
  }

  const useGemini = !!process.env.GEMINI_API_KEY;
  const useOpenAI = !!process.env.OPENAI_API_KEY;

  if (!useGemini && !useOpenAI) {
    console.error(chalk.red('✖ Please set either GEMINI_API_KEY or OPENAI_API_KEY environment variable.'));
    process.exit(1);
  }

  const providerName = useGemini ? 'Gemini 3.5 Flash' : 'OpenAI GPT-4o';
  console.log(chalk.gray(`🤖 Model Provider: ${chalk.white.bold(providerName)}\n`));

  const fileContents = fs.readFileSync(taskFile, 'utf8');
  const task = yaml.load(fileContents) as BenchmarkTask;
  const runner = new TaskRunner(task);

  const initSpinner = ora(`Initializing Task: ${chalk.yellow.bold(task.name)}`).start();
  
  try {
    await runner.setup();
    initSpinner.succeed(`Task environment initialized for ${chalk.yellow(task.id)}`);
    
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

    console.log(boxen(chalk.cyan(runner.getPrompt()), { padding: 1, title: 'Task Prompt', borderStyle: 'round', borderColor: 'cyan' }));

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
      
      const thinkSpinner = ora(chalk.magenta(`Turn ${turns}/${MAX_TURNS}: Agent is thinking...`)).start();
      let content = '';

      try {
        if (useGemini) {
          const chat = geminiModel.startChat({ history: geminiMessages.slice(0, -1) });
          const lastMsg = geminiMessages[geminiMessages.length - 1];
          const response = await chat.sendMessage(lastMsg.parts[0].text);
          content = response.response.text() || '';
          geminiMessages.push({ role: 'model', parts: [{ text: content }] });
        } else {
          const response = await openaiClient!.chat.completions.create({
            model: 'gpt-4o',
            messages: openaiMessages,
          });
          content = response.choices[0].message.content || '';
          openaiMessages.push({ role: 'assistant', content });
        }
        thinkSpinner.stop();
      } catch (err) {
        thinkSpinner.fail(chalk.red('Agent failed to respond.'));
        throw err;
      }

      console.log(chalk.magenta(`\n[Agent Response - Turn ${turns}]`));
      console.log(chalk.white(content));

      if (content.includes('<submit>')) {
        console.log(chalk.green.bold('\n✔ Agent has submitted the task for evaluation.'));
        break;
      }

      const executeMatch = content.match(/<execute>([\s\S]*?)<\/execute>/);
      let userReply = '';
      if (executeMatch && executeMatch[1]) {
        const command = executeMatch[1].trim();
        const execSpinner = ora(chalk.cyan(`Executing: ${command}`)).start();
        const result = await runner.runAgentCommand(command);
        execSpinner.succeed(chalk.cyan(`Executed: ${command}`));
        
        let outputColor = result.exitCode === 0 ? chalk.green : chalk.red;
        
        const outputBox = `Exit Code: ${outputColor(result.exitCode.toString())}\n` +
                          `${chalk.gray('--- Stdout ---')}\n${result.stdout || chalk.gray('(empty)')}\n` +
                          `${chalk.gray('--- Stderr ---')}\n${result.stderr || chalk.gray('(empty)')}`;
                          
        console.log(boxen(outputBox, { padding: 1, borderColor: result.exitCode === 0 ? 'green' : 'red', dimBorder: true }));
        
        userReply = `Command Result:\nExit Code: ${result.exitCode}\nStdout:\n${result.stdout}\nStderr:\n${result.stderr}`;
      } else {
        console.log(chalk.yellow('⚠ No <execute> or <submit> block found. Reminding agent...'));
        userReply = 'You must provide an <execute>...</execute> block to run a command, or <submit></submit> to finish.';
      }

      if (useGemini) {
        geminiMessages.push({ role: 'user', parts: [{ text: userReply }] });
      } else {
        openaiMessages.push({ role: 'user', content: userReply });
      }
    }

    if (turns >= MAX_TURNS) {
      console.log(chalk.red.bold(`\n⚠ Reached maximum turns (${MAX_TURNS}). Forcing evaluation.`));
    }

    const evalSpinner = ora('Evaluating task...').start();
    const passed = await runner.evaluate();
    
    if (passed) {
      evalSpinner.succeed(chalk.green('Evaluation Complete'));
      console.log('\n' + boxen(chalk.green.bold('🎉 FINAL SCORE: PASS ✅'), { padding: 1, borderStyle: 'double', borderColor: 'green' }));
    } else {
      evalSpinner.fail(chalk.red('Evaluation Complete'));
      console.log('\n' + boxen(chalk.red.bold('💥 FINAL SCORE: FAIL ❌'), { padding: 1, borderStyle: 'double', borderColor: 'red' }));
    }

  } catch (err: any) {
    console.error(chalk.red('\n✖ Error during execution:'), err.message);
  } finally {
    const teardownSpinner = ora('Tearing down environment...').start();
    await runner.teardown();
    teardownSpinner.succeed('Environment destroyed.');
  }
}

runCli();
