using System;
using System.Collections.Generic;
using System.ComponentModel;
using System.Data;
using System.Drawing;
using System.Linq;
using System.Text;
using System.Threading.Tasks;
using System.Windows.Forms;

namespace Cobalt.Engine
{
    public partial class InvisibleForm : Form
    {
        public InvisibleForm()
        {
            InitializeComponent();

            FormBorderStyle = FormBorderStyle.None;
            ShowInTaskbar = false;
            Load += (o, e) => Size = new Size(0,0);
        }
    }
}
