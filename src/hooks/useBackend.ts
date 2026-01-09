import { useState } from 'react';
import { invoke } from '@tauri-apps/api/core';

export interface FileInfo {
    path: string;
    name: string;
    size: number;
    modified: string;
    created?: string;
    is_directory: boolean;
    extension?: string;
    file_type: string;
}

export interface ScanResult {
    total_files: number;
    total_size: number;
    temp_files: FileInfo[];
    cache_files: FileInfo[];
    log_files: FileInfo[];
    large_files: FileInfo[];
    old_files: FileInfo[];
    potential_savings: number;
    scan_duration_ms: number;
}

export const useBackend = () => {
    const [loading, setLoading] = useState(false);
    const [error, setError] = useState<string | null>(null);

    const performScan = async (paths: string[]): Promise<ScanResult | null> => {
        setLoading(true);
        setError(null);
        try {
            const result = await invoke<ScanResult>('scan_directory', {
                options: {
                    paths,
                    include_hidden: false,
                    min_size_bytes: null,
                    max_age_days: null,
                    file_types: null,
                }
            });
            return result;
        } catch (e: any) {
            setError(e.toString());
            return null;
        } finally {
            setLoading(false);
        }
    };

    const cleanup = async (files: string[], secure: boolean = false) => {
        setLoading(true);
        try {
            return await invoke('cleanup_files', {
                files,
                options: {
                    dry_run: false,
                    secure_delete: secure,
                    use_trash: true,
                    create_restore_point: true,
                }
            });
        } catch (e: any) {
            setError(e.toString());
            return null;
        } finally {
            setLoading(false);
        }
    };

    const getDiskInfo = async () => {
        return await invoke('get_disk_usage');
    };

    const getMemoryInfo = async () => {
        return await invoke('get_system_memory');
    };

    return {
        loading,
        error,
        performScan,
        cleanup,
        getDiskInfo,
        getMemoryInfo
    };
};
