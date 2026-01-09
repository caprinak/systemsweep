// src/components/views/Dashboard.tsx
import React from 'react';
import { motion } from 'framer-motion';
import {
    HardDrive,
    Trash2,
    FileSearch,
    Clock,
    TrendingUp,
    AlertTriangle,
    CheckCircle,
    Play,
    ArrowRight,
} from 'lucide-react';
import { Card, CardHeader, CardTitle, CardDescription } from '../ui/Card';
import { Button } from '../ui/Button';
import { CircularProgress, Progress } from '../ui/Progress';
import { useAppStore } from '../../stores/appStore';
import { useSystemHealth } from '../../hooks/useSystemHealth';
import { useScanner } from '../../hooks/useScanner';
import { formatBytes, formatRelativeTime } from '../../utils/format';
import {
    AreaChart,
    Area,
    XAxis,
    YAxis,
    CartesianGrid,
    Tooltip,
    ResponsiveContainer,
} from 'recharts';

// Mock data for the chart
const mockChartData = [
    { date: 'Mon', freed: 1.2 },
    { date: 'Tue', freed: 0.8 },
    { date: 'Wed', freed: 2.1 },
    { date: 'Thu', freed: 1.5 },
    { date: 'Fri', freed: 0.9 },
    { date: 'Sat', freed: 3.2 },
    { date: 'Sun', freed: 1.8 },
];

