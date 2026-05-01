import * as vscode from 'vscode';
import { ConductorManager } from './conductorManager';
import { AgentListView } from './views/agentList';
import { ConductorStatusView } from './views/conductorStatus';

let conductorManager: ConductorManager;

export async function activate(context: vscode.ExtensionContext) {
  conductorManager = new ConductorManager();

  const statusView = new ConductorStatusView(conductorManager);
  const agentView = new AgentListView(conductorManager);

  context.subscriptions.push(
    vscode.window.registerTreeDataProvider('hestia-conductor', statusView),
    vscode.window.registerTreeDataProvider('hestia-agents', agentView)
  );

  const commands: [string, (...args: any[]) => any][] = [
    ['hestia.start', () => conductorManager.startAll()],
    ['hestia.stop', () => conductorManager.stopAll()],
    ['hestia.status', () => conductorManager.showStatus()],
    ['hestia.ai', () => conductorManager.startConductor('ai')],
    ['hestia.rtl', () => conductorManager.startConductor('rtl')],
    ['hestia.fpga', () => conductorManager.startConductor('fpga')],
    ['hestia.asic', () => conductorManager.startConductor('asic')],
    ['hestia.pcb', () => conductorManager.startConductor('pcb')],
    ['hestia.hal', () => conductorManager.startConductor('hal')],
    ['hestia.apps', () => conductorManager.startConductor('apps')],
    ['hestia.debug', () => conductorManager.startConductor('debug')],
    ['hestia.rag', () => conductorManager.startConductor('rag')],
    ['hestia.waveform', () => openWaveformViewer(context)],
  ];

  for (const [command, handler] of commands) {
    context.subscriptions.push(
      vscode.commands.registerCommand(command, handler)
    );
  }
}

async function openWaveformViewer(context: vscode.ExtensionContext) {
  const panel = vscode.window.createWebviewPanel(
    'hestiaWaveform',
    'Hestia Waveform Viewer',
    vscode.ViewColumn.Beside,
    { enableScripts: true }
  );
  panel.webview.html = getWaveformHtml();
}

function getWaveformHtml(): string {
  return `<!DOCTYPE html>
<html><head><meta charset="UTF-8">
<style>body { background: #1a1a1a; color: #d4d4d4; font-family: monospace; }</style>
</head><body>
<h2 style="color:#e84d2c">Hestia Waveform Viewer</h2>
<p>Drop a VCD/FST/GHW/EVCD file to view waveforms.</p>
</body></html>`;
}

export function deactivate() {
  conductorManager?.stopAll();
}