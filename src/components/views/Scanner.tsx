// src/components/views/Scanner.tsx
import React, { useState } from 'react';
import { motion } from 'framer-motion';
import { Play, Trash2, CheckCircle2, Filter } from 'lucide-react';
import { useScanner } from '../../hooks/useScanner';
import { useCleanup } from '../../hooks/useCleanup';
import { Card, CardHeader, CardTitle, CardDescription } from '../ui/Card';
import { Button } from '../ui/Button';
import { CircularProgress, Progress } from '../ui/Progress';
import { Badge } from '../ui/Badge';
import { formatBytes } from '../../utils/format';
import { CleanupCategory } from '../../types';
import { useAppStore } from '../../stores/appStore';

const CATEGORIES: { id: CleanupCategory; label: string; description: string }[] = [
    { id: 'system_cache', label: 'System Cache', description: 'Temporary system files' },
    { id: 'browser_cache', label: 'Browser Cache', description: 'Web browsing history and cache' },
    { id: 'temp_files', label: 'Temporary Files', description: 'App installation and runtime temp files' },
    { id: 'log_files', label: 'Log Files', description: 'System and application logs' },
    { id: 'trash', label: 'Trash', description: 'Recycle bin contents' },
];

export function Scanner() {
    const { isScanning, scanProgress, scanResults, startScan, cancelScan } = useScanner();
    const { performCleanup, isCleaning, cleanupProgress, selectedSize } = useCleanup();
    const { toggleResultSelection, deselectAllResults } = useAppStore();

    const [selectedCategories, setSelectedCategories] = useState<CleanupCategory[]>(
        CATEGORIES.map(c => c.id)
    );

    const toggleCategory = (id: CleanupCategory) => {
        if (selectedCategories.includes(id)) {
            setSelectedCategories(prev => prev.filter(c => c !== id));
        } else {
            setSelectedCategories(prev => [...prev, id]);
        }
    };

    const handleStartScan = () => {
        startScan(selectedCategories);
    };

    // Scanning State
    if (isScanning && scanProgress) {
        return (
            <div className="h-full flex flex-col items-center justify-center space-y-8">
                <div className="relative">
                    <CircularProgress
                        value={scanProgress.progress}
                        size={240}
                        strokeWidth={12}
                        showLabel={true}
                    />
                    <motion.div
                        animate={{ rotate: 360 }}
                        transition={{ duration: 4, repeat: Infinity, ease: "linear" }}
                        className="absolute inset-0 rounded-full border-b-4 border-primary-200 dark:border-primary-900 opacity-20"
                    />
                </div>

                <div className="text-center space-y-2 max-w-md">
                    <h2 className="text-2xl font-bold text-dark-900 dark:text-dark-50">
                        Scanning System
                    </h2>
                    <p className="text-dark-500 text-sm font-mono break-all">
                        {scanProgress.currentPath || 'Initializing...'}
                    </p>
                    <div className="flex justify-center gap-4 text-sm text-dark-500 mt-4">
                        <span>Files: {scanProgress.filesScanned}</span>
                        <span>Issues: {scanProgress.issuesFound}</span>
                    </div>
                </div>

                <Button variant="outline" onClick={cancelScan} className="mt-8">
                    Cancel Scan
                </Button>
            </div>
        );
    }

    // Results State
    if (scanResults.length > 0 && !isScanning) {
        return (
            <div className="space-y-6 h-full flex flex-col">
                {/* Header Summary */}
                <Card className="flex-shrink-0">
                    <div className="flex items-center justify-between p-4">
                        <div>
                            <h2 className="text-lg font-semibold text-dark-900 dark:text-dark-50">
                                Scan Complete
                            </h2>
                            <p className="text-sm text-dark-500">
                                Selected: <span className="font-bold text-primary-600">{formatBytes(selectedSize)}</span>
                            </p>
                        </div>
                        <div className="flex gap-3">
                            <Button variant="ghost" onClick={deselectAllResults}>Deselect All</Button>
                            <Button
                                onClick={performCleanup}
                                isLoading={isCleaning}
                                disabled={selectedSize === 0}
                                leftIcon={<Trash2 className="w-4 h-4" />}
                            >
                                Clean Now
                            </Button>
                        </div>
                    </div>
                    {isCleaning && cleanupProgress && (
                        <div className="px-4 pb-4">
                            <div className="flex justify-between text-xs mb-1">
                                <span>Cleaning...</span>
                                <span>{cleanupProgress.current} / {cleanupProgress.total}</span>
                            </div>
                            <Progress value={cleanupProgress.current} max={cleanupProgress.total} />
                        </div>
                    )}
                </Card>

                {/* Results List */}
                <div className="flex-1 overflow-y-auto space-y-2 pr-2">
                    {scanResults.map((result) => (
                        <motion.div
                            key={result.id}
                            layout
                            initial={{ opacity: 0 }}
                            animate={{ opacity: 1 }}
                            className={`group flex items-center gap-4 p-3 rounded-lg border transition-colors ${result.isSelected
                                    ? 'bg-primary-50 dark:bg-primary-900/10 border-primary-200 dark:border-primary-800'
                                    : 'bg-white dark:bg-dark-900 border-dark-200 dark:border-dark-800 hover:border-dark-300'
                                }`}
                            onClick={() => toggleResultSelection(result.id)}
                        >
                            <div className="flex items-center h-5">
                                <input
                                    type="checkbox"
                                    checked={result.isSelected}
                                    readOnly
                                    className="w-4 h-4 rounded text-primary-600 focus:ring-primary-500 border-gray-300"
                                />
                            </div>
                            <div className="flex-1 min-w-0">
                                <div className="flex items-center gap-2">
                                    <span className="font-medium text-sm text-dark-900 dark:text-dark-50 truncate">
                                        {result.name}
                                    </span>
                                    <Badge size="sm" variant={result.risk === 'high' ? 'danger' : 'info'}>
                                        {result.category.replace('_', ' ')}
                                    </Badge>
                                </div>
                                <p className="text-xs text-dark-500 truncate mt-0.5">{result.path}</p>
                            </div>
                            <div className="text-right whitespace-nowrap">
                                <span className="text-sm font-bold text-dark-700 dark:text-dark-300">
                                    {formatBytes(result.size)}
                                </span>
                            </div>
                        </motion.div>
                    ))}
                </div>
            </div>
        );
    }

    // Initial State
    return (
        <div className="max-w-4xl mx-auto space-y-8 py-8">
            <div className="text-center space-y-4">
                <h1 className="text-3xl font-bold text-dark-900 dark:text-dark-50">
                    Smart System Scan
                </h1>
                <p className="text-dark-500 dark:text-dark-400 max-w-xl mx-auto">
                    Select the categories you want to analyze. CleanDesk will scan your system for safe-to-remove files to free up space.
                </p>
            </div>

            <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
                {CATEGORIES.map((category) => (
                    <Card
                        key={category.id}
                        className={`cursor-pointer transition-all ${selectedCategories.includes(category.id)
                                ? 'ring-2 ring-primary-500 shadow-md'
                                : 'opacity-70 hover:opacity-100'
                            }`}
                        padding="md"
                    >
                        <div onClick={() => toggleCategory(category.id)}>
                            <div className="flex items-center justify-between mb-2">
                                <div className={`p-2 rounded-lg ${selectedCategories.includes(category.id)
                                        ? 'bg-primary-100 text-primary-600'
                                        : 'bg-dark-100 text-dark-500'
                                    }`}>
                                    <Filter className="w-5 h-5" />
                                </div>
                                {selectedCategories.includes(category.id) && (
                                    <CheckCircle2 className="w-5 h-5 text-primary-600" />
                                )}
                            </div>
                            <h3 className="font-semibold text-dark-900 dark:text-dark-50">{category.label}</h3>
                            <p className="text-sm text-dark-500 mt-1">{category.description}</p>
                        </div>
                    </Card>
                ))}
            </div>

            <div className="flex justify-center pt-8">
                <Button
                    size="lg"
                    onClick={handleStartScan}
                    disabled={selectedCategories.length === 0}
                    leftIcon={<Play className="w-5 h-5" />}
                >
                    Start Scan
                </Button>
            </div>
        </div>
    );
}
