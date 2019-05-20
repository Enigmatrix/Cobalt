using System;
using System.Collections.Generic;
using System.Diagnostics;
using System.Linq;
using System.Runtime.InteropServices;
using System.Text;
using System.Threading;
using System.Threading.Tasks;

namespace Cobalt.Tests.Integration
{
    public class ProcessUtil : IDisposable
    {

        [DllImport("user32.dll", SetLastError = true)]
        static extern void SwitchToThisWindow(IntPtr hWnd, bool turnOn);
        [DllImport("user32.dll")]
        static extern bool SetForegroundWindow(IntPtr hWnd);
        [DllImport("user32.dll")]
        static extern bool SetActiveWindow(IntPtr hWnd);
        [DllImport("user32.dll", SetLastError = true)]
        [return: MarshalAs(UnmanagedType.Bool)]
        static extern bool GetKeyboardState(byte[] lpKeyState);
        [DllImport("user32.dll", SetLastError = true)]
        static extern void keybd_event(byte bVk, byte bScan, int dwFlags, int dwExtraInfo);

        private Dictionary<string, Process> processes = new Dictionary<string, Process>();

        public ProcessUtil(params string[] procs)
        {
            foreach (var proc in procs)
            {
                var process = Process.Start(new ProcessStartInfo
                {
                    FileName = proc,
                    WindowStyle = ProcessWindowStyle.Normal
                });
                processes.Add(proc, process);
                Thread.Sleep(500);
                //Microsoft.VisualBasic.Interaction.AppActivate(process.Id);
                //SetForegroundWindow(process.MainWindowHandle);
            }
        }

        public void SetFgInternal(string name)
        {

                var keyStates = new byte[256];
                var VK_MENU = 0x12;
                if (GetKeyboardState(keyStates))
                    if ((keyStates[VK_MENU] & 0x80) == 0)
                        keybd_event((byte)VK_MENU, 0, 1 | 0, 0);

                SetForegroundWindow(processes[name].MainWindowHandle);

                if (GetKeyboardState(keyStates))
                    if ((keyStates[VK_MENU] & 0x80) == 0)
                        keybd_event((byte)VK_MENU, 0, 1 | 2, 0);
        }

        public void Dispose()
        {
            foreach(var proc in processes)
            {
                proc.Value.Kill();
            }
            processes.Clear();
        }
    }
}
