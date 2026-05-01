import * as vscode from 'vscode';
import { ConductorManager } from '../conductorManager';

export class AgentListView implements vscode.TreeDataProvider<AgentItem> {
  private _onDidChangeTreeData = new vscode.EventEmitter<AgentItem | undefined>();
  readonly onDidChangeTreeData = this._onDidChangeTreeData.event;

  constructor(private manager: ConductorManager) {}

  refresh(): void {
    this._onDidChangeTreeData.fire(undefined);
  }

  getTreeItem(element: AgentItem): vscode.TreeItem {
    return element;
  }

  getChildren(element?: AgentItem): AgentItem[] {
    if (!element) {
      const conductors = this.manager.getAllConductors();
      return conductors
        .filter((c) => c.status === 'Online')
        .map((c) => new AgentItem(c.id, c.name, c.status));
    }
    return [];
  }
}

class AgentItem extends vscode.TreeItem {
  constructor(id: string, name: string, status: string) {
    super(name, vscode.TreeItemCollapsibleState.None);
    this.description = status;
    this.tooltip = `Agent: ${name} (${status})`;
    this.contextValue = id;
  }
}