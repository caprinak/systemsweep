I will check the status of the running background tasks to see if the directory sizes have finished calculating.

Used tool: manage_task

I will check the size of common temporary folders and system cache directories that can often be safely cleaned up. This includes:
1. `C:\Windows\SoftwareDistribution\Download` (Windows Update cache)
2. `C:\Windows\Temp` (System temporary files)
3. `C:\Users\ADMIN\AppData\Local\Temp` (User temporary files)
4. `C:\Users\ADMIN\AppData\Local\Google\Chrome\User Data\Default\Cache` (Chrome cache)
5. `C:\Users\ADMIN\AppData\Local\BraveSoftware\Brave-Browser\User Data\Default\Cache` (Brave browser cache)

Ran command: `$paths = @(
    "C:\Windows\SoftwareDistribution\Download",
    "C:\Windows\Temp",
    "C:\Users\ADMIN\AppData\Local\Temp",
    "C:\Users\ADMIN\AppData\Local\Google\Chrome\User Data\Default\Cache",
    "C:\Users\ADMIN\AppData\Local\BraveSoftware\Brave-Browser\User Data\Default\Cache"
)

$paths | ForEach-Object {
    if (Test-Path $_) {
        $size = (Get-ChildItem $_ -Recurse -File -ErrorAction SilentlyContinue | Measure-Object -Property Length -Sum).Sum
        [PSCustomObject]@{
            Path = $_
            SizeMB = [Math]::Round($size / 1MB, 2)
        }
    } else {
        [PSCustomObject]@{
            Path = $_
            SizeMB = "Not Found"
        }
    }
} | Format-Table -AutoSize`

I will wait for 15 seconds for the current folder sizing operations to proceed.

Used tool: schedule

I am waiting for the background tasks analyzing your C drive folder sizes to finish. I will have the results in a moment.

I will check the status of the remaining background tasks (top-level folders and user directory folders).

Used tool: manage_task

I will wait for another 30 seconds for the two major sizing tasks (C:\ and C:\Users\ADMIN) to finish.

Used tool: schedule

I am waiting for the larger disk usage scanning tasks to finish. I will update you as soon as they complete.

I will check the status of our background tasks to see if they have finished.

Used tool: manage_task

I will set a 60-second timer to give the background directory size checks more time to finish. In the meantime, the system will wake me up automatically if they finish sooner.

Used tool: schedule

I am waiting for the background scan of the C: drive and the user directory to complete. I will show you the exact sizes of the folders as soon as the results are ready.

I will check the status of the tasks again.

Used tool: manage_task

I will list the folders inside `C:\Users\ADMIN` to see what subdirectories exist, which will help us target specific ones that are likely to be large.

Ran command: `Get-ChildItem -Path "C:\Users\ADMIN" -Directory -Force | Select-Object Name, LastWriteTime`

I will use `robocopy` in list-only mode (`/L`) to very quickly measure the sizes of the developer/user folders (like `.ollama`, `.gradle`, `.cargo`, `.m2`, `_bazel_ADMIN`, `Downloads`, etc.). This is much faster than standard PowerShell recursion.

Ran command: `$folders = @(
    ".ollama", ".gradle", ".m2", ".cargo", ".rustup", "_bazel_ADMIN", ".docker", ".vscode",
    "Downloads", "Documents", "Desktop", "Videos", "AppData\Local\Docker", "AppData\Local\Android", "AppData\Local\pip"
)

$folders | ForEach-Object {
    $fullPath = Join-Path "C:\Users\ADMIN" $_
    if (Test-Path $fullPath) {
        $robocopy = robocopy $fullPath NULL /S /L /BYTES /NJH /NJS /NFL /NDL /R:0 /W:0
        $sizeLine = $robocopy | Where-Object { $_ -match "Bytes\s+:\s+(\d+)" }
        if ($sizeLine -and $Matches[1]) {
            $bytes = [double]$Matches[1]
            $sizeGB = [Math]::Round($bytes / 1GB, 2)
            [PSCustomObject]@{
                Folder = $_
                SizeGB = $sizeGB
            }
        }
    }
} | Sort-Object SizeGB -Descending | Format-Table -AutoSize`

I will wait for 15 seconds to let the robocopy sizing task complete.

Used tool: schedule

I am waiting for the fast folder scans to complete using robocopy. I will present the sizes of the specific developer folders and user directories shortly.

I will check the status of our background tasks now.

Used tool: manage_task

I will check the current log of the robocopy sizing task to see how far it has progressed.

