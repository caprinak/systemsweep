// src/components/views/Settings.tsx
import React from 'react';
import { useAppStore } from '../../stores/appStore';
import { Card, CardHeader, CardTitle, CardDescription } from '../ui/Card';
import { Toggle } from '../ui/Toggle';
import { Button } from '../ui/Button';
import { RefreshCw } from 'lucide-react';

export function SettingsView() {
    const { settings, updateSettings, reset } = useAppStore();

    return (
        <div className="max-w-3xl mx-auto space-y-6">
            <div className="flex items-center justify-between">
                <h1 className="text-2xl font-bold text-dark-900 dark:text-dark-50">Settings</h1>
                <Button variant="ghost" onClick={reset} leftIcon={<RefreshCw className="w-4 h-4" />}>
                    Reset App State
                </Button>
            </div>

            <Card>
                <CardHeader>
                    <CardTitle>General</CardTitle>
                </CardHeader>
                <div className="space-y-6">
                    <div className="flex items-center justify-between">
                        <div>
                            <p className="font-medium">System Tray</p>
                            <p className="text-sm text-dark-500">Minimize to system tray instead of closing</p>
                        </div>
                        <Toggle
                            checked={settings.minimizeToTray}
                            onChange={(v) => updateSettings({ minimizeToTray: v })}
                        />
                    </div>
                    <div className="flex items-center justify-between">
                        <div>
                            <p className="font-medium">Start on Boot</p>
                            <p className="text-sm text-dark-500">Launch CleanDesk when computer starts</p>
                        </div>
                        <Toggle
                            checked={settings.startWithSystem}
                            onChange={(v) => updateSettings({ startWithSystem: v })}
                        />
                    </div>
                    <div className="flex items-center justify-between">
                        <div>
                            <p className="font-medium">Notifications</p>
                            <p className="text-sm text-dark-500">Show desktop notifications for scan results</p>
                        </div>
                        <Toggle
                            checked={settings.notifications}
                            onChange={(v) => updateSettings({ notifications: v })}
                        />
                    </div>
                </div>
            </Card>

            <Card>
                <CardHeader>
                    <CardTitle>Safety & Cleaning</CardTitle>
                    <CardDescription>Configure how files are deleted</CardDescription>
                </CardHeader>
                <div className="space-y-6">
                    <div className="flex items-center justify-between">
                        <div>
                            <p className="font-medium">Dry Run Mode</p>
                            <p className="text-sm text-dark-500">Simulate deletion without removing files</p>
                        </div>
                        <Toggle
                            checked={settings.dryRunDefault}
                            onChange={(v) => updateSettings({ dryRunDefault: v })}
                        />
                    </div>
                    <div className="flex items-center justify-between">
                        <div>
                            <p className="font-medium">Create Restore Points</p>
                            <p className="text-sm text-dark-500">Backup files before deleting (Recommended)</p>
                        </div>
                        <Toggle
                            checked={settings.createRestorePoints}
                            onChange={(v) => updateSettings({ createRestorePoints: v })}
                        />
                    </div>
                    <div className="flex items-center justify-between">
                        <div>
                            <p className="font-medium text-danger-600">Secure Delete</p>
                            <p className="text-sm text-dark-500">Overwrite files before deletion (Slower, unrecoverable)</p>
                        </div>
                        <Toggle
                            checked={settings.secureDelete}
                            onChange={(v) => updateSettings({ secureDelete: v })}
                        />
                    </div>
                </div>
            </Card>

            <Card>
                <CardHeader>
                    <CardTitle>Thresholds</CardTitle>
                </CardHeader>
                <div className="space-y-4">
                    <div>
                        <label className="block text-sm font-medium mb-1">Large File Threshold (MB)</label>
                        <input
                            type="number"
                            className="input max-w-xs"
                            value={settings.largeFileThreshold / 1024 / 1024}
                            onChange={(e) => updateSettings({ largeFileThreshold: Number(e.target.value) * 1024 * 1024 })}
                        />
                    </div>
                    <div>
                        <label className="block text-sm font-medium mb-1">Old File Age (Days)</label>
                        <input
                            type="number"
                            className="input max-w-xs"
                            value={settings.oldFileThreshold}
                            onChange={(e) => updateSettings({ oldFileThreshold: Number(e.target.value) })}
                        />
                    </div>
                </div>
            </Card>
        </div>
    );
}
