using System;
using System.Collections.Generic;
using System.ComponentModel;
using System.Linq;
using System.Runtime.InteropServices;
using System.Text;
using System.Threading.Tasks;

using static Cobalt.Engine.Win32;

namespace Cobalt.Engine
{
    public class AppWatcher
    {
        
        private readonly WinEventDelegate _foregroundWindowChangedCallback;

        public AppWatcher()
        {
            _foregroundWindowChangedCallback = ForegroundWindowChangedCallback;
        }

        public void Start()
        {
            var hook = SetWinEventHook(WinEvent.SYSTEM_FOREGROUND, WinEvent.SYSTEM_FOREGROUND,
                IntPtr.Zero, _foregroundWindowChangedCallback, 0, 0, HookFlags.OUTOFCONTEXT);
            if (hook == IntPtr.Zero)
                throw new Win32Exception(Marshal.GetLastWin32Error());
            Console.WriteLine("{0}, {1}", Environment.Is64BitOperatingSystem, Environment.Is64BitProcess);
        }

        private void ForegroundWindowChangedCallback(
            IntPtr hwineventhook, WinEvent eventtype, IntPtr hwnd, int idobject, int idchild, uint dweventthread,
            uint dwmseventtime)
        {
            var dwmsTimestamp = DateTime.Now.AddMilliseconds(dwmseventtime - Environment.TickCount);

            GetWindowThreadProcessId(hwnd, out var pid);
            var proc = OpenProcess(ProcessAccessFlags.QueryInformation|ProcessAccessFlags.VirtualMemoryRead, false, pid);
            if (proc == IntPtr.Zero)
            {
                Console.WriteLine("Access Error");
                return;
            }

            var pbi = new PROCESS_BASIC_INFORMATION();
            var hr = NtQueryInformationProcess(proc, PROCESSINFOCLASS.ProcessBasicInformation, ref pbi, pbi.Size, out var pbiWSz);
            if (hr != 0)
                    throw new Win32Exception(hr);

            var peb = ReadProcessMemory<PEB>(proc, pbi.PebBaseAddress);
            var pparams = ReadProcessMemory<RTL_USER_PROCESS_PARAMETERS>(proc, peb.ProcessParameters);
            var str = pparams.CommandLine.ToString(proc);
            Console.WriteLine(str);
        }
    }
}