I will check if the robocopy process is currently active on your system.

Ran command: `Get-Process -Name "robocopy" -ErrorAction SilentlyContinue | Select-Object Id, ProcessName, CPU, StartTime`

I will run a single robocopy command on a known folder to see the exact text output format so I can adjust the size extraction logic if needed.

Ran command: `robocopy "C:\Users\ADMIN\.ollama" NULL /S /L /BYTES /NJH /NJS /NFL /NDL /R:0 /W:0`

I will run robocopy without suppressing the summary table to see the total size.

Ran command: `robocopy "C:\Users\ADMIN\.ollama" NULL /S /L /BYTES /NFL /NDL /R:0 /W:0`

I will check if the long-running size calculations have finished yet.

Used tool: manage_task

I will run a comprehensive sizing script using `robocopy` to find the exact size of all top-level folders on the `C:\` drive and inside the `C:\Users\ADMIN` folder. This is optimized for speed and should finish quickly.

Ran command: `Write-Host "--- TOP LEVEL C:\ ---"
Get-ChildItem -Path "C:\" -Directory -Force | ForEach-Object {
    $fullPath = $_.FullName
    $robocopy = robocopy $fullPath NULL /S /L /BYTES /NJH /NFL /NDL /R:0 /W:0
    $sizeLine = $robocopy | Where-Object { $_ -match "Bytes\s+:\s+(\d+)" }
    if ($sizeLine -and $Matches[1]) {
        $bytes = [double]$Matches[1]
        $sizeGB = [Math]::Round($bytes / 1GB, 2)
        [PSCustomObject]@{
            Folder = $_.Name
            SizeGB = $sizeGB
        }
    }
} | Sort-Object SizeGB -Descending | Format-Table -AutoSize

Write-Host "--- USER PROFILE C:\Users\ADMIN ---"
Get-ChildItem -Path "C:\Users\ADMIN" -Directory -Force | ForEach-Object {
    $fullPath = $_.FullName
    $robocopy = robocopy $fullPath NULL /S /L /BYTES /NJH /NFL /NDL /R:0 /W:0
    $sizeLine = $robocopy | Where-Object { $_ -match "Bytes\s+:\s+(\d+)" }
    if ($sizeLine -and $Matches[1]) {
        $bytes = [double]$Matches[1]
        $sizeGB = [Math]::Round($bytes / 1GB, 2)
        [PSCustomObject]@{
            Folder = $_.Name
            SizeGB = $sizeGB
        }
    }
} | Sort-Object SizeGB -Descending | Format-Table -AutoSize`

I will cancel the previous slow-running sizing tasks to save system resources, as the new `robocopy` task will compute the results much faster.

Used tool: manage_task

I will cancel the other slow-running user directory sizing task too.

Used tool: manage_task

I will schedule a 20-second timer to wait for the fast robocopy sizing task to complete.

Used tool: schedule

I am waiting for the optimized scan of the C: drive and user directories to finish. I will have the output for you in a moment.

I will check the status of the sizing task now.

Used tool: manage_task

I will check the size of Zalo folders on your computer, as Zalo is known to accumulate a large amount of cache and media files (images/videos) over time, which can fill up the C drive.

Ran command: `$zaloPaths = @(
    "C:\Users\ADMIN\AppData\Local\ZaloPC",
    "C:\Users\ADMIN\AppData\Roaming\Zalo"
)

$zaloPaths | ForEach-Object {
    if (Test-Path $_) {
        $robocopy = robocopy $_ NULL /S /L /BYTES /NJH /NFL /NDL /R:0 /W:0
        $sizeLine = $robocopy | Where-Object { $_ -match "Bytes\s+:\s+(\d+)" }
        if ($sizeLine -and $Matches[1]) {
            $bytes = [double]$Matches[1]
            $sizeGB = [Math]::Round($bytes / 1GB, 2)
            [PSCustomObject]@{
                Path = $_
                SizeGB = $sizeGB
            }
        }
    }
} | Format-Table -AutoSize`

I will check the exact folder names for Zalo in AppData to see where it stores its files.

Ran command: `Get-ChildItem -Path C:\Users\ADMIN\AppData\Local, C:\Users\ADMIN\AppData\Roaming -Filter *Zalo* -Directory -ErrorAction SilentlyContinue | Select-Object FullName`

I will now measure the exact size of the `C:\Users\ADMIN\AppData\Local\Zalo` and `C:\Users\ADMIN\AppData\Roaming\ZaloData` folders.

Ran command: `$paths = @(
    "C:\Users\ADMIN\AppData\Local\Zalo",
    "C:\Users\ADMIN\AppData\Local\zalo-updater",
    "C:\Users\ADMIN\AppData\Roaming\ZaloData"
)

