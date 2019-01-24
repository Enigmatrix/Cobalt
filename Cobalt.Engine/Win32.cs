// ReSharper disable InconsistentNaming
// ReSharper disable MemberCanBePrivate.Global
using System;
using System.ComponentModel;
using System.Drawing;
using System.Runtime.InteropServices;

namespace Cobalt.Engine
{
    public static class Win32
    {
        #region Delegates

        public delegate void WinEventDelegate(IntPtr hWinEventHook, WinEvent eventType,
            IntPtr hwnd, int idObject, int idChild, uint dwEventThread, uint dwmsEventTime);

        public delegate bool EnumWindowsProc(IntPtr hwnd, IntPtr lParam);

        #endregion

        #region Structs

        [StructLayout(LayoutKind.Sequential)]
        public struct PROCESS_BASIC_INFORMATION
        {
            public IntPtr ExitStatus;
            public IntPtr PebBaseAddress;
            public IntPtr AffinityMask;
            public IntPtr BasePriority;
            public UIntPtr UniqueProcessId;
            public IntPtr InheritedFromUniqueProcessId;

            public int Size => Marshal.SizeOf(typeof(PROCESS_BASIC_INFORMATION));
        }
        
        [StructLayout(LayoutKind.Sequential)]
        public struct MSG
        {
            public readonly IntPtr Hwnd;
            public readonly uint Message;
            public readonly IntPtr WParam;
            public readonly IntPtr LParam;
            public readonly uint Time;
            public readonly Point Point;
        }

        //use this for more expansive structure: https://docs.rs/ntapi/0.2.0/ntapi/ntpebteb/struct.PEB.html
        [StructLayout(LayoutKind.Explicit, Size = 0x40)]
        public struct PEB
        {
            [FieldOffset(0x000)]
            public byte InheritedAddressSpace;
            [FieldOffset(0x001)]
            public byte ReadImageFileExecOptions;
            [FieldOffset(0x002)]
            public byte BeingDebugged;
            [FieldOffset(0x003)]

            public byte Spare;
            [FieldOffset(0x008)]
            public IntPtr Mutant;
            [FieldOffset(0x010)]
            public IntPtr ImageBaseAddress;     // (PVOID) 
            [FieldOffset(0x018)]
            public IntPtr Ldr;                  // (PPEB_LDR_DATA)
            [FieldOffset(0x020)]
            public IntPtr ProcessParameters;    // (PRTL_USER_PROCESS_PARAMETERS)
            [FieldOffset(0x028)]
            public IntPtr SubSystemData;        // (PVOID) 
            [FieldOffset(0x030)]
            public IntPtr ProcessHeap;          // (PVOID) 
            [FieldOffset(0x038)]
            public IntPtr FastPebLock;          // (PRTL_CRITICAL_SECTION)
        }

        [StructLayout(LayoutKind.Explicit, Size = 0xa0)]
        public struct RTL_USER_PROCESS_PARAMETERS
        {
            /*
            public uint MaximumLength;
            public uint Length;
            public uint Flags;
            public uint DebugFlags;
            public IntPtr ConsoleHandle;
            public uint ConsoleFlags;
            public IntPtr StdInputHandle;
            public IntPtr StdOutputHandle;
            public IntPtr StdErrorHandle;
            public UNICODE_STRING CurrentDirectoryPath;
            public IntPtr CurrentDirectoryHandle;
            public UNICODE_STRING DllPath;
            public UNICODE_STRING ImagePathName;*/
            [FieldOffset(0x70)]
            public UNICODE_STRING CommandLine;
        }
        
        [StructLayout(LayoutKind.Sequential)]
        public struct UNICODE_STRING
        {
            public ushort Length;
            public ushort MaximumLength;
            public IntPtr Buffer;

            public string ToString(IntPtr proc)
            {
                var s = new string('\0', Length / 2);
                if (!ReadProcessMemory(proc, Buffer, s, Length, out _))
                    throw new Win32Exception(Marshal.GetLastWin32Error());
                return s;
            }
        }