export function Dashboard() {
    const { systemHealth } = useSystemHealth();
    const { isScanning, startScan, scanResults, categoryStats } = useScanner();
    const { setCurrentView } = useAppStore();

    const quickActions = [
        {
            id: 'quick-scan',
            title: 'Quick Scan',
            description: 'Scan for junk files and cache',
            icon: <FileSearch className="w-6 h-6" />,
            color: 'from-blue-500 to-blue-600',
            action: () => startScan(['system_cache', 'temp_files', 'browser_cache']),
        },
        {
            id: 'empty-trash',
            title: 'Empty Trash',
            description: 'Clear recycle bin contents',
            icon: <Trash2 className="w-6 h-6" />,
            color: 'from-red-500 to-red-600',
            action: () => setCurrentView('scanner'),
        },
        {
            id: 'find-duplicates',
            title: 'Find Duplicates',
            description: 'Detect duplicate files',
            icon: <HardDrive className="w-6 h-6" />,
            color: 'from-purple-500 to-purple-600',
            action: () => setCurrentView('duplicates'),
        },
        {
            id: 'old-files',
            title: 'Old Files',
            description: 'Find unused old files',
            icon: <Clock className="w-6 h-6" />,
            color: 'from-orange-500 to-orange-600',
            action: () => setCurrentView('old-files'),
        },
    ];

    const diskUsagePercent = systemHealth
        ? (systemHealth.diskUsage.used / systemHealth.diskUsage.total) * 100
        : 0;

    const diskVariant =
        diskUsagePercent > 90 ? 'danger' : diskUsagePercent > 70 ? 'warning' : 'default';

    return (
        <div className="space-y-6">
            {/* Hero Stats */}
            <div className="grid grid-cols-1 md:grid-cols-3 gap-6">
                {/* Disk Usage */}
                <Card className="col-span-1">
                    <div className="flex items-center gap-6">
                        <CircularProgress
                            value={diskUsagePercent}
                            size={100}
                            strokeWidth={8}
                            variant={diskVariant as any}
                        />
                        <div>
                            <h3 className="text-2xl font-bold text-dark-900 dark:text-dark-50">
                                {systemHealth ? formatBytes(systemHealth.diskUsage.free) : '--'}
                            </h3>
                            <p className="text-dark-500 dark:text-dark-400">Free Space</p>
                            <p className="text-sm text-dark-400 mt-1">
                                {systemHealth
                                    ? `${formatBytes(systemHealth.diskUsage.used)} of ${formatBytes(
                                        systemHealth.diskUsage.total
                                    )}`
                                    : 'Loading...'}
                            </p>
                        </div>
                    </div>
                </Card>

                {/* Cleanable Space */}
                <Card className="col-span-1">
                    <div className="flex items-center gap-4">
                        <div className="w-14 h-14 bg-success-100 dark:bg-success-500/20 rounded-xl flex items-center justify-center">
                            <TrendingUp className="w-7 h-7 text-success-600 dark:text-success-400" />
                        </div>
                        <div>
                            <h3 className="text-2xl font-bold text-dark-900 dark:text-dark-50">
                                {systemHealth ? formatBytes(systemHealth.cleanableSpace) : '--'}
                            </h3>
                            <p className="text-dark-500 dark:text-dark-400">Can be cleaned</p>
                            <p className="text-sm text-success-600 dark:text-success-400 mt-1">
                                {scanResults.length} items found
                            </p>
                        </div>
                    </div>
                </Card>

                {/* System Health */}
                <Card className="col-span-1">
                    <div className="flex items-center gap-4">
                        <div className="w-14 h-14 bg-primary-100 dark:bg-primary-500/20 rounded-xl flex items-center justify-center">
                            {systemHealth && systemHealth.issuesFound > 0 ? (
                                <AlertTriangle className="w-7 h-7 text-warning-600 dark:text-warning-400" />
                            ) : (
                                <CheckCircle className="w-7 h-7 text-success-600 dark:text-success-400" />
                            )}
                        </div>
                        <div>
                            <h3 className="text-2xl font-bold text-dark-900 dark:text-dark-50">
                                {systemHealth?.issuesFound || 0} Issues
                            </h3>
                            <p className="text-dark-500 dark:text-dark-400">System Health</p>
                            <p className="text-sm text-dark-400 mt-1">
                                Last scan:{' '}
                                {systemHealth?.lastScan
                                    ? formatRelativeTime(systemHealth.lastScan)
                                    : 'Never'}
                            </p>
                        </div>
                    </div>
                </Card>
            </div>

            {/* Quick Actions */}
            <Card padding="lg">
                <CardHeader>
                    <CardTitle>Quick Actions</CardTitle>
                    <CardDescription>Common cleanup tasks at your fingertips</CardDescription>
                </CardHeader>
                <div className="grid grid-cols-2 md:grid-cols-4 gap-4">
                    {quickActions.map((action, index) => (
                        <motion.button
                            key={action.id}
                            initial={{ opacity: 0, y: 20 }}
                            animate={{ opacity: 1, y: 0 }}
                            transition={{ delay: index * 0.1 }}
                            onClick={action.action}
                            disabled={isScanning}
                            className="group relative overflow-hidden rounded-xl p-4 text-left transition-all hover:scale-105 disabled:opacity-50 disabled:hover:scale-100"
                        >
                            <div
                                className={`absolute inset-0 bg-gradient-to-br ${action.color} opacity-10 group-hover:opacity-20 transition-opacity`}
                            />
                            <div
                                className={`w-12 h-12 rounded-lg bg-gradient-to-br ${action.color} flex items-center justify-center text-white mb-3`}
                            >
                                {action.icon}
                            </div>
                            <h4 className="font-semibold text-dark-900 dark:text-dark-50">
                                {action.title}
                            </h4>
                            <p className="text-sm text-dark-500 dark:text-dark-400 mt-1">
                                {action.description}
                            </p>
                        </motion.button>
                    ))}
                </div>
            </Card>

            <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
                {/* Storage Analytics */}
                <Card padding="lg">
                    <CardHeader
                        action={
                            <Button variant="ghost" size="sm" rightIcon={<ArrowRight className="w-4 h-4" />}>
                                View Details
                            </Button>
                        }
                    >
                        <CardTitle>Space Freed This Week</CardTitle>
                    </CardHeader>
                    <div className="h-64">
                        <ResponsiveContainer width="100%" height="100%">
                            <AreaChart data={mockChartData}>
                                <defs>
                                    <linearGradient id="colorFreed" x1="0" y1="0" x2="0" y2="1">
                                        <stop offset="5%" stopColor="#3b82f6" stopOpacity={0.3} />
                                        <stop offset="95%" stopColor="#3b82f6" stopOpacity={0} />
                                    </linearGradient>
                                </defs>
                                <CartesianGrid strokeDasharray="3 3" className="stroke-dark-200 dark:stroke-dark-700" />
                                <XAxis
                                    dataKey="date"
                                    className="text-dark-500"
                                    tick={{ fill: 'currentColor', fontSize: 12 }}
                                />
                                <YAxis
                                    className="text-dark-500"
                                    tick={{ fill: 'currentColor', fontSize: 12 }}
                                    tickFormatter={(value) => `${value}GB`}
                                />
                                <Tooltip
                                    contentStyle={{
                                        backgroundColor: 'var(--background)',
                                        border: '1px solid var(--border)',
                                        borderRadius: '8px',
                                    }}
                                    formatter={(value: number) => [`${value} GB`, 'Space Freed']}
                                />
                                <Area
                                    type="monotone"
                                    dataKey="freed"
                                    stroke="#3b82f6"
                                    strokeWidth={2}
                                    fillOpacity={1}
                                    fill="url(#colorFreed)"
                                />
                            </AreaChart>
                        </ResponsiveContainer>
                    </div>
                </Card>

                {/* Categories Breakdown */}
                <Card padding="lg">
                    <CardHeader>
                        <CardTitle>Cleanup Categories</CardTitle>
                        <CardDescription>Breakdown of cleanable items by category</CardDescription>
                    </CardHeader>
                    <div className="space-y-4">
                        {categoryStats.length === 0 ? (
                            <div className="text-center py-8">
                                <p className="text-dark-500 dark:text-dark-400">
                                    Run a scan to see category breakdown
                                </p>
                                <Button
                                    onClick={() => startScan(['system_cache', 'temp_files', 'browser_cache', 'log_files'])}
                                    isLoading={isScanning}
                                    leftIcon={<Play className="w-4 h-4" />}
                                    className="mt-4"
                                >
                                    Start Scan
                                </Button>
                            </div>
                        ) : (
                            categoryStats.slice(0, 5).map((stat, index) => (
                                <motion.div
                                    key={stat.category}
                                    initial={{ opacity: 0, x: -20 }}
                                    animate={{ opacity: 1, x: 0 }}
                                    transition={{ delay: index * 0.1 }}
                                    className="flex items-center gap-4"
                                >
                                    <div className="flex-1">
                                        <div className="flex items-center justify-between mb-1">
                                            <span className="text-sm font-medium text-dark-700 dark:text-dark-300">
                                                {stat.label}
                                            </span>
                                            <span className="text-sm text-dark-500">
                                                {stat.count} files • {formatBytes(stat.size)}
                                            </span>
                                        </div>
                                        <Progress
                                            value={stat.size}
                                            max={categoryStats.reduce((acc, s) => acc + s.size, 0)}
                                            size="sm"
                                        />
                                    </div>
                                </motion.div>
                            ))
                        )}
                    </div>
                </Card>
            </div>
        </div>
    );
}
