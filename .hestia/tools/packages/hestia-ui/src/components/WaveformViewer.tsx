import React, { useRef, useEffect, useState } from 'react';
import type { WaveformSignal } from '../types';

interface WaveformViewerProps {
  signals: WaveformSignal[];
  format?: 'VCD' | 'FST' | 'GHW' | 'EVCD';
  width?: number;
  height?: number;
}

export const WaveformViewer: React.FC<WaveformViewerProps> = ({
  signals,
  format = 'VCD',
  width = 800,
  height = 400,
}) => {
  const canvasRef = useRef<HTMLCanvasElement>(null);
  const [selectedSignals, setSelectedSignals] = useState<Set<string>>(new Set());

  useEffect(() => {
    const canvas = canvasRef.current;
    if (!canvas) return;
    const ctx = canvas.getContext('2d');
    if (!ctx) return;

    ctx.fillStyle = '#1a1a1a';
    ctx.fillRect(0, 0, width, height);

    ctx.fillStyle = '#525252';
    ctx.font = '12px monospace';
    ctx.fillText(`Waveform Viewer (${format})`, 10, 20);
    ctx.fillText(`${signals.length} signals loaded`, 10, 40);

    const rowHeight = 24;
    const nameWidth = 160;
    const visibleSignals = signals.filter((s) =>
      selectedSignals.size === 0 || selectedSignals.has(s.id)
    );

    visibleSignals.slice(0, Math.floor((height - 60) / rowHeight)).forEach((signal, i) => {
      const y = 60 + i * rowHeight;
      ctx.fillStyle = '#d4d4d4';
      ctx.fillText(signal.displayName, 10, y + 14);
      ctx.strokeStyle = '#2d8f5e';
      ctx.beginPath();
      ctx.moveTo(nameWidth, y + 8);
      ctx.lineTo(nameWidth + (width - nameWidth - 20), y + 8);
      ctx.stroke();
    });
  }, [signals, selectedSignals, format, width, height]);

  const toggleSignal = (id: string) => {
    setSelectedSignals((prev) => {
      const next = new Set(prev);
      if (next.has(id)) {
        next.delete(id);
      } else {
        next.add(id);
      }
      return next;
    });
  };

  return (
    <div className="hestia-waveform-viewer" style={{ width, height }}>
      <div style={{ display: 'flex', gap: '4px', flexWrap: 'wrap', marginBottom: '4px' }}>
        {signals.slice(0, 20).map((signal) => (
          <button
            key={signal.id}
            onClick={() => toggleSignal(signal.id)}
            style={{
              fontSize: '11px',
              padding: '2px 6px',
              backgroundColor: selectedSignals.has(signal.id) ? '#e84d2c' : '#404040',
              color: '#fff',
              border: 'none',
              borderRadius: '3px',
              cursor: 'pointer',
            }}
          >
            {signal.displayName}
          </button>
        ))}
      </div>
      <canvas
        ref={canvasRef}
        width={width}
        height={height - 30}
        style={{ border: '1px solid #404040', borderRadius: '4px' }}
      />
    </div>
  );
};