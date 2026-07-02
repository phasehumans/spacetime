import Docker from 'dockerode';
import { ExecutionResult } from './types';

const docker = new Docker();

export class EnvironmentManager {
  private container: Docker.Container | null = null;
  private imagePulled: boolean = false;

  constructor(private image: string) {}

  async initialize(): Promise<void> {
    if (!this.imagePulled) {
      await new Promise((resolve, reject) => {
        docker.pull(this.image, (err: Error, stream: any) => {
          if (err) return reject(err);
          docker.modem.followProgress(stream, (onFinishedErr: Error, output: any) => {
            if (onFinishedErr) return reject(onFinishedErr);
            resolve(output);
          });
        });
      });
      this.imagePulled = true;
    }

    this.container = await docker.createContainer({
      Image: this.image,
      Cmd: ['/bin/sh', '-c', 'sleep infinity'],
      Tty: true,
      HostConfig: {
        AutoRemove: true,
      }
    });

    await this.container.start();
  }

  async executeCommand(command: string): Promise<ExecutionResult> {
    if (!this.container) throw new Error("Container not initialized");

    const exec = await this.container.exec({
      Cmd: ['/bin/sh', '-c', command],
      AttachStdout: true,
      AttachStderr: true,
    });

    const stream = await exec.start({ Detach: false });

    return new Promise((resolve, reject) => {
      let stdout = '';
      let stderr = '';

      stream.on('data', (chunk) => {
        stdout += chunk.toString('utf-8');
      });

      stream.on('end', async () => {
        const inspectInfo = await exec.inspect();
        resolve({
          stdout: stdout.trim(),
          stderr: stderr.trim(),
          exitCode: inspectInfo.ExitCode || 0
        });
      });

      stream.on('error', (err) => reject(err));
    });
  }

  async destroy(): Promise<void> {
    if (this.container) {
      await this.container.stop();
      this.container = null;
    }
  }
}