        #endregion

        #region Enum and Consts

        public enum PROCESSINFOCLASS
        {
            ProcessBasicInformation = 0, // 0, q: PROCESS_BASIC_INFORMATION, PROCESS_EXTENDED_BASIC_INFORMATION
            ProcessQuotaLimits, // qs: QUOTA_LIMITS, QUOTA_LIMITS_EX
            ProcessIoCounters, // q: IO_COUNTERS
            ProcessVmCounters, // q: VM_COUNTERS, VM_COUNTERS_EX
            ProcessTimes, // q: KERNEL_USER_TIMES
            ProcessBasePriority, // s: KPRIORITY
            ProcessRaisePriority, // s: ULONG
            ProcessDebugPort, // q: HANDLE
            ProcessExceptionPort, // s: HANDLE
            ProcessAccessToken, // s: PROCESS_ACCESS_TOKEN
            ProcessLdtInformation, // 10
            ProcessLdtSize,
            ProcessDefaultHardErrorMode, // qs: ULONG
            ProcessIoPortHandlers, // (kernel-mode only)
            ProcessPooledUsageAndLimits, // q: POOLED_USAGE_AND_LIMITS
            ProcessWorkingSetWatch, // q: PROCESS_WS_WATCH_INFORMATION[], s: void
            ProcessUserModeIOPL,
            ProcessEnableAlignmentFaultFixup, // s: BOOLEAN
            ProcessPriorityClass, // qs: PROCESS_PRIORITY_CLASS
            ProcessWx86Information,
            ProcessHandleCount, // 20, q: ULONG, PROCESS_HANDLE_INFORMATION
            ProcessAffinityMask, // s: KAFFINITY
            ProcessPriorityBoost, // qs: ULONG
            ProcessDeviceMap, // qs: PROCESS_DEVICEMAP_INFORMATION, PROCESS_DEVICEMAP_INFORMATION_EX
            ProcessSessionInformation, // q: PROCESS_SESSION_INFORMATION
            ProcessForegroundInformation, // s: PROCESS_FOREGROUND_BACKGROUND
            ProcessWow64Information, // q: ULONG_PTR
            ProcessImageFileName, // q: UNICODE_STRING
            ProcessLUIDDeviceMapsEnabled, // q: ULONG
            ProcessBreakOnTermination, // qs: ULONG
            ProcessDebugObjectHandle, // 30, q: HANDLE
            ProcessDebugFlags, // qs: ULONG
            ProcessHandleTracing, // q: PROCESS_HANDLE_TRACING_QUERY, s: size 0 disables, otherwise enables
            ProcessIoPriority, // qs: ULONG
            ProcessExecuteFlags, // qs: ULONG
            ProcessResourceManagement,
            ProcessCookie, // q: ULONG
            ProcessImageInformation, // q: SECTION_IMAGE_INFORMATION
            ProcessCycleTime, // q: PROCESS_CYCLE_TIME_INFORMATION
            ProcessPagePriority, // q: ULONG
            ProcessInstrumentationCallback, // 40
            ProcessThreadStackAllocation, // s: PROCESS_STACK_ALLOCATION_INFORMATION, PROCESS_STACK_ALLOCATION_INFORMATION_EX
            ProcessWorkingSetWatchEx, // q: PROCESS_WS_WATCH_INFORMATION_EX[]
            ProcessImageFileNameWin32, // q: UNICODE_STRING
            ProcessImageFileMapping, // q: HANDLE (input)
            ProcessAffinityUpdateMode, // qs: PROCESS_AFFINITY_UPDATE_MODE
            ProcessMemoryAllocationMode, // qs: PROCESS_MEMORY_ALLOCATION_MODE
            ProcessGroupInformation, // q: USHORT[]
            ProcessTokenVirtualizationEnabled, // s: ULONG
            ProcessConsoleHostProcess, // q: ULONG_PTR
            ProcessWindowInformation, // 50, q: PROCESS_WINDOW_INFORMATION
            ProcessHandleInformation, // q: PROCESS_HANDLE_SNAPSHOT_INFORMATION // since WIN8
            ProcessMitigationPolicy, // s: PROCESS_MITIGATION_POLICY_INFORMATION
            ProcessDynamicFunctionTableInformation,
            ProcessHandleCheckingMode,
            ProcessKeepAliveCount, // q: PROCESS_KEEPALIVE_COUNT_INFORMATION
            ProcessRevokeFileHandles, // s: PROCESS_REVOKE_FILE_HANDLES_INFORMATION
            MaxProcessInfoClass
        }

