import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import { useEffect, useState } from 'react';
import { Shield, Key, Check, AlertCircle, Copy } from 'lucide-react';
import '../styles/license.css';

interface LicenseStatus {
  activated: boolean;
  machineId: string;
  fullMachineId?: string;
}

export function LicenseGate({ children }: { children: React.ReactNode }) {
  const [status, setStatus] = useState<LicenseStatus | null>(null);
  const [loading, setLoading] = useState(true);
  const [code, setCode] = useState('');
  const [error, setError] = useState('');
  const [activating, setActivating] = useState(false);
  const [activated, setActivated] = useState(false);

  const checkStatus = async () => {
    try {
      const [s, fullId] = await Promise.all([
        invoke<LicenseStatus>('get_license_status'),
        invoke<string>('get_full_machine_id'),
      ]);
      setStatus({ ...s, fullMachineId: fullId });
      if (s.activated) {
        setActivated(true);
      }
    } catch (e) {
      console.error('License check failed:', e);
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    checkStatus();

    // Listen for license:required event from backend
    const unlisten = listen('license:required', () => {
      setLoading(false);
      setActivated(false);
    });

    return () => {
      unlisten.then(fn => fn());
    };
  }, []);

  const handleActivate = async () => {
    const trimmed = code.trim();
    if (!trimmed) {
      setError('请输入授权码');
      return;
    }
    setActivating(true);
    setError('');
    try {
      await invoke('activate_license', { licenseCode: trimmed });
      setActivated(true);
      setError('');
    } catch (e: any) {
      setError(typeof e === 'string' ? e : '激活失败，授权码无效');
    } finally {
      setActivating(false);
    }
  };

  if (loading) {
    return (
      <div className="license-screen">
        <div className="license-loading">加载中...</div>
      </div>
    );
  }

  if (activated) {
    return <>{children}</>;
  }

  return (
    <div className="license-screen">
      <div className="license-card">
        <div className="license-icon">
          <Shield size={48} />
        </div>
        <h1>Cockpit Tools</h1>
        <p className="license-subtitle">请输入授权码以激活应用</p>

        <div className="license-machine">
          <span className="license-label">机器标识</span>
          <code className="license-machine-id">{status?.machineId || '...'}</code>
        </div>

        {status?.fullMachineId && (
          <div className="license-machine-full">
            <code className="license-full-id">{status.fullMachineId}</code>
            <button
              className="license-copy-btn"
              onClick={() => navigator.clipboard.writeText(status!.fullMachineId!)}
              title="复制完整机器ID"
            >
              <Copy size={14} />
            </button>
          </div>
        )}

        <p className="license-hint">将此机器标识发给管理员获取授权码</p>

        <div className="license-input-group">
          <Key size={18} className="license-input-icon" />
          <input
            type="text"
            className="license-input"
            placeholder="输入授权码..."
            value={code}
            onChange={(e) => {
              setCode(e.target.value);
              setError('');
            }}
            onKeyDown={(e) => e.key === 'Enter' && handleActivate()}
            disabled={activating}
            autoFocus
          />
        </div>

        {error && (
          <div className="license-error">
            <AlertCircle size={16} />
            <span>{error}</span>
          </div>
        )}

        <button
          className="license-btn"
          onClick={handleActivate}
          disabled={activating || !code.trim()}
        >
          {activating ? (
            <>验证中...</>
          ) : (
            <>
              <Check size={18} />
              激活
            </>
          )}
        </button>
      </div>
    </div>
  );
}
