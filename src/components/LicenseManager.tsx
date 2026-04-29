import { invoke } from '@tauri-apps/api/core';
import { useState } from 'react';
import { Key, Copy, Check, AlertCircle } from 'lucide-react';

export function LicenseManager() {
  const [targetId, setTargetId] = useState('');
  const [generatedCode, setGeneratedCode] = useState('');
  const [copied, setCopied] = useState(false);
  const [error, setError] = useState('');

  const handleGenerate = async () => {
    const trimmed = targetId.trim();
    if (!trimmed) {
      setError('请输入对方的完整机器标识');
      return;
    }
    try {
      const code = await invoke<string>('generate_license_code', { targetMachineId: trimmed });
      setGeneratedCode(code);
      setError('');
    } catch (e: any) {
      setError(typeof e === 'string' ? e : '生成失败');
      setGeneratedCode('');
    }
  };

  const handleCopy = () => {
    navigator.clipboard.writeText(generatedCode);
    setCopied(true);
    setTimeout(() => setCopied(false), 2000);
  };

  return (
    <div className="settings-section">
      <h3 className="settings-section-title">授权管理</h3>
      <p className="settings-section-desc">为其他机器生成授权码</p>

      <div className="license-mgr">
        <label className="license-mgr-label">目标机器标识</label>
        <div className="license-mgr-input-row">
          <input
            type="text"
            className="license-mgr-input"
            placeholder="粘贴对方的完整机器标识..."
            value={targetId}
            onChange={(e) => {
              setTargetId(e.target.value);
              setError('');
            }}
          />
          <button className="license-mgr-btn" onClick={handleGenerate}>
            <Key size={16} /> 生成授权码
          </button>
        </div>

        {error && (
          <div className="license-mgr-error">
            <AlertCircle size={14} /> {error}
          </div>
        )}

        {generatedCode && (
          <div className="license-mgr-result">
            <code className="license-mgr-code">{generatedCode}</code>
            <button className="license-mgr-copy" onClick={handleCopy}>
              {copied ? <Check size={16} /> : <Copy size={16} />}
            </button>
          </div>
        )}
      </div>
    </div>
  );
}
