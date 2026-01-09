// src/components/views/Startup.tsx
import React, { useEffect, useState } from 'react';
import { Rocket, Trash2 } from 'lucide-react';
import { Card } from '../ui/Card';
import { Toggle } from '../ui/Toggle';
import { Badge } from '../ui/Badge';
import { Button } from '../ui/Button';
import * as api from '../../utils/api';
import { StartupItem } from '../../types';

export function Startup() {
    const [items, setItems] = useState<StartupItem[]>([]);
    const [loading, setLoading] = useState(true);

    const fetchItems = async () => {
        try {
            const data = await api.getStartupItems();
            setItems(data);
        } finally {
            setLoading(false);
        }
    };

    useEffect(() => {
        fetchItems();
    }, []);

    const handleToggle = async (id: string, enabled: boolean) => {
        setItems(items.map(i => i.id === id ? { ...i, enabled } : i));
        await api.toggleStartupItem(id, enabled);
    };

    return (
        <div className="space-y-6">
            <div className="flex items-center gap-4 mb-6">
                <div className="p-3 bg-blue-100 dark:bg-blue-900/30 rounded-xl text-blue-600">
                    <Rocket className="w-8 h-8" />
                </div>
                <div>
                    <h1 className="text-2xl font-bold">Startup Apps</h1>
                    <p className="text-dark-500">Manage applications that start with your computer</p>
                </div>
            </div>

            <Card padding="none" className="overflow-hidden">
                <table className="w-full text-left border-collapse">
                    <thead className="bg-dark-50 dark:bg-dark-800 text-dark-500 text-sm">
                        <tr>
                            <th className="p-4 font-medium">Application</th>
                            <th className="p-4 font-medium">Impact</th>
                            <th className="p-4 font-medium">Status</th>
                            <th className="p-4 font-medium text-right">Action</th>
                        </tr>
                    </thead>
                    <tbody className="divide-y divide-dark-100 dark:divide-dark-800">
                        {items.map(item => (
                            <tr key={item.id} className="hover:bg-dark-50 dark:hover:bg-dark-800/50 transition-colors">
                                <td className="p-4">
                                    <div className="font-medium text-dark-900 dark:text-dark-50">{item.name}</div>
                                    <div className="text-xs text-dark-400 font-mono mt-1">{item.path}</div>
                                </td>
                                <td className="p-4">
                                    <Badge variant={item.impact === 'high' ? 'danger' : item.impact === 'medium' ? 'warning' : 'success'}>
                                        {item.impact}
                                    </Badge>
                                </td>
                                <td className="p-4">
                                    <Toggle
                                        checked={item.enabled}
                                        onChange={(c) => handleToggle(item.id, c)}
                                        size="sm"
                                        label={item.enabled ? 'Enabled' : 'Disabled'}
                                    />
                                </td>
                                <td className="p-4 text-right">
                                    <Button variant="ghost" size="sm" className="text-danger-500 hover:text-danger-600">
                                        <Trash2 className="w-4 h-4" />
                                    </Button>
                                </td>
                            </tr>
                        ))}
                    </tbody>
                </table>
                {items.length === 0 && !loading && (
                    <div className="p-8 text-center text-dark-500">
                        No startup items found.
                    </div>
                )}
            </Card>
        </div>
    );
}
