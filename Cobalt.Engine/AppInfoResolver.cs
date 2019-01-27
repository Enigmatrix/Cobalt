using System;
using System.Collections.Generic;
using System.ComponentModel;
using System.IO;
using System.Runtime.InteropServices;
using Cobalt.Common.Data.Entities;
using Cobalt.Common.Util;
using static Cobalt.Engine.Win32;

namespace Cobalt.Engine
{
    public class AppInfoResolver
    {
        private static string ApplicationFrameHost { get; } = @"C:\windows\system32\ApplicationFrameHost.exe";
        private static string JavaProgram { get; } = @"java.exe";

        public string WindowPath(IntPtr hwnd)
        {
            if (!IsWindowVisible(hwnd)) return null;

            GetWindowThreadProcessId(hwnd, out var pid);

            var (pparams, proc) = WindowPathInfo(pid);
            var path = pparams.ImagePathName.ToString(proc);

            //TODO Squirrel Apps => `<Local>/<AppName>/<app-x.x.x>/<shit>.exe`
            //  exists `<Local>/<AppName>/<app-x.x.x>/squirrel.exe`
            //  exists `<Local>/<AppName>/update.exe`

            //Java Apps => get CommandLine starts with java somewhere
            //TODO make sure the Java binary is real?
            if (string.Equals(Path.GetFileName(path), JavaProgram, StringComparison.OrdinalIgnoreCase))
            {
                var cmd = pparams.CommandLine.ToString(proc);
                var args = CommandLineToArgs(cmd);
                var pre = Array.IndexOf(args, "-jar");
                path = args[pre + 1];
            }

            CloseHandle(proc);

            //Windows Store Apps => Magic
            if (!string.Equals(path, ApplicationFrameHost, StringComparison.OrdinalIgnoreCase))
                return path;

            (pparams, proc) = WindowPathInfo(GetModernAppProcessId(hwnd, pid));
            path = pparams.ImagePathName.ToString(proc);
            CloseHandle(proc);

            return path;
        }

        private (RTL_USER_PROCESS_PARAMETERS, IntPtr) WindowPathInfo(uint pid)
        {
            var proc = OpenProcess(ProcessAccessFlags.QueryInformation | ProcessAccessFlags.VirtualMemoryRead,
                false, pid).Handled();

            var pbi = new PROCESS_BASIC_INFORMATION();
            NtQueryInformationProcess(proc, PROCESSINFOCLASS.ProcessBasicInformation,
                ref pbi, pbi.Size, out _).Handled();

            var peb = ReadProcessMemory<PEB>(proc, pbi.PebBaseAddress);
            var pparams = ReadProcessMemory<RTL_USER_PROCESS_PARAMETERS>(proc, peb.ProcessParameters);
            return (pparams, proc);
        }

        public App WithDetails(App app)
        {
            var name = AppResource.GetAppName(app.Path);
            var (icon, color) = AppResource.GetAppIconAndColor(app.Path);
            app.Color = color;
            app.Icon = new Lazy<byte[]>(() => icon);
            app.Name = name;
            return app;
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
                    GetWindowThreadProcessId(childHwnd, out var childPid);
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
                EnumWindowsProc childProc = EnumChildWindow;
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
            if (!(gch.Target is List<IntPtr> list))
                throw new InvalidCastException("GCHandle Target could not be cast as List<IntPtr>");
            list.Add(handle);

            return true;
        }

        public static string[] CommandLineToArgs(string commandLine)
        {
            var argv = CommandLineToArgvW(commandLine, out var argc);
            if (argv == IntPtr.Zero)
                throw new Win32Exception();
            try
            {
                var args = new string[argc];
                for (var i = 0; i < args.Length; i++)
                {
                    var p = Marshal.ReadIntPtr(argv, i * IntPtr.Size);
                    args[i] = Marshal.PtrToStringUni(p);
                }

                return args;
            }
            finally
            {
                Marshal.FreeHGlobal(argv);
            }
        }
    }
}