$paths | ForEach-Object {
    if (Test-Path $_) {
        $robocopy = robocopy $_ NULL /S /L /BYTES /NJH /NFL /NDL /R:0 /W:0
        $sizeLine = $robocopy | Where-Object { $_ -match "Bytes\s+:\s+(\d+)" }
        if ($sizeLine -and $Matches[1]) {
            $bytes = [double]$Matches[1]
            $sizeGB = [Math]::Round($bytes / 1GB, 2)
            [PSCustomObject]@{
                Path = $_
                SizeGB = $sizeGB
            }
        }
    }
} | Format-Table -AutoSize`

I will check the status of the main folder scan task.

Used tool: manage_task

I will wait for another 30 seconds for the main directory sizing script (`task-78`) to proceed further.

Used tool: schedule

I am waiting for the main folder scan to finish. I will update you as soon as I receive the results.

I will check the status of the sizing task again.

Used tool: manage_task

I will list the folders in `C:\ProgramData` to check for any hidden application caches or installer files that might be consuming disk space.

Ran command: `Get-ChildItem -Path "C:\ProgramData" -Directory -Force | Select-Object Name, LastWriteTime`

I will check if the main directory sizing task (`task-78`) has finished.

Used tool: manage_task

I will inspect the log file of the main sizing task to see what it has outputted so far.

Viewed task-78.log:1-2

I will check the sizes of directories inside `C:\ProgramData` and print the results progressively to see which ones are taking up more than 10 MB.

Ran command: `Get-ChildItem -Path "C:\ProgramData" -Directory -Force | ForEach-Object {
    $fullPath = $_.FullName
    $robocopy = robocopy $fullPath NULL /S /L /BYTES /NJH /NFL /NDL /R:0 /W:0
    $sizeLine = $robocopy | Where-Object { $_ -match "Bytes\s+:\s+(\d+)" }
    if ($sizeLine -and $Matches[1]) {
        $bytes = [double]$Matches[1]
        $sizeMB = [Math]::Round($bytes / 1MB, 2)
        if ($sizeMB -gt 10) {
            Write-Host "$($_.Name): $sizeMB MB"
        }
    }
}`

I will measure and list all folders in `C:\Users\ADMIN` that are larger than 100 MB, printing them progressively so we don't have to wait for the entire scan to buffer.

Ran command: `Get-ChildItem -Path "C:\Users\ADMIN" -Directory -Force | ForEach-Object {
    $fullPath = $_.FullName
    $robocopy = robocopy $fullPath NULL /S /L /BYTES /NJH /NFL /NDL /R:0 /W:0
    $sizeLine = $robocopy | Where-Object { $_ -match "Bytes\s+:\s+(\d+)" }
    if ($sizeLine -and $Matches[1]) {
        $bytes = [double]$Matches[1]
        $sizeMB = [Math]::Round($bytes / 1MB, 2)
        if ($sizeMB -gt 100) {
            Write-Host "$($_.Name): $sizeMB MB"
        }
    }
}`

I will wait for 15 seconds for this progressive user directory scan to print its initial results.

Used tool: schedule

I am waiting for the progressive scan of the user directory. I will show the folders that are larger than 100 MB as they are discovered.

I will check the status and log output of the progressive user directory sizing task (`task-114`).

Used tool: manage_task

I will wait for another 15 seconds for the progressive scan to check the folders that start with letters after 'v' (including AppData, Downloads, Documents, Videos, etc.).

Used tool: schedule

I am waiting for the remaining folders in the user directory to be measured. I will show you the sizes of folders like AppData, Downloads, and Documents next.

I will check the status and latest log output of the user directory sizing task (`task-114`).

Used tool: manage_task

I will cancel the whole C:\ drive scan task (`task-78`) to free up system resources, as we now have specific folder sizes and the user-level scan is the most important part.

Used tool: manage_task

I will schedule a 30-second timer to give the user directory scan more time to traverse `AppData` and show its results.

Used tool: schedule

I am waiting for the user directory sizing task (`task-114`) to complete scanning the AppData, Downloads, and other remaining folders. I will present the new sizes as soon as they are scanned.

I will run a progressive scan inside `C:\Users\ADMIN\AppData` (Local, LocalLow, Roaming) to measure subfolders that are larger than 500 MB. This will locate the exact apps or cache files causing the 88.9 GB usage.

Ran command: `Get-ChildItem -Path "C:\Users\ADMIN\AppData\Local", "C:\Users\ADMIN\AppData\Roaming" -Directory -Force | ForEach-Object {
    $fullPath = $_.FullName
    $robocopy = robocopy $fullPath NULL /S /L /BYTES /NJH /NFL /NDL /R:0 /W:0
    $sizeLine = $robocopy | Where-Object { $_ -match "Bytes\s+:\s+(\d+)" }
    if ($sizeLine -and $Matches[1]) {
        $bytes = [double]$Matches[1]
        $sizeMB = [Math]::Round($bytes / 1MB, 2)
        if ($sizeMB -gt 500) {
            Write-Host "$($_.Parent.Name)\$($_.Name): $sizeMB MB"
        }
    }
}`

I will wait 15 seconds for this AppData subdirectory sizing task to start outputting results.

Used tool: schedule

I am waiting for the AppData subfolders scan to complete. I will show the folders that are larger than 500 MB in AppData Local and Roaming.

I will dive deeper into the largest AppData folders to find the exact files/backups we can clean:
1. **Docker (26.5 GB):** Check for large WSL virtual disks (`ext4.vhdx`).
2. **Google (12.2 GB):** Check if it is Chrome profile data or Google Drive local cache.
3. **Apple Computer (6.57 GB):** Check for old iOS backups in `MobileSync\Backup`.
4. **Cypress & npm-cache (7.2 GB total):** Verify files/folders inside them.

Ran command: `# Check Docker
Write-Host "--- Docker Files > 100MB ---"
Get-ChildItem -Path "C:\Users\ADMIN\AppData\Local\Docker" -File -Recurse -ErrorAction SilentlyContinue |
    Where-Object { $_.Length -gt 100MB } |
    Select-Object FullName, @{Name="SizeGB";Expression={[Math]::Round($_.Length / 1GB, 2)}} | Format-Table -AutoSize

