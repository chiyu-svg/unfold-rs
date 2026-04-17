import { useState } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { open } from '@tauri-apps/plugin-dialog';
import './App.css';

interface LogEntry {
  message: string;
  type: 'info' | 'success' | 'error';
  timestamp: Date;
}

function App() {
  const [sourceDir, setSourceDir] = useState('');
  const [targetDir, setTargetDir] = useState('');
  const [moveFiles, setMoveFiles] = useState(false);
  const [dryRun, setDryRun] = useState(false);
  const [conflictStrategy, setConflictStrategy] = useState<'skip' | 'overwrite' | 'rename'>('skip');
  const [cleanup, setCleanup] = useState(true);
  const [isExecuting, setIsExecuting] = useState(false);
  const [logs, setLogs] = useState<LogEntry[]>([]);

  const addLog = (message: string, type: LogEntry['type'] = 'info') => {
    setLogs((prev) => [...prev, { message, type, timestamp: new Date() }]);
  };

  // 选择源目录
  const handleSelectSource = async () => {
    try {
      const selected = await open({
        directory: true,
        multiple: false,
        title: '选择源目录',
      });
      
      if (selected && typeof selected === 'string') {
        setSourceDir(selected);
        addLog(`已选择源目录: ${selected}`, 'info');
      }
    } catch (error) {
      addLog(`选择目录失败: ${String(error)}`, 'error');
    }
  };

  // 选择目标目录
  const handleSelectTarget = async () => {
    try {
      const selected = await open({
        directory: true,
        multiple: false,
        title: '选择目标目录',
      });
      
      if (selected && typeof selected === 'string') {
        setTargetDir(selected);
        addLog(`已选择目标目录: ${selected}`, 'info');
      }
    } catch (error) {
      addLog(`选择目录失败: ${String(error)}`, 'error');
    }
  };

  const handleExecute = async () => {
    if (!sourceDir || !targetDir) {
      addLog('请填写源目录和目标目录', 'error');
      return;
    }

    setIsExecuting(true);
    addLog('开始执行文件平铺操作...', 'info');

    try {
      const params = {
        source: sourceDir,
        dest: targetDir,
        moveFiles,
        dryRun,
        conflict: conflictStrategy,
        cleanup,
      }
      console.log('params++++', params);
      const result = await invoke('run', { params });
      addLog(result as string, 'success');
    } catch (error) {
      addLog(String(error), 'error');
    } finally {
      setIsExecuting(false);
    }
  };

  const handleUndo = async () => {
    setIsExecuting(true);
    addLog('开始撤销操作...', 'info');

    try {
      const result = await invoke('undo');
      addLog(result as string, 'success');
    } catch (error) {
      addLog(String(error), 'error');
    } finally {
      setIsExecuting(false);
    }
  };

  const handleViewLog = async () => {
    setIsExecuting(true);
    addLog('查看操作日志...', 'info');

    try {
      const result = await invoke('log');
      addLog(result as string, 'success');
    } catch (error) {
      addLog(String(error), 'error');
    } finally {
      setIsExecuting(false);
    }
  };

  const clearLogs = () => {
    setLogs([]);
  };

  return (
    <div className="container">
      <h1 className="title">📁 Unfold 文件平铺工具</h1>

      <div className="form">
        <div className="form-group">
          <label htmlFor="source">源目录</label>
          <div className="input-with-button">
            <input
              id="source"
              type="text"
              value={sourceDir}
              onChange={(e) => setSourceDir(e.target.value)}
              placeholder="点击浏览按钮选择源目录"
              readOnly={false}
            />
            <button
              type="button"
              className="button button-small"
              onClick={handleSelectSource}
              disabled={isExecuting}
            >
              📁 浏览
            </button>
          </div>
        </div>

        <div className="form-group">
          <label htmlFor="target">目标目录</label>
          <div className="input-with-button">
            <input
              id="target"
              type="text"
              value={targetDir}
              onChange={(e) => setTargetDir(e.target.value)}
              placeholder="点击浏览按钮选择目标目录"
              readOnly={false}
            />
            <button
              type="button"
              className="button button-small"
              onClick={handleSelectTarget}
              disabled={isExecuting}
            >
              📁 浏览
            </button>
          </div>
        </div>

        <div className="checkbox-group">
          <label className="checkbox-label">
            <input
              type="checkbox"
              checked={moveFiles}
              onChange={(e) => setMoveFiles(e.target.checked)}
            />
            移动文件（而非复制）
          </label>

          <label className="checkbox-label">
            <input
              type="checkbox"
              checked={dryRun}
              onChange={(e) => setDryRun(e.target.checked)}
            />
            演习模式（不实际执行）
          </label>

          <label className="checkbox-label">
            <input
              type="checkbox"
              checked={cleanup}
              onChange={(e) => setCleanup(e.target.checked)}
            />
            清理空文件夹
          </label>
        </div>

        <div className="form-group">
          <label htmlFor="conflict">冲突策略</label>
          <select
            id="conflict"
            value={conflictStrategy}
            onChange={(e) => setConflictStrategy(e.target.value as any)}
          >
            <option value="skip">跳过</option>
            <option value="overwrite">覆盖</option>
            <option value="rename">重命名</option>
          </select>
        </div>

        <div className="button-group">
          <button
            className="button button-primary"
            onClick={handleExecute}
            disabled={isExecuting}
          >
            {isExecuting ? '执行中...' : '🚀 开始执行'}
          </button>
          <button
            className="button button-secondary"
            onClick={handleUndo}
            disabled={isExecuting}
          >
            ↩️ 撤销
          </button>
          <button
            className="button button-secondary"
            onClick={handleViewLog}
            disabled={isExecuting}
          >
            📋 查看日志
          </button>
        </div>
      </div>

      <div className="log-section">
        <div className="log-header">
          <h2>操作日志</h2>
          <button className="button-small" onClick={clearLogs}>
            清空日志
          </button>
        </div>
        <div className="log-container">
          {logs.length === 0 ? (
            <p className="log-empty">暂无日志</p>
          ) : (
            logs.map((log, index) => (
              <div key={index} className={`log-entry log-${log.type}`}>
                <span className="log-time">
                  {log.timestamp.toLocaleTimeString()}
                </span>
                <span className="log-message">{log.message}</span>
              </div>
            ))
          )}
        </div>
      </div>
    </div>
  );
}

export default App;
