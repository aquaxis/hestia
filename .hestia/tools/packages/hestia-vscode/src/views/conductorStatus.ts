import * as vscode from 'vscode';
import { ConductorManager, ConductorInfo } from '../conductorManager';

export class ConductorStatusView implements vscode.TreeDataProvider<ConductorItem> {
  private _onDidChangeTreeData = new vscode.EventEmitter<ConductorItem | undefined>();
  readonly onDidChangeTreeData = this._onDidChangeTreeData.event;

  constructor(private manager: ConductorManager) {}

  refresh(): void {
    this._onDidChangeTreeData.fire(undefined);
  }

  getTreeItem(element: ConductorItem): vscode.TreeItem {
    return element;
  }

  getChildren(element?: ConductorItem): ConductorItem[] {
    if (!element) {
      return this.manager.getAllConductors().map(
        (c) => new ConductorItem(c, vscode.TreeItemCollapsibleState.None)
      );
    }
    return [];
  }
}

class ConductorItem extends vscode.TreeItem {
  constructor(conductor: ConductorInfo, collapsibleState: vscode.TreeItemCollapsibleState) {
    super(conductor.name, collapsibleState);
    this.description = conductor.status;
    this.tooltip = `${conductor.name} v${conductor.version} - ${conductor.status}`;

    const iconColor = conductor.status === 'Online'
      ? new vscode.ThemeIcon('circle-filled', new vscode.ThemeColor('testing.iconPassed'))
      : conductor.status === 'Degraded'
        ? new vscode.ThemeIcon('circle-filled', new vscode.ThemeColor('testing.iconQueued'))
        : new vscode.ThemeIcon('circle-outline');

    this.iconPath = iconColor;
    this.contextValue = conductor.status;
  }
}