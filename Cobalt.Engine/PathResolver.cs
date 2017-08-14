using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using System.Management;
using System.Runtime.InteropServices;
using System.Text;
using Serilog;
using static Cobalt.Engine.Win32;

namespace Cobalt.Engine
{
    public class PathResolver
    {
        public string ResolveWindowPath(IntPtr hwnd)
        {
            if (!IsWindowVisible(hwnd))
            {
                Log.Information("Invisible Window, {hwnd}", hwnd);
                return null;
            }

            var pid = GetWindowProcessId(hwnd);
            var fileName = GetMainModuleFilePath(pid);
            //if it isnt a app hosted by ApplicationFrameHost (a UWP/Win10 app)
            if (!string.Equals(fileName, ApplicationFrameHost, StringComparison.OrdinalIgnoreCase))
                return fileName;

            Log.Information("Detected Win10/UWP App");
            fileName = GetMainModuleFilePath(GetModernAppProcessId(hwnd, pid));

            return fileName;
        }

        public string GetMainModuleFilePath(uint pid)
        {
            if (pid == 0) return null;
            var ret = QueryFullProcessImageNameStrategy(pid);
            //check if this strategy works, else use another
            if (IsValidWindowFilePath(ret)) return ret;
            Log.Information("QFPIN Strategy failed, using WMI");
            ret = WmiStrategy(pid);
            if (IsValidWindowFilePath(ret)) return ret;
            Log.Information("Unable to ascertain window path, {pid}", pid);
            return null;
        }

        private string WmiStrategy(uint pid)
        {
            //TODO CACHE IMPLEMENTATION
            var wmiQueryString = $"SELECT ProcessId, ExecutablePath FROM Win32_Process WHERE ProcessId = {pid}";
            using (var searcher = new ManagementObjectSearcher(wmiQueryString))
            {
                using (var results = searcher.Get())
                {
                    return (string) results.Cast<ManagementBaseObject>().First()["ExecutablePath"];
                }
            }
        }

        public bool IsValidWindowFilePath(string path)
        {
            return !string.IsNullOrWhiteSpace(path) && File.Exists(path);
        }

        public string QueryFullProcessImageNameStrategy(uint pid)
        {
            var proc = OpenProcess(
                QueryLimitedInformation | ProcessVmRead, false, pid);
            //arbitrary
            var pathLen = 512;
            var path = new StringBuilder(pathLen);
            var finalPath = !QueryFullProcessImageName(proc, 0, path, ref pathLen) ? null : path.ToString();
            CloseHandle(proc);
            return finalPath;
        }

        private uint GetModernAppProcessId(IntPtr hwnd, uint pid)
        {
            //now this is a bit tricky. Modern apps are hosted inside ApplicationFrameHost process, so we need to find
            //child window which does NOT belong to this process.This should be the process we need. (in main checker)

            //But, fuck Win32/Metro, it doesnt allow me to find the app before the Splash finishes loading
            //So, I repeat the process while there are no windows, which is when the app closes, in the case
            //of open app -> immediately close (or atleast b4 the splash finishes)

            List<IntPtr> children;
            while ((children = GetChildWindows(hwnd)).Count != 0)
                foreach (var childHwnd in children)
                {
                    var childPid = GetWindowProcessId(childHwnd);
                    if (childPid != pid)
                        return childPid;
                }
            //error! (technically 0 is an invalid process id for a window)
            return 0;
        }


        private static List<IntPtr> GetChildWindows(IntPtr hwnd)
        {
            var result = new List<IntPtr>();
            var listHandle = GCHandle.Alloc(result);
            try
            {
                EnumWindowProc childProc = EnumChildWindow;
                EnumChildWindows(hwnd, childProc, GCHandle.ToIntPtr(listHandle));
            }
            finally
            {
                if (listHandle.IsAllocated)
                    listHandle.Free();
            }
            return result;
        }

        private static bool EnumChildWindow(IntPtr handle, IntPtr pointer)
        {
            var gch = GCHandle.FromIntPtr(pointer);
            var list = gch.Target as List<IntPtr>;
            if (list == null)
                throw new InvalidCastException("GCHandle Target could not be cast as List<IntPtr>");
            list.Add(handle);

            return true;
        }

        public static uint GetWindowProcessId(IntPtr hwnd)
        {
            uint pid;
            GetWindowThreadProcessId(hwnd, out pid);
            return pid;
        }
    }
}