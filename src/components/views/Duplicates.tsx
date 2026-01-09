// src/components/views/Duplicates.tsx
import React, { useState } from 'react';
import { Copy, Trash2, Search } from 'lucide-react';
import { useAppStore } from '../../stores/appStore';
import * as api from '../../utils/api';
import { formatBytes } from '../../utils/format';
import { Card } from '../ui/Card';
import { Button } from '../ui/Button';
import { Badge } from '../ui/Badge';

export function Duplicates() {
    const { duplicates, setDuplicates, addNotification } = useAppStore();
    const [loading, setLoading] = useState(false);

    const scanDuplicates = async () => {
        setLoading(true);
        try {
            const results = await api.findDuplicates();
            setDuplicates(results);
            addNotification({
                type: 'success',
                title: 'Scan Complete',
                message: `Found ${results.length} sets of duplicates`,
            });
        } catch (err) {
            addNotification({ type: 'error', title: 'Error', message: 'Failed to find duplicates' });
        } finally {
            setLoading(false);
        }
    };

    const smartSelect = (strategy: 'newest' | 'oldest') => {
        const newDuplicates = duplicates.map(group => {
            const sorted = [...group.files].sort((a, b) =>
                new Date(a.modified).getTime() - new Date(b.modified).getTime()
            );

            const toKeep = strategy === 'oldest' ? sorted[0] : sorted[sorted.length - 1];

            return {
                ...group,
                files: group.files.map(f => ({
                    ...f,
                    isSelected: f.id !== toKeep.id
                }))
            };
        });
        setDuplicates(newDuplicates);
    };

    const deleteSelected = async () => {
        const selectedIds = duplicates.flatMap(g => g.files.filter(f => f.isSelected).map(f => f.id));
        if (selectedIds.length === 0) return;

        try {
            setLoading(true);
            const res = await api.deleteDuplicates(selectedIds, true);
            addNotification({
                type: 'success',
                title: 'Cleanup Complete',
                message: `Deleted ${res.deleted} duplicates, freeing ${formatBytes(res.freed)}`
            });
            scanDuplicates();
        } catch (err) {
            addNotification({ type: 'error', title: 'Error', message: 'Failed to delete duplicates' });
        } finally {
            setLoading(false);
        }
    };

    const totalSelectedSize = duplicates.reduce((acc, group) => {
        return acc + group.files.filter(f => f.isSelected).reduce((sum, f) => sum + f.size, 0);
    }, 0);

    if (duplicates.length === 0 && !loading) {
        return (
            <div className="h-full flex flex-col items-center justify-center p-8 text-center">
                <div className="w-20 h-20 bg-purple-100 dark:bg-purple-900/20 rounded-2xl flex items-center justify-center mb-6">
                    <Copy className="w-10 h-10 text-purple-600 dark:text-purple-400" />
                </div>
                <h2 className="text-2xl font-bold mb-2">Find Duplicate Files</h2>
                <p className="text-dark-500 max-w-md mb-8">
                    Scan your system for identical files that are wasting space. We compare file contents, not just names.
                </p>
                <Button onClick={scanDuplicates} size="lg" leftIcon={<Search className="w-5 h-5" />}>
                    Scan for Duplicates
                </Button>
            </div>
        );
    }

    return (
        <div className="space-y-6 h-full flex flex-col">
            <Card className="flex-shrink-0">
                <div className="flex items-center justify-between p-4">
                    <div className="flex gap-4">
                        <Button variant="secondary" size="sm" onClick={() => smartSelect('oldest')}>
                            Keep Oldest
                        </Button>
                        <Button variant="secondary" size="sm" onClick={() => smartSelect('newest')}>
                            Keep Newest
                        </Button>
                    </div>
                    <div className="flex items-center gap-4">
                        <span className="text-sm font-medium">
                            Selected: <span className="text-danger-600">{formatBytes(totalSelectedSize)}</span>
                        </span>
                        <Button
                            variant="danger"
                            onClick={deleteSelected}
                            isLoading={loading}
                            disabled={totalSelectedSize === 0}
                            leftIcon={<Trash2 className="w-4 h-4" />}
                        >
                            Remove Selected
                        </Button>
                    </div>
                </div>
            </Card>

            <div className="flex-1 overflow-y-auto space-y-4">
                {duplicates.map((group) => (
                    <Card key={group.hash} padding="none" className="overflow-hidden">
                        <div className="bg-dark-50 dark:bg-dark-800 p-3 flex justify-between items-center border-b border-dark-200 dark:border-dark-700">
                            <div className="flex items-center gap-2">
                                <Badge variant="warning">{group.files.length} Copies</Badge>
                                <span className="text-sm text-dark-500">{formatBytes(group.files[0].size)} each</span>
                            </div>
                            <span className="text-xs font-mono text-dark-400">{group.hash.substring(0, 12)}...</span>
                        </div>
                        <div className="divide-y divide-dark-100 dark:divide-dark-800">
                            {group.files.map((file) => (
                                <div key={file.id} className="p-3 flex items-center gap-3 hover:bg-dark-50 dark:hover:bg-dark-800/50">
                                    <input
                                        type="checkbox"
                                        checked={file.isSelected}
                                        onChange={() => {
                                            const newDups = duplicates.map(g =>
                                                g.hash === group.hash
                                                    ? { ...g, files: g.files.map(f => f.id === file.id ? { ...f, isSelected: !f.isSelected } : f) }
                                                    : g
                                            );
                                            setDuplicates(newDups);
                                        }}
                                        className="rounded border-gray-300"
                                    />
                                    <div className="flex-1 min-w-0">
                                        <p className="text-sm font-medium truncate">{file.name}</p>
                                        <p className="text-xs text-dark-500 truncate">{file.path}</p>
                                    </div>
                                    <span className="text-xs text-dark-400">
                                        {new Date(file.modified).toLocaleDateString()}
                                    </span>
                                </div>
                            ))}
                        </div>
                    </Card>
                ))}
            </div>
        </div>
    );
}
