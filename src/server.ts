import express from 'express';
import yaml from 'js-yaml';
import fs from 'fs';
import { BenchmarkTask } from './types';
import { TaskRunner } from './taskRunner';

const app = express();
app.use(express.json());

let currentTaskRunner: TaskRunner | null = null;

app.post('/start', async (req, res) => {
  const { taskFile } = req.body;
  if (!taskFile) {
    return res.status(400).json({ error: 'taskFile is required' });
  }

  try {
    const fileContents = fs.readFileSync(taskFile, 'utf8');
    const task = yaml.load(fileContents) as BenchmarkTask;
    
    if (currentTaskRunner) {
      await currentTaskRunner.teardown();
    }

    currentTaskRunner = new TaskRunner(task);
    await currentTaskRunner.setup();
    
    res.json({
      message: 'Task started successfully',
      prompt: currentTaskRunner.getPrompt()
    });
  } catch (error: any) {
    res.status(500).json({ error: error.message });
  }
});

app.post('/execute', async (req, res) => {
  if (!currentTaskRunner) {
    return res.status(400).json({ error: 'No task is currently running' });
  }

  const { command } = req.body;
  if (!command) {
    return res.status(400).json({ error: 'command is required' });
  }

  try {
    const result = await currentTaskRunner.runAgentCommand(command);
    res.json(result);
  } catch (error: any) {
    res.status(500).json({ error: error.message });
  }
});

app.post('/evaluate', async (req, res) => {
  if (!currentTaskRunner) {
    return res.status(400).json({ error: 'No task is currently running' });
  }

  try {
    const passed = await currentTaskRunner.evaluate();
    await currentTaskRunner.teardown();
    currentTaskRunner = null;
    
    res.json({ passed });
  } catch (error: any) {
    res.status(500).json({ error: error.message });
  }
});

const PORT = process.env.PORT || 3000;
app.listen(PORT, () => {
  console.log(`Agent Benchmark API Server running on port ${PORT}`);
});
