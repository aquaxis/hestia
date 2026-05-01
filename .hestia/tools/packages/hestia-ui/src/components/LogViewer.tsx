import React, { useRef, useEffect } from 'react';
import type { LogEntry } from '../types';

const levelColors: Record<string, string> = {
  Trace: '#737373',
  Debug: '#525252',
  Info: '#2d8f5e',
  Warn: '#e8a62c',
  Error: '#e84d2c',
};

interface LogViewerProps {
  logs: LogEntry[];
  maxLines?: number;
  autoScroll?: boolean;
  filter?: (entry: LogEntry) => boolean;
}

export const LogViewer: React.FC<LogViewerProps> = ({
  logs,
  maxLines = 1000,
  autoScroll = true,
  filter,
}) => {
  const containerRef = useRef<HTMLDivElement>(null);
  const filteredLogs = filter ? logs.filter(filter) : logs;
  const displayLogs = filteredLogs.slice(-maxLines);

  useEffect(() => {
    if (autoScroll && containerRef.current) {
      containerRef.current.scrollTop = containerRef.current.scrollHeight;
    }
  }, [displayLogs.length, autoScroll]);

  return (
    <div
      ref={containerRef}
      className="hestia-log-viewer"
      style={{
        fontFamily: 'monospace',
        fontSize: '12px',
        height: '400px',
        overflowY: 'auto',
        backgroundColor: '#1a1a1a',
        color: '#d4d4d4',
        padding: '8px',
        borderRadius: '4px',
      }}
    >
      {displayLogs.map((log, i) => (
        <div key={i} style={{ lineHeight: '1.6' }}>
          <span style={{ color: '#737373' }}>[{log.timestamp}]</span>{' '}
          <span style={{ color: levelColors[log.level] || '#d4d4d4' }}>
            {log.level.padEnd(5)}
          </span>{' '}
          <span style={{ color: '#525252' }}>{log.source}:</span>{' '}
          {log.message}
          {log.traceId && (
            <span style={{ color: '#525252', marginLeft: '8px' }}>
              trace:{log.traceId}
            </span>
          )}
        </div>
      ))}
    </div>
  );
};