        [Flags]
        public enum HookFlags : uint
        {
            OUTOFCONTEXT = 0x0000, // Events are ASYNC
            SKIPOWNTHREAD = 0x0001, // Don't call back for events on installer's thread
            SKIPOWNPROCESS = 0x0002, // Don't call back for events on installer's process
            INCONTEXT = 0x0004, // Events are SYNC, this causes your dll to be injected into every process
        }

        public enum WinEvent : uint
        {
            MIN = 0x00000001,
            MAX = 0x7FFFFFFF,
            SYSTEM_SOUND = 0x0001,
            SYSTEM_ALERT = 0x0002,
            SYSTEM_FOREGROUND = 0x0003,
            SYSTEM_MENUSTART = 0x0004,
            SYSTEM_MENUEND = 0x0005,
            SYSTEM_MENUPOPUPSTART = 0x0006,
            SYSTEM_MENUPOPUPEND = 0x0007,
            SYSTEM_CAPTURESTART = 0x0008,
            SYSTEM_CAPTUREEND = 0x0009,
            SYSTEM_MOVESIZESTART = 0x000A,
            SYSTEM_MOVESIZEEND = 0x000B,
            SYSTEM_CONTEXTHELPSTART = 0x000C,
            SYSTEM_CONTEXTHELPEND = 0x000D,
            SYSTEM_DRAGDROPSTART = 0x000E,
            SYSTEM_DRAGDROPEND = 0x000F,
            SYSTEM_DIALOGSTART = 0x0010,
            SYSTEM_DIALOGEND = 0x0011,
            SYSTEM_SCROLLINGSTART = 0x0012,
            SYSTEM_SCROLLINGEND = 0x0013,
            SYSTEM_SWITCHSTART = 0x0014,
            SYSTEM_SWITCHEND = 0x0015,
            SYSTEM_MINIMIZESTART = 0x0016,
            SYSTEM_MINIMIZEEND = 0x0017,
            SYSTEM_DESKTOPSWITCH = 0x0020,
            SYSTEM_END = 0x00FF,
            OEM_DEFINED_START = 0x0101,
            OEM_DEFINED_END = 0x01FF,
            UIA_EVENTID_START = 0x4E00,
            UIA_EVENTID_END = 0x4EFF,
            UIA_PROPID_START = 0x7500,
            UIA_PROPID_END = 0x75FF,
            CONSOLE_CARET = 0x4001,
            CONSOLE_UPDATE_REGION = 0x4002,
            CONSOLE_UPDATE_SIMPLE = 0x4003,
            CONSOLE_UPDATE_SCROLL = 0x4004,
            CONSOLE_LAYOUT = 0x4005,
            CONSOLE_START_APPLICATION = 0x4006,
            CONSOLE_END_APPLICATION = 0x4007,
            CONSOLE_END = 0x40FF,
            OBJECT_CREATE = 0x8000, // hwnd ID idChild is created item
            OBJECT_DESTROY = 0x8001, // hwnd ID idChild is destroyed item
            OBJECT_SHOW = 0x8002, // hwnd ID idChild is shown item
            OBJECT_HIDE = 0x8003, // hwnd ID idChild is hidden item
            OBJECT_REORDER = 0x8004, // hwnd ID idChild is parent of zordering children
            OBJECT_FOCUS = 0x8005, // hwnd ID idChild is focused item,
            OBJECT_SELECTION = 0x8006, // hwnd ID idChild is selected item (if only one), or idChild is OBJID_WINDOW if complex,
            OBJECT_SELECTIONADD = 0x8007, // hwnd ID idChild is item added,
            OBJECT_SELECTIONREMOVE = 0x8008, // hwnd ID idChild is item removed,
            OBJECT_SELECTIONWITHIN = 0x8009, // hwnd ID idChild is parent of changed selected items,
            OBJECT_STATECHANGE = 0x800A, // hwnd ID idChild is item w/ state change,
            OBJECT_LOCATIONCHANGE = 0x800B, // hwnd ID idChild is moved/sized item,
            OBJECT_NAMECHANGE = 0x800C, // hwnd ID idChild is item w/ name change,
            OBJECT_DESCRIPTIONCHANGE = 0x800D, // hwnd ID idChild is item w/ desc change,
            OBJECT_VALUECHANGE = 0x800E, // hwnd ID idChild is item w/ value change,
            OBJECT_PARENTCHANGE = 0x800F, // hwnd ID idChild is item w/ new parent,
            OBJECT_HELPCHANGE = 0x8010, // hwnd ID idChild is item w/ help change,
            OBJECT_DEFACTIONCHANGE = 0x8011, // hwnd ID idChild is item w/ def action change,
            OBJECT_ACCELERATORCHANGE = 0x8012, // hwnd ID idChild is item w/ keybd accel change,
            OBJECT_INVOKED = 0x8013, // hwnd ID idChild is item invoked,
            OBJECT_TEXTSELECTIONCHANGED = 0x8014, // hwnd ID idChild is item w? test selection change,
            OBJECT_CONTENTSCROLLED = 0x8015,
            SYSTEM_ARRANGMENTPREVIEW = 0x8016,
            OBJECT_END = 0x80FF,
            AIA_START = 0xA000,
            AIA_END = 0xAFFF,
        }

