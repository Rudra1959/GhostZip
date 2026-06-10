import React from 'react';
import { X } from 'lucide-react';

export function SettingsPanel({
  theme, autoDelete, notifications, onTheme, onAutoDelete, onNotifications, onClose
}: any) {
  return (
    <div className="drawer-overlay fade-in">
      <div className="drawer-panel slide-in-right">
        <header className="drawer-header">
          <h2>Settings</h2>
          <button className="icon-btn" onClick={onClose}><X size={20} /></button>
        </header>
        
        <div className="settings-content">
          <div className="setting-group">
            <label>Theme</label>
            <select className="glass-select" value={theme} onChange={(e) => onTheme(e.target.value)}>
              <option value="dark">Dark Theme</option>
              <option value="light">Light Theme</option>
            </select>
          </div>
          
          <div className="setting-group checkbox-group">
            <label className="checkbox-label">
              <input type="checkbox" checked={autoDelete} onChange={(e) => onAutoDelete(e.target.checked)} />
              <span className="checkbox-text">
                <strong>Auto-delete source archive</strong>
                <span>Automatically delete the original archive file to save space when extraction completes</span>
              </span>
            </label>
          </div>

          <div className="setting-group checkbox-group">
            <label className="checkbox-label">
              <input type="checkbox" checked={notifications} onChange={(e) => onNotifications(e.target.checked)} />
              <span className="checkbox-text">
                <strong>System notifications</strong>
                <span>Show desktop notifications when extraction completes</span>
              </span>
            </label>
          </div>
        </div>
      </div>
    </div>
  );
}
