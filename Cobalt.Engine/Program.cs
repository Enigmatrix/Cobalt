using System;
using System.Collections.Generic;
using System.Linq;
using System.Threading.Tasks;
using System.Windows.Forms;
using Cobalt.Engine.Util;
using Serilog;

namespace Cobalt.Engine
{
    static class Program
    {
        //TODO

        private static readonly Win32.WinEventProc _foregroundWindowChangedCallback = ForegroundWindowChangedCallback;
        /// <summary>
        /// The main entry point for the application.
        /// </summary>
        [STAThread]
        static void Main()
        {
            Application.EnableVisualStyles();
            Application.SetCompatibleTextRenderingDefault(false);

            Log.Logger = new LoggerConfiguration()
                .WriteTo.File("./log.txt")
                .CreateLogger();
            Log.Information("NEW SESSION");
            var hooker = new HookManager();
            hooker.Hook(Win32.WinEvent.EVENT_SYSTEM_FOREGROUND, _foregroundWindowChangedCallback);
            Application.Run(new InvisibleMessageWindow());
        }
        private static void ForegroundWindowChangedCallback(
            IntPtr hwineventhook, uint eventtype, IntPtr hwnd, int idobject, int idchild, uint dweventthread,
            uint dwmseventtime)
        {
            Log.Information("USERFUL HERHERHERHE");
        }
    }
}
