import * as vscode from 'vscode';
import { execFile } from 'child_process';

export interface ConductorInfo {
  id: string;
  name: string;
  status: 'Online' | 'Offline' | 'Degraded' | 'Upgrading';
  version: string;
}

const CONDUCTOR_IDS = [
  'ai', 'rtl', 'fpga', 'asic', 'pcb', 'hal', 'apps', 'debug', 'rag',
];

export class ConductorManager {
  private conductors: Map<string, ConductorInfo> = new Map();

  constructor() {
    for (const id of CONDUCTOR_IDS) {
      this.conductors.set(id, {
        id,
        name: `${id}-conductor`,
        status: 'Offline',
        version: '0.1.0',
      });
    }
  }

  async startConductor(id: string): Promise<void> {
    try {
      execFile('hestia', ['start', id], (error, stdout, stderr) => {
        if (error) {
          vscode.window.showErrorMessage(`Failed to start ${id}-conductor: ${error.message}`);
          return;
        }
        const info = this.conductors.get(id);
        if (info) {
          info.status = 'Online';
        }
        vscode.window.showInformationMessage(`${id}-conductor started`);
      });
    } catch (err) {
      vscode.window.showErrorMessage(`Error starting ${id}-conductor: ${err}`);
    }
  }

  async startAll(): Promise<void> {
    for (const id of CONDUCTOR_IDS) {
      await this.startConductor(id);
    }
  }

  async stopAll(): Promise<void> {
    for (const [id, info] of this.conductors) {
      info.status = 'Offline';
    }
    vscode.window.showInformationMessage('All conductors stopped');
  }

  async showStatus(): Promise<void> {
    const items = Array.from(this.conductors.values()).map(
      (c) => `${c.name}: ${c.status}`
    );
    vscode.window.showInformationMessage(items.join('\n'));
  }

  getConductor(id: string): ConductorInfo | undefined {
    return this.conductors.get(id);
  }

  getAllConductors(): ConductorInfo[] {
    return Array.from(this.conductors.values());
  }
}