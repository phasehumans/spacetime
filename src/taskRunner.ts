import { BenchmarkTask, ExecutionResult } from './types';
import { EnvironmentManager } from './docker';

export class TaskRunner {
  private env: EnvironmentManager;

  constructor(private task: BenchmarkTask) {
    this.env = new EnvironmentManager(task.base_image);
  }

  async setup(): Promise<void> {
    await this.env.initialize();
    
    if (this.task.setup_script) {
      const result = await this.env.executeCommand(this.task.setup_script);
      if (result.exitCode !== 0) {
        throw new Error(`Setup script failed: ${result.stderr}`);
      }
    }
  }

  async evaluate(): Promise<boolean> {
    const result = await this.env.executeCommand(this.task.validation_script);
    
    if (result.exitCode === 0) {
      return true;
    } else {
      return false;
    }
  }

  async teardown(): Promise<void> {
    await this.env.destroy();
  }

  async runAgentCommand(command: string): Promise<ExecutionResult> {
    return await this.env.executeCommand(command);
  }

  getPrompt(): string {
    return this.task.prompt;
  }
}
