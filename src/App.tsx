import React, { useState, useEffect } from "react";
import {
    Shield,
    Trash2,
    HardDrive,
    Cpu,
    Clock,
    Settings,
    Menu,
    X,
    Search,
    Zap,
    RotateCcw,
    BarChart3
} from "lucide-react";
import { useBackend, ScanResult } from "./hooks/useBackend";

const App: React.FC = () => {
    const [activeTab, setActiveTab] = useState("dashboard");
    const [scanResult, setScanResult] = useState<ScanResult | null>(null);
    const { performScan, cleanup, loading, error, getDiskInfo } = useBackend();
    const [scanning, setScanning] = useState(false);

    const startScan = async () => {
        setScanning(true);
        // Scan common user paths
        const result = await performScan(["C:/Users/ADMIN/AppData/Local/Temp", "C:/Windows/Temp"]);
        if (result) setScanResult(result);
        setScanning(false);
    };

    const handleCleanup = async () => {
        if (!scanResult) return;
        const allFiles = [
            ...scanResult.temp_files.map(f => f.path),
            ...scanResult.cache_files.map(f => f.path),
            ...scanResult.log_files.map(f => f.path)
        ];
        await cleanup(allFiles);
        setScanResult(null);
        alert("Cleanup complete!");
    };

    return (
        <div className="flex w-full h-full">
            {/* Sidebar */}
            <div className="w-64 bg-secondary flex flex-col border-r border-border">
                <div className="p-6 flex items-center gap-3">
                    <Shield className="text-accent-primary w-8 h-8" />
                    <span className="font-bold text-xl tracking-tight">SystemSweep</span>
                </div>

                <nav className="flex-1 px-4 py-4 flex flex-col gap-2">
                    <NavItem
                        icon={<Zap size={20} />}
                        label="Dashboard"
                        active={activeTab === "dashboard"}
                        onClick={() => setActiveTab("dashboard")}
                    />
                    <NavItem
                        icon={<Search size={20} />}
                        label="Scanner"
                        active={activeTab === "scanner"}
                        onClick={() => setActiveTab("scanner")}
                    />
                    <NavItem
                        icon={<Cpu size={20} />}
                        label="Optimizer"
                        active={activeTab === "optimizer"}
                        onClick={() => setActiveTab("optimizer")}
                    />
                    <NavItem
                        icon={<Clock size={20} />}
                        label="Scheduler"
                        active={activeTab === "scheduler"}
                        onClick={() => setActiveTab("scheduler")}
                    />
                    <NavItem
                        icon={<BarChart3 size={20} />}
                        label="Analytics"
                        active={activeTab === "analytics"}
                        onClick={() => setActiveTab("analytics")}
                    />
                    <NavItem
                        icon={<RotateCcw size={20} />}
                        label="Restore"
                        active={activeTab === "restore"}
                        onClick={() => setActiveTab("restore")}
                    />
                </nav>

                <div className="p-4 border-t border-border">
                    <NavItem
                        icon={<Settings size={20} />}
                        label="Settings"
                        active={activeTab === "settings"}
                        onClick={() => setActiveTab("settings")}
                    />
                </div>
            </div>

            {/* Main Content */}
            <main className="flex-1 flex flex-col overflow-hidden">
                {/* Header */}
                <header className="h-16 border-b border-border flex items-center justify-between px-8 glass sticky top-0 z-10">
                    <h2 className="text-xl font-semibold capitalize">{activeTab}</h2>
                    <div className="flex items-center gap-4">
                        <button
                            onClick={startScan}
                            disabled={scanning}
                            className={`px-6 py-2 rounded-full font-medium flex items-center gap-2 ${scanning
                                    ? "bg-border text-text-secondary"
                                    : "bg-accent-primary hover:bg-accent-secondary text-bg-primary shadow-lg shadow-accent-primary/20"
                                }`}
                        >
                            <Search size={18} />
                            {scanning ? "Scanning..." : "Quick Scan"}
                        </button>
                    </div>
                </header>

                {/* Content Area */}
                <div className="flex-1 overflow-y-auto p-8 custom-scrollbar">
                    {activeTab === "dashboard" && (
                        <div className="fade-in space-y-8">
                            {/* Stats Cards */}
                            <div className="grid grid-cols-1 md:grid-cols-3 gap-6">
                                <StatCard
                                    icon={<HardDrive className="text-accent-primary" />}
                                    label="Space Saved"
                                    value={scanResult ? formatBytes(scanResult.potential_savings) : "0 GB"}
                                    sub="Last 30 days"
                                />
                                <StatCard
                                    icon={<Trash2 className="text-danger" />}
                                    label="Files Cleaned"
                                    value={scanResult ? (scanResult.total_files).toString() : "0"}
                                    sub="Junk files found"
                                />
                                <StatCard
                                    icon={<Shield className="text-success" />}
                                    label="System Health"
                                    value="Excellent"
                                    sub="Protected"
                                />
                            </div>

                            {/* Action Area */}
                            {scanResult && (
                                <div className="p-8 border border-accent-primary/30 rounded-2xl bg-accent-primary/5 flex items-center justify-between">
                                    <div>
                                        <h3 className="text-2xl font-bold mb-2">Ready to optimize?</h3>
                                        <p className="text-text-secondary">We found {formatBytes(scanResult.potential_savings)} of junk files that can be safely removed.</p>
                                    </div>
                                    <button
                                        onClick={handleCleanup}
                                        className="px-8 py-3 bg-danger hover:bg-red-600 rounded-xl font-bold shadow-xl shadow-danger/20 transition-all"
                                    >
                                        Clean Now
                                    </button>
                                </div>
                            )}

                            {/* Recent Activity */}
                            <div className="bg-secondary rounded-2xl border border-border p-6">
                                <h3 className="text-lg font-semibold mb-6 flex items-center gap-2">
                                    <Clock size={18} className="text-text-secondary" />
                                    Recent Protection
                                </h3>
                                <div className="space-y-4">
                                    <ActivityItem label="System Scan" time="2 hours ago" status="Success" />
                                    <ActivityItem label="Registry Cleanup" time="1 day ago" status="Success" />
                                    <ActivityItem label="Duplicate Removal" time="3 days ago" status="Success" />
                                </div>
                            </div>
                        </div>
                    )}

                    {activeTab === "scanner" && (
                        <div className="fade-in flex items-center justify-center min-h-[400px]">
                            <div className="text-center">
                                <h2 className="text-2xl font-bold mb-4">Deep Scanner</h2>
                                <p className="text-text-secondary mb-8">Choose directories to analyze for duplicates and large files.</p>
                                <div className="grid grid-cols-2 gap-4">
                                    <button className="p-6 bg-secondary border border-border rounded-2xl hover:border-accent-primary transition-all group">
                                        <Search className="mx-auto mb-3 group-hover:text-accent-primary" />
                                        <span className="font-medium">Duplicate Finder</span>
                                    </button>
                                    <button className="p-6 bg-secondary border border-border rounded-2xl hover:border-accent-primary transition-all group">
                                        <HardDrive className="mx-auto mb-3 group-hover:text-accent-primary" />
                                        <span className="font-medium">Large Files</span>
                                    </button>
                                </div>
                            </div>
                        </div>
                    )}
                </div>
            </main>
        </div>
    );
};

