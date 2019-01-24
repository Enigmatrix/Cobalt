using System;
using System.Reactive.Concurrency;

namespace Cobalt.Engine
{
    public class Program
    {
        public static void Main(string[] args)
        {
            var watcher = new AppWatcher();
            watcher.Start();

            EventLoop();
        }

        private static void EventLoop()
        {
            //keep getting messages
            while (true)
            {
                if (Win32.GetMessage(out var msg, IntPtr.Zero, 0, 0) == 0) break;

                //even ms docs say that you dont need to understand these
                Win32.TranslateMessage(ref msg);
                Win32.DispatchMessage(ref msg);
            }
        }
    }
}