        [Flags]
        public enum ProcessAccessFlags : uint
        {
            All = 0x001F0FFF,
            Terminate = 0x00000001,
            CreateThread = 0x00000002,
            VirtualMemoryOperation = 0x00000008,
            VirtualMemoryRead = 0x00000010,
            VirtualMemoryWrite = 0x00000020,
            DuplicateHandle = 0x00000040,
            CreateProcess = 0x000000080,
            SetQuota = 0x00000100,
            SetInformation = 0x00000200,
            QueryInformation = 0x00000400,
            QueryLimitedInformation = 0x00001000,
            Synchronize = 0x00100000
        }

        #endregion

        #region Functions

        [DllImport("NTDLL.DLL", SetLastError = true)]
        public static extern int NtQueryInformationProcess(IntPtr hProcess, PROCESSINFOCLASS pic,
            ref PROCESS_BASIC_INFORMATION pbi, int cb, out int pSize);

        [DllImport("user32.dll")]
        public static extern IntPtr SetWinEventHook(WinEvent eventMin, WinEvent eventMax, IntPtr
                hmodWinEventProc, WinEventDelegate lpfnWinEventProc, uint idProcess,
            uint idThread, HookFlags dwFlags);

        
        /// <summary>
        ///     Enumerates all top-level windows on the screen by passing the handle to each window, in turn, to an
        ///     application-defined callback function. <see cref="EnumWindows" /> continues until the last top-level window is
        ///     enumerated or the callback function returns FALSE.
        ///     <para>
        ///         Go to https://msdn.microsoft.com/en-us/library/windows/desktop/ms633497%28v=vs.85%29.aspx for more
        ///         information
        ///     </para>
        /// </summary>
        /// <param name="lpEnumFunc">
        ///     C++ ( lpEnumFunc [in]. Type: WNDENUMPROC )<br />A pointer to an application-defined callback
        ///     function. For more information, see
        ///     <see cref="!:https://msdn.microsoft.com/en-us/library/windows/desktop/ms633498%28v=vs.85%29.aspx">EnumWindowsProc</see>
        ///     .
        /// </param>
        /// <param name="lParam">
        ///     C++ ( lParam [in]. Type: LPARAM )<br />An application-defined value to be passed to the callback
        ///     function.
        /// </param>
        /// <returns>
        ///     <c>true</c> if the return value is nonzero., <c>false</c> otherwise. If the function fails, the return value
        ///     is zero.<br />To get extended error information, call GetLastError.<br />If <see cref="EnumWindowsProc" /> returns
        ///     zero, the return value is also zero. In this case, the callback function should call SetLastError to obtain a
        ///     meaningful error code to be returned to the caller of <see cref="EnumWindows" />.
        /// </returns>
        /// <remarks>
        ///     The <see cref="EnumWindows" /> function does not enumerate child windows, with the exception of a few
        ///     top-level windows owned by the system that have the WS_CHILD style.
        ///     <para />
        ///     This function is more reliable than calling the
        ///     <see cref="!:https://msdn.microsoft.com/en-us/library/windows/desktop/ms633515%28v=vs.85%29.aspx">GetWindow</see>
        ///     function in a loop. An application that calls the GetWindow function to perform this task risks being caught in an
        ///     infinite loop or referencing a handle to a window that has been destroyed.<br />Note For Windows 8 and later,
        ///     EnumWindows enumerates only top-level windows of desktop apps.
        /// </remarks>
        [DllImport("user32.dll", SetLastError = true)]
        [return: MarshalAs(UnmanagedType.Bool)]
        public static extern bool EnumWindows(EnumWindowsProc lpEnumFunc, IntPtr lParam);