// Sub-components
const NavItem: React.FC<{ icon: React.ReactNode, label: string, active?: boolean, onClick: () => void }> = ({ icon, label, active, onClick }) => (
    <button
        onClick={onClick}
        className={`w-full flex items-center gap-3 px-4 py-3 rounded-xl font-medium transition-all ${active
                ? "bg-accent-primary/10 text-accent-primary shadow-sm"
                : "text-text-secondary hover:bg-white/5 hover:text-text-primary"
            }`}
    >
        {icon}
        <span>{label}</span>
    </button>
);

const StatCard: React.FC<{ icon: React.ReactNode, label: string, value: string, sub: string }> = ({ icon, label, value, sub }) => (
    <div className="bg-secondary rounded-2xl border border-border p-6 hover:border-accent-primary/30 transition-all cursor-default group">
        <div className="flex items-center justify-between mb-4">
            <div className="p-3 bg-bg-primary rounded-xl group-hover:scale-110 transition-all">
                {icon}
            </div>
        </div>
        <div className="flex flex-col">
            <span className="text-text-secondary text-sm font-medium">{label}</span>
            <span className="text-3xl font-bold my-1 tracking-tight">{value}</span>
            <span className="text-text-secondary text-xs">{sub}</span>
        </div>
    </div>
);

const ActivityItem: React.FC<{ label: string, time: string, status: string }> = ({ label, time, status }) => (
    <div className="flex items-center justify-between p-3 rounded-xl hover:bg-white/5 transition-all">
        <div className="flex flex-col">
            <span className="font-medium">{label}</span>
            <span className="text-xs text-text-secondary">{time}</span>
        </div>
        <span className="text-xs font-bold px-2 py-1 bg-success/10 text-success rounded-lg">{status}</span>
    </div>
);

const formatBytes = (bytes: number) => {
    if (bytes === 0) return '0 Bytes';
    const k = 1024;
    const sizes = ['Bytes', 'KB', 'MB', 'GB', 'TB'];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i];
};

export default App;
