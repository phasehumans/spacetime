export interface BenchmarkTask {
  id: string;
  name: string;
  description: string;
  base_image: string;
  setup_script: string;
  prompt: string;
  validation_script: string;
}

export interface ExecutionResult {
  stdout: string;
  stderr: string;
  exitCode: number;
}