        [DllImport("user32.dll", SetLastError = true)]
        public static extern IntPtr GetForegroundWindow();
        
        [DllImport("user32.dll", SetLastError = true)]
        public static extern uint GetWindowThreadProcessId(IntPtr hwnd, out uint lpdwProcessId);

        [DllImport("user32.dll", SetLastError = true)]
        public static extern int GetMessage(out MSG lpMsg, IntPtr hWnd, uint wMsgFilterMin, uint wMsgFilterMax);

        [DllImport("user32.dll", SetLastError = true)]
        public static extern bool TranslateMessage(ref MSG lpMsg);

        [DllImport("user32.dll", SetLastError = true)]
        public static extern IntPtr DispatchMessage(ref MSG lpMsg);

        [DllImport("kernel32.dll", SetLastError = true)]
        public static extern IntPtr OpenProcess(
            ProcessAccessFlags processAccess,
            bool bInheritHandle,
            uint processId
        );        

        
        [DllImport("kernel32.dll", SetLastError = true)]
        private static extern bool ReadProcessMemory(IntPtr hProcess, IntPtr lpBaseAddress, IntPtr lpBuffer, int dwSize, out uint lpNumberOfBytesRead);
        
        [DllImport("kernel32.dll", SetLastError = true)]
        private static extern bool ReadProcessMemory(IntPtr hProcess, IntPtr lpBaseAddress, [MarshalAs(UnmanagedType.LPWStr)] string lpBuffer, uint dwSize, out uint lpNumberOfBytesRead);

        #endregion

        #region Derived

        public static T ReadProcessMemory<T>(IntPtr hProcess, IntPtr lpBaseAddress)
            where T: struct
        {
            var rsz = Marshal.SizeOf<T>();
            var pnt = Marshal.AllocHGlobal(rsz);
            if (!ReadProcessMemory(hProcess, lpBaseAddress, pnt, rsz, out _))
                throw new Win32Exception(Marshal.GetLastWin32Error());
            var ret = Marshal.PtrToStructure<T>(pnt);
            Marshal.FreeHGlobal(pnt);
            return ret;
        }

        #endregion
    }
}