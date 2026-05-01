import React from 'react';
import type { TaskInfo } from '../types';

const statusColors: Record<string, string> = {
  Pending: '#737373',
  Running: '#2c7de8',
  Completed: '#2d8f5e',
  Failed: '#e84d2c',
};

interface TaskProgressProps {
  tasks: TaskInfo[];
  onCancel?: (id: string) => void;
  onRetry?: (id: string) => void;
}

export const TaskProgress: React.FC<TaskProgressProps> = ({ tasks, onCancel, onRetry }) => {
  return (
    <div className="hestia-task-progress" style={{ fontSize: '14px' }}>
      {tasks.map((task) => (
        <div key={task.id} style={{ padding: '8px 0', borderBottom: '1px solid #e5e5e5' }}>
          <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
            <div>
              <strong>{task.name}</strong>
              <span style={{ fontSize: '12px', color: '#737373', marginLeft: '8px' }}>
                [{task.conductorId}]
              </span>
            </div>
            <span
              style={{
                fontSize: '12px',
                color: statusColors[task.status],
                fontWeight: 500,
              }}
            >
              {task.status}
            </span>
          </div>

          {task.status === 'Running' && (
            <div style={{ marginTop: '4px' }}>
              <div
                style={{
                  width: '100%',
                  height: '6px',
                  backgroundColor: '#e5e5e5',
                  borderRadius: '3px',
                  overflow: 'hidden',
                }}
              >
                <div
                  style={{
                    width: `${task.progress}%`,
                    height: '100%',
                    backgroundColor: '#2c7de8',
                    borderRadius: '3px',
                    transition: 'width 0.3s ease',
                  }}
                />
              </div>
              <span style={{ fontSize: '11px', color: '#737373' }}>{task.progress}%</span>
            </div>
          )}

          <div style={{ display: 'flex', gap: '4px', marginTop: '4px' }}>
            {task.status === 'Running' && onCancel && (
              <button onClick={() => onCancel(task.id)} style={{ fontSize: '11px' }}>
                Cancel
              </button>
            )}
            {task.status === 'Failed' && onRetry && (
              <button onClick={() => onRetry(task.id)} style={{ fontSize: '11px' }}>
                Retry
              </button>
            )}
          </div>
        </div>
      ))}
    </div>
  );
};