# Check Google subdirectories
Write-Host "--- Google Subfolders ---"
Get-ChildItem -Path "C:\Users\ADMIN\AppData\Local\Google" -Directory | ForEach-Object {
    $size = (robocopy $_.FullName NULL /S /L /BYTES /NJH /NFL /NDL /R:0 /W:0 | Where-Object { $_ -match "Bytes\s+:\s+(\d+)" })
    if ($Matches[1]) {
        [PSCustomObject]@{
            Name = $_.Name
            SizeGB = [Math]::Round(([double]$Matches[1]) / 1GB, 2)
        }
    }
} | Format-Table -AutoSize

# Check Apple Computer backups
Write-Host "--- Apple Computer Subfolders ---"
Get-ChildItem -Path "C:\Users\ADMIN\AppData\Roaming\Apple Computer" -Directory -Recurse -ErrorAction SilentlyContinue | ForEach-Object {
    $size = (robocopy $_.FullName NULL /S /L /BYTES /NJH /NFL /NDL /R:0 /W:0 | Where-Object { $_ -match "Bytes\s+:\s+(\d+)" })
    if ($Matches[1]) {
        [PSCustomObject]@{
            Path = $_.FullName.Replace("C:\Users\ADMIN\AppData\Roaming\Apple Computer\", "")
            SizeGB = [Math]::Round(([double]$Matches[1]) / 1GB, 2)
        }
    }
} | Where-Object { $_.SizeGB -gt 0.1 } | Format-Table -AutoSize`

I will list the specific update files inside the iTunes folder to see exactly what files are there.

Ran command: `Get-ChildItem -Path "C:\Users\ADMIN\AppData\Roaming\Apple Computer\iTunes\iPad Software Updates" -File -ErrorAction SilentlyContinue | Select-Object Name, @{Name="SizeGB";Expression={[Math]::Round($_.Length / 1GB, 2)}}`

I have analyzed your C drive space usage, subfolders, and installed applications. Here is a breakdown of what is taking up space and how you can clean it up safely.

---

### 1. 📂 Safe to Delete Immediately (Saves ~14.2 GB)
These files can be deleted manually or using a command, and will not affect any of your apps.

*   **iTunes iPad Update File (6.37 GB):**
    *   **Path:** `C:\Users\ADMIN\AppData\Roaming\Apple Computer\iTunes\iPad Software Updates\iPad_Pro_HFR_17.7.4_21H414_Restore.ipsw`
    *   **Why:** This is a firmware download (.ipsw) used to restore or update an iPad Pro. Once the update is finished, this file is completely useless.
    *   **Action:** Delete the `.ipsw` file (or the whole `iPad Software Updates` folder).
*   **npm Cache (4.14 GB):**
    *   **Path:** `C:\Users\ADMIN\AppData\Local\npm-cache`
    *   **Why:** Caches Node packages.
    *   **Action:** Run `npm cache clean --force` in terminal, or delete the folder. (npm will download packages from scratch when you install packages next time, which is fine).
*   **Cypress Cache (3.10 GB):**
    *   **Path:** `C:\Users\ADMIN\AppData\Local\Cypress`
    *   **Why:** Stores old test runner binaries of Cypress.
    *   **Action:** Run `npx cypress cache clear` in terminal, or delete the folder.
*   **Windows Update Cache (653 MB):**
    *   **Path:** `C:\Windows\SoftwareDistribution\Download`
    *   **Why:** Stores downloaded Windows update installation files that have already been installed.
    *   **Action:** Delete everything **inside** this folder. (Do not delete the `Download` folder itself, just its contents).

---

### 2. 🛠️ Developer Caches (Safe to delete, but will re-download when you build projects) (Saves up to ~16.3 GB)
These directories store local caches of dependencies for coding. If you delete them, your next build/run of those languages will take a bit longer as it re-downloads everything, but it is 100% safe to do so.

*   **Maven `.m2` Cache (5.43 GB):** Delete `C:\Users\ADMIN\.m2\repository`
*   **Bazel Cache (4.91 GB):** Delete `C:\Users\ADMIN\_bazel_ADMIN`
*   **Gradle Cache (3.66 GB):** Delete `C:\Users\ADMIN\.gradle\caches`
*   **Rust Cargo Cache (2.32 GB):** Delete `C:\Users\ADMIN\.cargo\registry\cache`

---

### 3. 🖥️ Applications to Uninstall (Safe to Uninstall via Windows Settings)
These programs are currently installed and can be removed via **Windows Settings -> Apps -> Installed Apps**:

*   **McAfee Security Scan Plus / McAfee WebAdvisor:**
    *   **Why:** These are security scanners that usually bundle with other software (like Adobe reader) and are generally considered bloatware. Windows Defender (built-in) is already active and sufficient.
*   **Level Up (VNGGames) (~396 MB):**
    *   **Why:** Game launcher. If you do not play games from VNG anymore, this can be safely uninstalled.
*   **iVMS-4200 Lite (Hikvision) (~222 MB) / SmartPSS:**
    *   **Why:** Surveillance camera viewing tools. If you no longer use this PC to monitor security cameras, uninstall them.
*   **Java JDK 17 & JDK 23:**
    *   **Why:** You have two different JDKs installed. If you only build projects on one version (e.g., JDK 23 or JDK 17), you can uninstall the unused one.

---

### 4. 🐳 Advanced Space Reclamation: Docker Virtual Disk (26.5 GB)
Your WSL2 Docker Desktop disk image is taking up **25.68 GB** at:
`C:\Users\ADMIN\AppData\Local\Docker\wsl\disk\docker_data.vhdx`

> [!WARNING]
> In WSL2, even if you delete containers or run `docker system prune` to clear space, the `.vhdx` virtual disk file on Windows **never automatically shrinks** on your SSD. 

To reclaim this space:
1. Open terminal and run Docker cleanup to delete unused containers/images:
   ```bash
   docker system prune -a --volumes
   ```
2. Close Docker Desktop completely.
3. Open PowerShell as **Administrator** and shut down WSL:
   ```powershell
   wsl --shutdown
   ```
4. Compact the virtual disk using `diskpart`:
   ```cmd
   diskpart
   # Inside diskpart, run these:
   select vdisk file="C:\Users\ADMIN\AppData\Local\Docker\wsl\disk\docker_data.vhdx"
   attach vdisk readonly
   compact vdisk
   detach vdisk
   exit
   ```
   *This will shrink the physical size of the `.vhdx` file on your C drive to only what is actually being used by Docker.*

---

### Summary of what is using space on C:\
*   **Users Folder:** 117.31 GB (mostly in `AppData`: 88.90 GB)
*   **Windows System Folder:** 46.82 GB (Standard)
*   **Program Files + Program Files (x86):** 26.63 GB (Standard)
*   **System Hibernation file (`hiberfil.sys`):** 3.13 GB *(Can be deleted by disabling hibernation if you never put your PC to sleep/hibernate).*