using System;
using System.Collections.Generic;
using System.ComponentModel;
using System.Data;
using System.Drawing;
using System.Linq;
using System.Text;
using System.Threading.Tasks;
using System.Windows.Forms;
using Cobalt.Engine.Util;

namespace Cobalt.Engine
{
    public partial class InvisibleMessageWindow : Form
    {
        //TODO CLEANUP
        private readonly Dictionary<Win32.WindowMessages,
            Action<IntPtr, Win32.WindowMessages, IntPtr, IntPtr>> handlers = 
            new Dictionary<Win32.WindowMessages, Action<IntPtr, Win32.WindowMessages, IntPtr, IntPtr>>();

        public InvisibleMessageWindow()
        {
            InitializeComponent();

            FormBorderStyle = FormBorderStyle.None;
            ShowInTaskbar = false;
            Load += (o, e) => Size = new Size(0,0);
        }


        public static InvisibleMessageWindow Instance = new InvisibleMessageWindow();

        public void AddHook(Win32.WindowMessages msg, Action<IntPtr, Win32.WindowMessages, IntPtr, IntPtr> handler)
        {
            handlers[msg] = handler;
        }

        protected override void WndProc(ref Message m)
        {
            var msg = (Win32.WindowMessages) m.Msg;
            if (handlers.ContainsKey(msg))
                handlers[msg](m.HWnd, msg, m.WParam, m.LParam);
            base.WndProc(ref m);
        }

        //hide from alt-tab and task manager 'user apps'
        protected override CreateParams CreateParams
        {
            get
            {
                var cp = base.CreateParams;
                cp.ExStyle |= 0x80;  // Turn on WS_EX_TOOLWINDOW
                return cp;
            }
        }
    }
}
